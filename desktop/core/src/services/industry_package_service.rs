//! Industry Data Model foundations - the third piece (transactional
//! install/rollback machinery) alongside `models::industry_package` (the
//! manifest format) and `repositories::industry_package_repo` (the
//! registry tables). See the dev spec ("Top 10 Industry Data Models &
//! Packaged Business Apps") for the epic this exists ahead of - this
//! module is deliberately industry-agnostic; it installs whatever
//! manifest it's given, one Field Service / Property Management / etc.
//! package at a time, once one is actually written against this format.
//! No industry-specific logic belongs here.
//!
//! Deliberately NOT built yet (foundation phase only):
//! - Update-with-diff that preserves admin customizations (spec 1.1/13.3's
//!   "package updates must not silently overwrite them") - re-installing
//!   an already-installed `package_id` is rejected outright (see
//!   `run_install`'s conflict check) rather than attempted as an update.
//! - Destructive uninstall with dependency/data-disposition review (spec
//!   lifecycle step 9's non-default path) - only non-destructive
//!   `deactivate`/`reactivate` exist.
//! - `is_locally_customized` drift detection (no second package version
//!   exists yet to test diffing against).
//! - A full semver range grammar - `version_satisfies` below only
//!   understands `"*"` and `">=X.Y.Z"`.
//! - "Default views" (spec 1.1) - no saved-view concept exists anywhere
//!   in the platform yet, so `ManifestApp` doesn't model one either.

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::app_definition::{AppDefinitionInput, AppDefinitionUpdate};
use crate::models::custom_report::CustomReport;
use crate::models::dashboard_layout::{DashboardLayoutInput, DashboardLayoutUpdate, DashboardWidget, DashboardWidgets};
use crate::models::industry_package::{
    AppInstallRun, AppPackage, ImportPackageInput, IndustryPackageManifest, InstalledApp, InstalledAppDetail,
    ManifestDashboardWidget, INDUSTRY_PACKAGE_FORMAT_VERSION,
};
use crate::models::relationship::RelationshipDefinition;
use crate::models::screen_layout::{LayoutTabs, ScreenLayoutInput, ScreenLayoutUpdate};
use crate::repositories::industry_package_repo;
use crate::services::{
    app_service, business_rule_service, custom_field_service, custom_object_service, custom_record_service,
    custom_report_service, dashboard_layout_service, numbering_service, relationship_service, screen_layout_service,
    workflow_service,
};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

// --- version handling ------------------------------------------------------
//
// No semver crate dependency (see this module's own doc comment) - just
// enough of major.minor.patch comparison to support `min_lanesra_version`
// and a dependency's `version_constraint`. A missing/non-numeric
// component parses as 0, so "1.2" and "1.2.0" compare equal and "bogus"
// is simply (0, 0, 0) - lenient on the input side, since a malformed
// version string should fail the *comparison* it's used in, not panic.

fn parse_version(v: &str) -> (u64, u64, u64) {
    let mut parts = v.trim().split('.').map(|p| p.trim().parse::<u64>().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

/// `constraint` is `"*"` (any version) or `">=X.Y.Z"` - anything else
/// fails closed (returns false) rather than being silently ignored.
fn version_satisfies(version: &str, constraint: &str) -> bool {
    let constraint = constraint.trim();
    if constraint.is_empty() || constraint == "*" {
        return true;
    }
    match constraint.strip_prefix(">=") {
        Some(min) => parse_version(version) >= parse_version(min),
        None => false,
    }
}

/// This build's own version, the same `env!("CARGO_PKG_VERSION")` value
/// `backup_service` already stamps onto every backup's manifest.
fn current_lanesra_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Not cryptographic (no hash crate is a dependency of this crate) -
/// good enough for what `app_packages.checksum` is for per migration
/// 0027's own comment: catching a tampered or corrupted re-import that
/// silently differs from what was already validated, not defending
/// against a deliberate adversary.
fn checksum_of(manifest_json: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    manifest_json.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn parse_manifest(manifest_json: &str) -> AppResult<IndustryPackageManifest> {
    let manifest: IndustryPackageManifest =
        serde_json::from_str(manifest_json).map_err(|e| AppError::Validation(format!("Invalid package manifest: {e}")))?;
    if manifest.format_version != INDUSTRY_PACKAGE_FORMAT_VERSION {
        return Err(AppError::Validation(format!(
            "Unsupported manifest format_version {} (this build understands {})",
            manifest.format_version, INDUSTRY_PACKAGE_FORMAT_VERSION
        )));
    }
    if manifest.package_id.trim().is_empty() {
        return Err(AppError::Validation("Manifest is missing package_id".into()));
    }
    if manifest.name.trim().is_empty() {
        return Err(AppError::Validation("Manifest is missing name".into()));
    }
    if manifest.version.trim().is_empty() {
        return Err(AppError::Validation("Manifest is missing version".into()));
    }
    Ok(manifest)
}

/// Everything checkable before an install is actually attempted - min
/// version and dependencies. Object/field key collisions are
/// deliberately *not* re-checked here: `custom_object_service::create_with_key`
/// / `custom_field_service::create_definition_with_key` already hard-fail
/// on those (see their own doc comments) and `install` runs inside one
/// transaction, so a collision caught mid-install rolls back exactly as
/// cleanly as one caught here would have - this just gives the Review
/// step (spec: "Admin -> App Catalog -> Review -> Validate -> Install") a
/// cheap up-front check for the two things a collision check can't catch
/// anyway (an unmet minimum version, a missing dependency).
pub fn validate(conn: &Connection, workspace_id: &str, manifest: &IndustryPackageManifest) -> AppResult<()> {
    if !version_satisfies(current_lanesra_version(), &format!(">={}", manifest.min_lanesra_version)) {
        return Err(AppError::Validation(format!(
            "'{}' requires Lanesra {} or later (this workspace is running {})",
            manifest.name,
            manifest.min_lanesra_version,
            current_lanesra_version()
        )));
    }
    for dep in &manifest.dependencies {
        let installed = industry_package_repo::get_installed_app_by_package(conn, workspace_id, &dep.package_id)?;
        let satisfied = installed
            .as_ref()
            .map(|app| app.status == "active" && version_satisfies(&app.installed_version, &dep.version_constraint))
            .unwrap_or(false);
        if !satisfied && dep.is_required {
            return Err(AppError::Validation(format!(
                "'{}' requires '{}' {} to be installed and active first",
                manifest.name, dep.package_id, dep.version_constraint
            )));
        }
    }
    Ok(())
}

/// Parses, validates the manifest's own shape, and adds it to this
/// workspace's local catalog (spec 13.6's "review before install") -
/// does not install anything yet.
pub fn import_package(conn: &Connection, workspace_id: &str, input: &ImportPackageInput, actor_user_id: Option<&str>) -> AppResult<AppPackage> {
    require_admin(conn, actor_user_id)?;
    let manifest = parse_manifest(&input.manifest_json)?;
    if industry_package_repo::get_package_by_version(conn, workspace_id, &manifest.package_id, &manifest.version)?.is_some() {
        return Err(AppError::Conflict(format!(
            "'{}' version {} has already been imported into this workspace",
            manifest.package_id, manifest.version
        )));
    }
    let id = new_uuid();
    let checksum = checksum_of(&input.manifest_json);
    Ok(industry_package_repo::insert_package(
        conn,
        &id,
        workspace_id,
        &manifest.package_id,
        &manifest.name,
        &manifest.industry,
        &manifest.version,
        &manifest.min_lanesra_version,
        &input.manifest_json,
        &checksum,
        "import",
        actor_user_id,
    )?)
}

pub fn list_packages(conn: &Connection, workspace_id: &str) -> AppResult<Vec<AppPackage>> {
    Ok(industry_package_repo::list_packages(conn, workspace_id)?)
}

/// Bundled starter manifests (`services::reference_packages`) an admin
/// can load into the Review step's textarea before importing - not a
/// silent one-click install, so the same Review -> Validate -> Install
/// flow a hand-authored manifest goes through still applies. `key` is
/// the reference package's own short name, not its `package_id`.
pub fn reference_package_manifest(key: &str) -> AppResult<String> {
    match key {
        "field_service" => Ok(super::reference_packages::field_service_manifest_json()),
        "property_management" => Ok(super::reference_packages::property_management_manifest_json()),
        "construction" => Ok(super::reference_packages::construction_manifest_json()),
        "professional_services" => Ok(super::reference_packages::professional_services_manifest_json()),
        "practice_admin" => Ok(super::reference_packages::practice_admin_manifest_json()),
        "recruitment" => Ok(super::reference_packages::recruitment_manifest_json()),
        "real_estate" => Ok(super::reference_packages::real_estate_manifest_json()),
        "legal_practice" => Ok(super::reference_packages::legal_practice_manifest_json()),
        other => Err(AppError::NotFound(format!("Reference package '{other}'"))),
    }
}

/// Re-parses and re-validates an already-imported package's manifest -
/// what the Admin -> App Catalog screen's "Validate" step calls before
/// offering "Install".
pub fn validate_package(conn: &Connection, workspace_id: &str, app_package_id: &str) -> AppResult<()> {
    let package = get_owned_package(conn, workspace_id, app_package_id)?;
    let manifest = parse_manifest(&package.manifest_json)?;
    validate(conn, workspace_id, &manifest)
}

fn get_owned_package(conn: &Connection, workspace_id: &str, app_package_id: &str) -> AppResult<AppPackage> {
    let package = industry_package_repo::get_package(conn, app_package_id)?.ok_or_else(|| AppError::NotFound("Package".into()))?;
    if package.workspace_id != workspace_id {
        return Err(AppError::NotFound("Package".into()));
    }
    Ok(package)
}

/// Takes a whole-workspace `.lanesra` safety snapshot and writes it to a
/// local temp file, returning that path - belt-and-suspenders alongside
/// the install transaction's own atomic rollback (see migration 0027's
/// `app_install_runs.backup_snapshot_path` comment): it lets an
/// administrator restore to this exact pre-install point even after a
/// run that committed successfully but is later found undesirable.
/// `backup_service::create_backup` itself only ever returns the package
/// in memory (base64, meant for a client download) - this is the one
/// place in the app that also persists a copy server-side.
fn take_safety_backup(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<String> {
    let package = super::backup_service::create_backup(conn, actor_user_id)?;
    let bytes = BASE64
        .decode(&package.package_base64)
        .map_err(|e| AppError::Validation(format!("could not decode safety backup: {e}")))?;
    let path = std::env::temp_dir().join(format!("lanesra-install-backup-{}.lanesra", new_uuid()));
    std::fs::write(&path, bytes).map_err(|e| AppError::Validation(format!("could not write safety backup: {e}")))?;
    Ok(path.to_string_lossy().into_owned())
}

/// Screen/App Builder Phase 3's `LayoutTab.related` holds relationship
/// *keys* - but a `RelationshipDefinition.key` is auto-derived at install
/// time, not something a package author controls (see
/// `ManifestScreenLayout`'s own doc comment). So a manifest's `related`
/// entries are the relationship's 0-based index into
/// `IndustryPackageManifest::relationships`, written as a decimal string
/// (e.g. `["0", "2"]`) - resolved here, once, into the real keys the
/// screen layout actually needs.
fn resolve_related_indices(draft: &mut LayoutTabs, relationships: &[RelationshipDefinition]) -> AppResult<()> {
    for tab in &mut draft.tabs {
        let mut resolved = Vec::with_capacity(tab.related.len());
        for raw in &tab.related {
            let idx: usize = raw.parse().map_err(|_| {
                AppError::Validation(format!(
                    "Screen layout tab '{}' has an invalid relationship reference '{raw}' (expected a 0-based index into the manifest's relationships list)",
                    tab.title
                ))
            })?;
            let rel = relationships.get(idx).ok_or_else(|| {
                AppError::Validation(format!(
                    "Screen layout tab '{}' references relationship index {idx}, but the manifest only defines {} relationship(s)",
                    tab.title,
                    relationships.len()
                ))
            })?;
            resolved.push(rel.key.clone());
        }
        tab.related = resolved;
    }
    Ok(())
}

/// Same index-reference reasoning as `resolve_related_indices`, for a
/// `"chart"` dashboard widget's `report_ref` (see `ManifestDashboard`'s
/// own doc comment) - rewritten into the real `report_id` a
/// `DashboardWidget` needs.
fn resolve_dashboard_widgets(widgets: &[ManifestDashboardWidget], reports: &[CustomReport]) -> AppResult<Vec<DashboardWidget>> {
    widgets
        .iter()
        .map(|w| {
            let mut config = w.config.clone();
            if w.kind == "chart" {
                let idx = config.get("report_ref").and_then(|v| v.as_u64()).ok_or_else(|| {
                    AppError::Validation("A 'chart' dashboard widget needs a numeric 'report_ref' index into the manifest's reports list".into())
                })? as usize;
                let report = reports.get(idx).ok_or_else(|| {
                    AppError::Validation(format!(
                        "Dashboard widget references report index {idx}, but the manifest only defines {} report(s)",
                        reports.len()
                    ))
                })?;
                if let Some(obj) = config.as_object_mut() {
                    obj.remove("report_ref");
                    obj.insert("report_id".to_string(), serde_json::Value::String(report.id.clone()));
                }
            }
            Ok(DashboardWidget { id: new_uuid(), kind: w.kind.clone(), config })
        })
        .collect()
}

/// Same index-reference reasoning again, for the two workflow action
/// types that embed a relationship id in their opaque `params_json`
/// (`create_record`'s optional link, `update_related_record`'s target) -
/// see those actions' own param structs in `workflow_service`. A
/// manifest author writes `"relationship_ref": <index>` in place of
/// `"relationship_definition_id"`; this rewrites it to the real id
/// before the action is ever validated, the same substitution
/// `resolve_related_indices`/`resolve_dashboard_widgets` do for their
/// own opaque references. Any other action type's `params_json` passes
/// through untouched.
fn resolve_workflow_action_relationship_refs(
    workflow: &crate::models::workflow::WorkflowDefinitionInput,
    relationships: &[RelationshipDefinition],
) -> AppResult<crate::models::workflow::WorkflowDefinitionInput> {
    let mut resolved = workflow.clone();
    for action in &mut resolved.actions {
        if action.action_type != "create_record" && action.action_type != "update_related_record" {
            continue;
        }
        let mut params: serde_json::Value = serde_json::from_str(&action.params_json)
            .map_err(|e| AppError::Validation(format!("Invalid parameters for '{}': {e}", action.action_type)))?;
        let idx = match params.get("relationship_ref").and_then(|v| v.as_u64()) {
            Some(idx) => idx as usize,
            None => continue, // create_record's link is optional - no ref, nothing to resolve
        };
        let rel = relationships.get(idx).ok_or_else(|| {
            AppError::Validation(format!(
                "Workflow '{}' action '{}' references relationship index {idx}, but the manifest only defines {} relationship(s)",
                workflow.name,
                action.action_type,
                relationships.len()
            ))
        })?;
        if let Some(obj) = params.as_object_mut() {
            obj.remove("relationship_ref");
            obj.insert("relationship_definition_id".to_string(), serde_json::Value::String(rel.id.clone()));
        }
        action.params_json = params.to_string();
    }
    Ok(resolved)
}

/// The actual work of an install, run inside `install`'s transaction -
/// every sub-service call below takes `&Connection`, which a
/// `rusqlite::Transaction` derefs to, so a failure anywhere (a `?` on any
/// line) unwinds back to `install`, which drops the transaction without
/// committing - nothing this function did is left half-applied. Order
/// matters: later steps (screen layouts, dashboard) reference relationships
/// and reports created earlier by array index (see
/// `resolve_related_indices`/`resolve_dashboard_widgets`), and the
/// `installed_apps` row itself is only created once its
/// `app_definition_id` is known, at the very end - see the inline
/// comments below for why `package_artifacts` rows are buffered in memory
/// until then instead of inserted as each item is created.
fn run_install(conn: &Connection, workspace_id: &str, manifest: &IndustryPackageManifest, actor_user_id: Option<&str>) -> AppResult<InstalledApp> {
    if industry_package_repo::get_installed_app_by_package(conn, workspace_id, &manifest.package_id)?.is_some() {
        return Err(AppError::Conflict(format!("'{}' is already installed in this workspace", manifest.package_id)));
    }
    validate(conn, workspace_id, manifest)?;

    // (artifact_type, metadata_id) - not inserted until the installed_apps
    // row exists (package_artifacts.installed_app_id is NOT NULL), so
    // buffered here and flushed in one pass near the end.
    let mut artifacts: Vec<(&'static str, String)> = Vec::new();

    for obj in &manifest.objects {
        let created = custom_object_service::create_with_key(conn, workspace_id, &obj.key, &obj.definition, actor_user_id)?;
        artifacts.push(("custom_object", created.id));
    }

    for field in &manifest.fields {
        let created = custom_field_service::create_definition_with_key(conn, workspace_id, &field.key, &field.definition, actor_user_id)?;
        artifacts.push(("custom_field", created.id));
    }

    // Kept around (not just their ids) so screen layouts below can
    // resolve a `related` index into a real relationship key.
    let mut created_relationships: Vec<RelationshipDefinition> = Vec::new();
    for rel_input in &manifest.relationships {
        let created = relationship_service::create(conn, workspace_id, rel_input, actor_user_id)?;
        artifacts.push(("relationship_definition", created.id.clone()));
        created_relationships.push(created);
    }

    for rule_input in &manifest.business_rules {
        let created = business_rule_service::create_rule(conn, workspace_id, rule_input, actor_user_id)?;
        artifacts.push(("business_rule", created.id));
    }

    for workflow_input in &manifest.workflows {
        let resolved_input = resolve_workflow_action_relationship_refs(workflow_input, &created_relationships)?;
        let created = workflow_service::create_rule(conn, workspace_id, &resolved_input, actor_user_id)?;
        artifacts.push(("workflow_definition", created.id));
    }

    for layout in &manifest.screen_layouts {
        let mut draft = layout.draft.clone();
        resolve_related_indices(&mut draft, &created_relationships)?;
        let created = screen_layout_service::create_layout(
            conn,
            workspace_id,
            &ScreenLayoutInput { entity_type: layout.entity_type.clone(), name: layout.name.clone(), initial_fields: vec![] },
            actor_user_id,
        )?;
        let created = screen_layout_service::update_layout(
            conn,
            &created.id,
            &ScreenLayoutUpdate { name: layout.name.clone(), roles: vec![], draft },
            actor_user_id,
        )?;
        if layout.publish {
            screen_layout_service::publish_layout(conn, &created.id, actor_user_id)?;
        }
        artifacts.push(("screen_layout", created.id));
    }

    // Created before the dashboard, since a "chart" widget resolves its
    // report_ref against this list.
    let mut created_reports: Vec<CustomReport> = Vec::new();
    for report_input in &manifest.reports {
        let created = custom_report_service::create(conn, workspace_id, report_input, actor_user_id)?;
        artifacts.push(("custom_report", created.id.clone()));
        created_reports.push(created);
    }

    let mut dashboard_layout_id: Option<String> = None;
    if let Some(dashboard) = &manifest.dashboard {
        let widgets = resolve_dashboard_widgets(&dashboard.widgets, &created_reports)?;
        let created = dashboard_layout_service::create_layout(
            conn,
            workspace_id,
            &DashboardLayoutInput { name: dashboard.name.clone(), initial_kpi_keys: vec![], app_id: None },
            actor_user_id,
        )?;
        let created = dashboard_layout_service::update_layout(
            conn,
            &created.id,
            &DashboardLayoutUpdate { name: dashboard.name.clone(), roles: vec![], draft: DashboardWidgets { widgets }, app_id: None },
            actor_user_id,
        )?;
        if dashboard.publish {
            dashboard_layout_service::publish_layout(conn, &created.id, actor_user_id)?;
        }
        artifacts.push(("dashboard_layout", created.id.clone()));
        dashboard_layout_id = Some(created.id);
    }

    for override_input in &manifest.numbering_overrides {
        numbering_service::set_override(conn, workspace_id, override_input, actor_user_id)?;
        // NumberingOverride has no id of its own (see migration 0026) -
        // its entity_type is the table's real key, same convention
        // migration 0027's own comment on package_artifacts documents.
        artifacts.push(("numbering_override", override_input.entity_type.clone()));
    }

    // App Builder grouping (spec 13.4's "app-level UX") - reused as-is
    // rather than building a parallel nav/permission system. Not itself
    // tracked as a package_artifact row: it already has a first-class
    // home on installed_apps.app_definition_id.
    let mut app_definition_id: Option<String> = None;
    let mut recommended_permissions_json = "[]".to_string();
    let (app_name, app_icon, app_description) = match &manifest.app {
        Some(app) => {
            let created = app_service::create(
                conn,
                workspace_id,
                &AppDefinitionInput { name: app.name.clone(), icon: app.icon.clone(), description: app.description.clone() },
                actor_user_id,
            )?;
            let dashboard_id = if app.use_package_dashboard { dashboard_layout_id.clone() } else { None };
            let created = app_service::update(
                conn,
                &created.id,
                &AppDefinitionUpdate {
                    name: app.name.clone(),
                    icon: app.icon.clone(),
                    description: app.description.clone(),
                    object_keys: app.object_keys.clone(),
                    dashboard_id,
                },
                actor_user_id,
            )?;
            if app.publish {
                app_service::publish(conn, &created.id, actor_user_id)?;
            }
            app_definition_id = Some(created.id);
            recommended_permissions_json = serde_json::to_string(&app.recommended_permissions).unwrap_or_else(|_| "[]".into());
            (app.name.clone(), app.icon.clone(), app.description.clone())
        }
        None => (manifest.name.clone(), "\u{2b21}".to_string(), None),
    };

    let installed_app_id = new_uuid();
    let installed = industry_package_repo::create_installed_app(
        conn,
        &installed_app_id,
        workspace_id,
        &manifest.package_id,
        &app_name,
        &app_icon,
        &manifest.industry,
        app_description.as_deref(),
        &manifest.version,
        app_definition_id.as_deref(),
        &recommended_permissions_json,
        actor_user_id,
    )?;

    for (artifact_type, metadata_id) in &artifacts {
        industry_package_repo::insert_artifact(conn, &new_uuid(), &installed_app_id, artifact_type, metadata_id, &manifest.version)?;
    }

    // Seed data (spec 1.1: "reference data only") created last, since a
    // sample record can carry values for this package's own custom
    // fields, all of which now exist.
    for sample in &manifest.seed_data {
        let record = custom_record_service::create(conn, workspace_id, &sample.record, actor_user_id)?;
        if !sample.field_values.is_empty() {
            custom_field_service::set_entity_values(conn, &sample.object_key, &record.id, &sample.field_values, actor_user_id)?;
        }
        industry_package_repo::insert_artifact(conn, &new_uuid(), &installed_app_id, "custom_record", &record.id, &manifest.version)?;
    }

    Ok(installed)
}

/// Installs an already-imported package into this workspace: a safety
/// backup, then one atomic transaction covering every sub-system
/// `run_install` touches (`Connection::unchecked_transaction` rather than
/// `Connection::transaction` so this can take the same `&Connection`
/// every other service function does, whether that's a Tauri command's
/// `MutexGuard<Connection>` or the server dispatcher's pooled
/// `&Connection` - see this crate's other services for that convention).
/// `app_install_runs` records the attempt either way, success or failure,
/// so a failed install still leaves a readable "why it failed" record.
pub fn install(conn: &Connection, workspace_id: &str, app_package_id: &str, actor_user_id: Option<&str>) -> AppResult<InstalledApp> {
    require_admin(conn, actor_user_id)?;
    let package = get_owned_package(conn, workspace_id, app_package_id)?;
    let manifest = parse_manifest(&package.manifest_json)?;

    let run_id = new_uuid();
    industry_package_repo::start_run(conn, &run_id, workspace_id, &manifest.package_id, &manifest.version, "install", actor_user_id)?;

    // `validate` runs *inside* the recorded attempt (not before start_run)
    // so a rejected min-version or unmet dependency leaves the same
    // readable "why it failed" run row a mid-transaction failure would -
    // the only failures with no run row at all are ones that couldn't even
    // identify which package/version was being attempted (bad admin auth,
    // an unknown app_package_id, a corrupt stored manifest).
    let outcome: AppResult<(InstalledApp, String)> = (|| {
        validate(conn, workspace_id, &manifest)?;
        let backup_path = take_safety_backup(conn, actor_user_id)?;
        let tx = conn.unchecked_transaction()?;
        let installed = run_install(&tx, workspace_id, &manifest, actor_user_id)?;
        tx.commit()?;
        Ok((installed, backup_path))
    })();

    match outcome {
        Ok((installed, backup_path)) => {
            industry_package_repo::complete_run(conn, &run_id, "succeeded", None, Some(&backup_path))?;
            Ok(installed)
        }
        Err(e) => {
            industry_package_repo::complete_run(conn, &run_id, "failed", Some(&e.to_string()), None)?;
            Err(e)
        }
    }
}

pub fn list_installed(conn: &Connection, workspace_id: &str) -> AppResult<Vec<InstalledApp>> {
    Ok(industry_package_repo::list_installed_apps(conn, workspace_id)?)
}

pub fn get_installed_detail(conn: &Connection, id: &str) -> AppResult<InstalledAppDetail> {
    let app = industry_package_repo::get_installed_app(conn, id)?.ok_or_else(|| AppError::NotFound("Installed app".into()))?;
    let artifacts = industry_package_repo::list_artifacts(conn, id)?;
    Ok(InstalledAppDetail { app, artifacts })
}

pub fn list_runs(conn: &Connection, workspace_id: &str) -> AppResult<Vec<AppInstallRun>> {
    Ok(industry_package_repo::list_runs(conn, workspace_id)?)
}

/// Non-destructive: hides the app from navigation (unpublishing its
/// linked `AppDefinition`, if any - App Builder's own nav filtering keys
/// off `is_published`, not `installed_apps.status`, so this is needed for
/// deactivating to actually have that effect) without touching any
/// business record or metadata the install created - spec's stated
/// default removal behavior. `reactivate` is the exact mirror.
pub fn deactivate(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<InstalledApp> {
    require_admin(conn, actor_user_id)?;
    let app = industry_package_repo::get_installed_app(conn, id)?.ok_or_else(|| AppError::NotFound("Installed app".into()))?;
    let run_id = new_uuid();
    industry_package_repo::start_run(conn, &run_id, &app.workspace_id, &app.package_id, &app.installed_version, "deactivate", actor_user_id)?;
    let outcome: AppResult<()> = (|| {
        industry_package_repo::set_status(conn, id, false, actor_user_id)?;
        if let Some(app_definition_id) = &app.app_definition_id {
            app_service::unpublish(conn, app_definition_id, actor_user_id)?;
        }
        Ok(())
    })();
    match &outcome {
        Ok(()) => industry_package_repo::complete_run(conn, &run_id, "succeeded", None, None)?,
        Err(e) => industry_package_repo::complete_run(conn, &run_id, "failed", Some(&e.to_string()), None)?,
    }
    outcome?;
    industry_package_repo::get_installed_app(conn, id)?.ok_or_else(|| AppError::NotFound("Installed app".into()))
}

pub fn reactivate(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<InstalledApp> {
    require_admin(conn, actor_user_id)?;
    let app = industry_package_repo::get_installed_app(conn, id)?.ok_or_else(|| AppError::NotFound("Installed app".into()))?;
    let run_id = new_uuid();
    industry_package_repo::start_run(conn, &run_id, &app.workspace_id, &app.package_id, &app.installed_version, "reactivate", actor_user_id)?;
    let outcome: AppResult<()> = (|| {
        industry_package_repo::set_status(conn, id, true, actor_user_id)?;
        if let Some(app_definition_id) = &app.app_definition_id {
            app_service::publish(conn, app_definition_id, actor_user_id)?;
        }
        Ok(())
    })();
    match &outcome {
        Ok(()) => industry_package_repo::complete_run(conn, &run_id, "succeeded", None, None)?,
        Err(e) => industry_package_repo::complete_run(conn, &run_id, "failed", Some(&e.to_string()), None)?,
    }
    outcome?;
    industry_package_repo::get_installed_app(conn, id)?.ok_or_else(|| AppError::NotFound("Installed app".into()))
}
