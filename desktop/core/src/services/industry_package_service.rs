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
    ManifestDashboardWidget, WorkspaceArtifact, WorkspaceDependency, INDUSTRY_PACKAGE_FORMAT_VERSION,
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
    // Every reference package ships namespaced as "lanesra.<name>", and
    // ensure_defaults (idempotent) guarantees that publisher exists, so
    // this only ever rejects a hand-authored manifest under an
    // unregistered namespace - never a bundled starter.
    super::publisher_service::ensure_defaults(conn, workspace_id)?;
    let publisher = super::publisher_service::resolve_for_package_id(conn, workspace_id, &manifest.package_id)?;

    let id = new_uuid();
    let checksum = checksum_of(&input.manifest_json);
    let package = industry_package_repo::insert_package(
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
        &publisher.id,
        true,
        actor_user_id,
    )?;
    // Recorded now, not deferred to install: a package's declared
    // dependencies are part of what the spec's Review step shows before
    // Install is ever offered, and a Solution Management "Dependencies"
    // view should be able to show every imported package's requirements,
    // not only installed ones.
    for dep in &manifest.dependencies {
        industry_package_repo::insert_dependency(conn, &new_uuid(), &package.id, &dep.package_id, &dep.version_constraint, dep.is_required)?;
    }
    Ok(package)
}

pub fn list_packages(conn: &Connection, workspace_id: &str) -> AppResult<Vec<AppPackage>> {
    Ok(industry_package_repo::list_packages(conn, workspace_id)?)
}

/// Every imported version of one `package_id`, oldest first - the
/// Solution Management "Releases" view for a package. See
/// `industry_package_repo::list_versions_for_package`'s own comment for
/// why this needed no new table: each `app_packages` row already is an
/// immutable version snapshot.
pub fn list_package_versions(conn: &Connection, workspace_id: &str, package_id: &str) -> AppResult<Vec<AppPackage>> {
    Ok(industry_package_repo::list_versions_for_package(conn, workspace_id, package_id)?)
}

/// Every dependency declared by every package imported into this
/// workspace, with `is_satisfied` computed the same way `validate` checks
/// it before an install: an active install of `dependency_package_id`
/// whose version meets `version_constraint`. Read-only - this never
/// blocks anything, it's what the Solution Management "Dependencies" tab
/// shows.
pub fn list_dependencies_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<WorkspaceDependency>> {
    let rows = industry_package_repo::list_dependencies_for_workspace(conn, workspace_id)?;
    rows.into_iter()
        .map(|(dependency, package_id, package_name, package_version)| {
            let installed = industry_package_repo::get_installed_app_by_package(conn, workspace_id, &dependency.dependency_package_id)?;
            let is_satisfied = installed
                .as_ref()
                .map(|app| app.status == "active" && version_satisfies(&app.installed_version, &dependency.version_constraint))
                .unwrap_or(false);
            Ok(WorkspaceDependency { dependency, package_id, package_name, package_version, is_satisfied })
        })
        .collect()
}

/// Every artifact created by every app installed in this workspace,
/// across every installed app - the workspace-wide counterpart to
/// `get_installed_detail`'s per-app artifact list, and the "what have I
/// customized beyond what I installed" view the Solution Management
/// "Components" tab exists to answer.
pub fn list_artifacts_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<WorkspaceArtifact>> {
    let rows = industry_package_repo::list_artifacts_for_workspace(conn, workspace_id)?;
    Ok(rows
        .into_iter()
        .map(|(artifact, installed_app_name, package_id)| WorkspaceArtifact { artifact, installed_app_name, package_id })
        .collect())
}

/// The Managed/Unmanaged distinction's other deliverable: a real,
/// re-importable `.lanesra`-style manifest built from everything this
/// workspace's `local` publisher currently owns - the reverse of
/// `run_install`, reusing the exact same `IndustryPackageManifest` shape
/// rather than a separate export format. Deliberately narrower than a
/// full package in three ways, each documented rather than silently
/// worked around:
///   - No `dashboard`, no `app` grouping, no `seed_data` - a workspace's
///     hand-built dashboards/App Builder apps aren't uniquely determined
///     (there can be several, or none, and the manifest format only
///     supports one dashboard), and seed_data has no home in
///     component-tagging at all (migration 0030 excludes `custom_record`
///     from tagging the same way it excludes `numbering_override`).
///   - A workflow action's `create_record`/`update_related_record` link to
///     a relationship this export doesn't itself own (e.g. one created by
///     an installed package, not by `local`) is dropped - embedding that
///     relationship's real, source-workspace-only id into the exported
///     JSON would silently reference nothing (or the wrong thing) once
///     re-imported elsewhere.
///   - `min_lanesra_version` is stamped as this build's own version
///     (the export is only ever validated against the workspace that just
///     produced it, or a workspace running the same or newer build).
/// `version` is a synthetic `1.0.<unix timestamp>` so exporting the same
/// workspace twice produces two importable versions rather than colliding
/// on `(package_id, version)` - `package_id` is fixed as
/// `"local.workspace_export"`, deliberately under the `local` publisher's
/// own namespace so importing it back (into this workspace or any other
/// that already auto-seeded `local`, i.e. every workspace) never needs a
/// publisher registered first.
pub fn export_local_workspace(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<String> {
    require_admin(conn, actor_user_id)?;
    let components = super::solution_component_service::list_local(conn, workspace_id)?;
    let refs: Vec<(String, String)> = components.into_iter().map(|c| (c.artifact_type, c.metadata_id)).collect();
    let version = format!("1.0.{}", chrono::Utc::now().timestamp());
    let manifest = build_export_manifest(conn, &refs, "local.workspace_export".to_string(), "Local Workspace Export".to_string(), version)?;
    serde_json::to_string_pretty(&manifest).map_err(|e| AppError::Validation(format!("could not serialize export: {e}")))
}

/// A named, scoped export - the Dynamics-365-style "build a solution in
/// test, export it" half of the workflow `solution_service` exists for.
/// Reads a Solution's own curated membership (a deliberate, admin-picked
/// subset - not "everything `local` owns", see `export_local_workspace`'s
/// own doc comment for that distinction) and reuses the exact same
/// manifest-building pass this module already uses for the whole-workspace
/// export, so a Solution's export is importable through the *unmodified*
/// existing pipeline into a second, separate workspace - "prod" - via the
/// ordinary Admin → App Catalog → Import → Validate → Install flow. No new
/// import/install machinery exists for this; the scoping happens entirely
/// on the export side.
///
/// `package_id` is fixed as `"local.solution.<solution id>"` - always
/// under the reserved `local` namespace (see `export_local_workspace`'s
/// own doc comment for why that namespace needs no publisher registered
/// on the importing side) and stable across every export of the same
/// Solution, so repeated exports become successive versions of one
/// importable package - listable via the existing `list_package_versions`
/// / Releases view, and upgradeable in prod via the existing
/// `plan_update`/`apply_update` pair, exactly like any other package.
/// `name`/`version` come from the Solution's own admin-set fields, not a
/// synthetic timestamp like `export_local_workspace` uses - the version is
/// the number an admin deliberately bumps each time they hand a new
/// snapshot to prod.
pub fn export_solution(conn: &Connection, workspace_id: &str, solution_id: &str, actor_user_id: Option<&str>) -> AppResult<String> {
    require_admin(conn, actor_user_id)?;
    let solution = super::solution_service::get(conn, workspace_id, solution_id)?;
    let members = super::solution_service::list_member_refs(conn, solution_id)?;
    let refs: Vec<(String, String)> = members.into_iter().map(|m| (m.artifact_type, m.metadata_id)).collect();
    let manifest = build_export_manifest(conn, &refs, format!("local.solution.{}", solution.id), solution.name.clone(), solution.version.clone())?;
    serde_json::to_string_pretty(&manifest).map_err(|e| AppError::Validation(format!("could not serialize export: {e}")))
}

/// Shared by `export_local_workspace` and `export_solution` - everything
/// about turning a list of `(artifact_type, metadata_id)` component
/// references into an `IndustryPackageManifest` is identical between the
/// two; only which components go in, and what `package_id`/`name`/
/// `version` get stamped on the result, differ. See
/// `export_local_workspace`'s own doc comment for the three deliberate
/// scope limits every caller of this function inherits (no dashboard/app/
/// seed_data; a relationship reference this export doesn't own is
/// dropped; `min_lanesra_version` is stamped as this build's own version).
fn build_export_manifest(
    conn: &Connection,
    components: &[(String, String)],
    package_id: String,
    name: String,
    version: String,
) -> AppResult<IndustryPackageManifest> {
    let mut objects = Vec::new();
    let mut fields = Vec::new();
    let mut relationships: Vec<crate::models::relationship::RelationshipDefinitionInput> = Vec::new();
    let mut relationship_key_index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut relationship_id_index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut business_rules = Vec::new();
    let mut raw_workflows: Vec<crate::models::workflow::WorkflowDefinitionInput> = Vec::new();
    let mut screen_layouts = Vec::new();
    let mut reports = Vec::new();

    // Pass 1: everything relationships might need to reference (objects,
    // fields don't reference relationships; relationships themselves are
    // gathered here so their real id/key can be mapped to an export-local
    // index before screen layouts / workflow actions - which store real
    // ids/keys today, not indices - are rewritten below.
    for (artifact_type, metadata_id) in components {
        match artifact_type.as_str() {
            "custom_object" => {
                if let Some(def) = super::custom_object_service::get(conn, metadata_id)? {
                    objects.push(crate::models::industry_package::ManifestObject {
                        key: def.key,
                        definition: crate::models::custom_object::CustomObjectDefinitionInput {
                            singular_label: def.singular_label,
                            plural_label: def.plural_label,
                            icon: def.icon,
                            prefix: def.prefix,
                            digits: def.digits,
                        },
                    });
                }
            }
            "custom_field" => {
                if let Some(def) = super::custom_field_service::get_definition(conn, metadata_id)? {
                    fields.push(crate::models::industry_package::ManifestField {
                        key: def.key,
                        definition: crate::models::custom_field::CustomFieldDefinitionInput {
                            entity_type: def.entity_type,
                            label: def.label,
                            field_type: def.field_type,
                            options: def.options,
                            required: def.required,
                            show_in_list: def.show_in_list,
                            sort_order: def.sort_order,
                            min_value: def.min_value,
                            max_value: def.max_value,
                            max_length: def.max_length,
                            regex_pattern: def.regex_pattern,
                            is_searchable: def.is_searchable,
                            is_filterable: def.is_filterable,
                            is_reportable: def.is_reportable,
                            default_value: def.default_value,
                            is_unique: def.is_unique,
                            help_text: def.help_text,
                            placeholder: def.placeholder,
                            is_hidden_by_default: def.is_hidden_by_default,
                        },
                    });
                }
            }
            "relationship_definition" => {
                if let Some(def) = super::relationship_service::get(conn, metadata_id)? {
                    let idx = relationships.len();
                    relationship_key_index.insert(def.key.clone(), idx);
                    relationship_id_index.insert(def.id.clone(), idx);
                    relationships.push(crate::models::relationship::RelationshipDefinitionInput {
                        source_entity_type: def.source_entity_type,
                        target_entity_type: def.target_entity_type,
                        relationship_type: def.relationship_type,
                        forward_label: def.forward_label,
                        reverse_label: def.reverse_label,
                        is_required: def.is_required,
                        show_related_list: def.show_related_list,
                        delete_behavior: def.delete_behavior,
                        sort_order: def.sort_order,
                    });
                }
            }
            "business_rule" => {
                if let Some(rule) = super::business_rule_service::get_rule(conn, metadata_id)? {
                    business_rules.push(crate::models::business_rule::BusinessRuleInput {
                        entity_type: rule.entity_type,
                        name: rule.name,
                        description: rule.description,
                        match_type: rule.match_type,
                        priority: rule.priority,
                        effective_start_date: rule.effective_start_date,
                        effective_end_date: rule.effective_end_date,
                        // App Builder apps are workspace-local groupings,
                        // not portable customization content - dropped on
                        // export the same way dashboard/app themselves are
                        // (see this function's own doc comment).
                        app_id: None,
                        conditions: rule
                            .conditions
                            .into_iter()
                            .map(|c| crate::models::business_rule::BusinessRuleConditionInput {
                                field_source: c.field_source,
                                field_key: c.field_key,
                                operator: c.operator,
                                value: c.value,
                                compare_field_source: c.compare_field_source,
                                compare_field_key: c.compare_field_key,
                                group_id: c.group_id,
                            })
                            .collect(),
                        actions: rule
                            .actions
                            .into_iter()
                            .map(|a| crate::models::business_rule::BusinessRuleActionInput {
                                action_type: a.action_type,
                                target_field_key: a.target_field_key,
                                target_field_source: a.target_field_source,
                                action_value: a.action_value,
                                message: a.message,
                            })
                            .collect(),
                    });
                }
            }
            "workflow_definition" => {
                if let Some(wf) = super::workflow_service::get_rule(conn, metadata_id)? {
                    raw_workflows.push(crate::models::workflow::WorkflowDefinitionInput {
                        entity_type: wf.entity_type,
                        name: wf.name,
                        description: wf.description,
                        trigger_type: wf.trigger_type,
                        trigger_status: wf.trigger_status,
                        trigger_field_key: wf.trigger_field_key,
                        trigger_field_source: wf.trigger_field_source,
                        trigger_offset_days: wf.trigger_offset_days,
                        match_type: wf.match_type,
                        priority: wf.priority,
                        app_id: None,
                        conditions: wf
                            .conditions
                            .into_iter()
                            .map(|c| crate::models::workflow::WorkflowConditionInput {
                                field_source: c.field_source,
                                field_key: c.field_key,
                                operator: c.operator,
                                value: c.value,
                                compare_field_source: c.compare_field_source,
                                compare_field_key: c.compare_field_key,
                                group_id: c.group_id,
                            })
                            .collect(),
                        actions: wf
                            .actions
                            .into_iter()
                            .map(|a| crate::models::workflow::WorkflowActionInput { action_type: a.action_type, params_json: a.params_json })
                            .collect(),
                    });
                }
            }
            "screen_layout" => {
                if let Ok(layout) = super::screen_layout_service::get_layout(conn, metadata_id) {
                    let draft = layout.published.unwrap_or(layout.draft);
                    screen_layouts.push(crate::models::industry_package::ManifestScreenLayout {
                        entity_type: layout.entity_type,
                        name: layout.name,
                        draft,
                        publish: true,
                    });
                }
            }
            "custom_report" => {
                if let Some(report) = super::custom_report_service::get(conn, metadata_id)? {
                    reports.push(crate::models::custom_report::CustomReportInput {
                        name: report.name,
                        entity_type: report.entity_type,
                        group_by_source: report.group_by_source,
                        group_by_field: report.group_by_field,
                        aggregate: report.aggregate,
                        sum_field_key: report.sum_field_key,
                    });
                }
            }
            // dashboard_layout: intentionally not exported - see this
            // function's own doc comment. Still visible in the Components
            // tab (solution_component_service::list_for_workspace), just
            // not part of the exported manifest.
            _ => {}
        }
    }

    // Pass 2: rewrite the real relationship keys/ids screen layouts and
    // workflow actions were saved with into export-local indices, the
    // same substitution `resolve_related_indices`/
    // `resolve_workflow_action_relationship_refs` undo on import.
    for layout in &mut screen_layouts {
        for tab in &mut layout.draft.tabs {
            tab.related = tab.related.iter().filter_map(|key| relationship_key_index.get(key).map(|i| i.to_string())).collect();
        }
    }
    let mut workflows = Vec::with_capacity(raw_workflows.len());
    for mut wf in raw_workflows {
        let mut keep = true;
        for action in &mut wf.actions {
            if action.action_type != "create_record" && action.action_type != "update_related_record" {
                continue;
            }
            let Ok(mut params) = serde_json::from_str::<serde_json::Value>(&action.params_json) else { continue };
            let real_id = params.get("relationship_definition_id").and_then(|v| v.as_str()).map(str::to_string);
            match real_id.and_then(|id| relationship_id_index.get(&id).copied()) {
                Some(idx) => {
                    if let Some(obj) = params.as_object_mut() {
                        obj.remove("relationship_definition_id");
                        obj.insert("relationship_ref".to_string(), serde_json::Value::from(idx));
                    }
                    action.params_json = params.to_string();
                }
                None if params.get("relationship_definition_id").is_some() => {
                    // Points at a relationship this export doesn't own
                    // (see this function's own doc comment). create_record's
                    // link is optional - safe to just drop it and keep the
                    // rest of the action; update_related_record's target is
                    // NOT optional, so the whole action (and, since it's the
                    // one thing this workflow does, the whole workflow if it
                    // was its only action) is skipped instead of exporting
                    // something that would silently do nothing on import.
                    if action.action_type == "update_related_record" {
                        keep = false;
                    } else if let Some(obj) = params.as_object_mut() {
                        obj.remove("relationship_definition_id");
                        action.params_json = params.to_string();
                    }
                }
                None => {}
            }
        }
        if keep {
            workflows.push(wf);
        }
    }

    Ok(IndustryPackageManifest {
        format_version: INDUSTRY_PACKAGE_FORMAT_VERSION,
        package_id,
        name,
        industry: "Custom".to_string(),
        version,
        min_lanesra_version: current_lanesra_version().to_string(),
        dependencies: Vec::new(),
        objects,
        fields,
        relationships,
        business_rules,
        workflows,
        screen_layouts,
        dashboard: None,
        reports,
        numbering_overrides: Vec::new(),
        app: None,
        seed_data: Vec::new(),
    })
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
        "nonprofit_association" => Ok(super::reference_packages::nonprofit_association_manifest_json()),
        "auto_service" => Ok(super::reference_packages::auto_service_manifest_json()),
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
    // Resolved once up front so the retag loop near the end doesn't
    // re-derive it per artifact - the same publisher `import_package`
    // already required to exist before this package could even be
    // imported (see that function's own comment).
    let publisher = super::publisher_service::resolve_for_package_id(conn, workspace_id, &manifest.package_id)?;

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
        // Every artifact above was already tagged 'local' the moment its
        // own service function created it (see solution_component_service
        // and migration 0030's own comment) - this overwrites that tag
        // with the installing package's real publisher, now that
        // installed_app_id exists to attribute it to. numbering_override
        // never went through a tag_local call (migration 0030 excludes it
        // from component-tagging entirely, same as custom_record - see
        // that migration's own comment), so retagging it here would just
        // insert a spurious new solution_components row rather than
        // correct an existing one - skipped explicitly. custom_record
        // artifacts (below, from seed_data) are inserted straight to
        // package_artifacts without ever entering this `artifacts` buffer,
        // so they never reach this loop at all.
        if *artifact_type != "numbering_override" {
            super::solution_component_service::retag(conn, workspace_id, artifact_type, metadata_id, &publisher.id, &installed_app_id, actor_user_id)?;
        }
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

/// Looks up the already-installed version of `new_manifest.package_id`
/// and its own original manifest - shared setup both `plan_update` and
/// `apply_update` need before they can compare anything. Fails closed
/// with a clear message if the package isn't installed yet at all
/// (that's `install`'s job, not this pair's) or if its recorded
/// installed-version manifest has somehow gone missing (should be
/// unreachable - `app_packages` rows are never deleted).
fn currently_installed_manifest(conn: &Connection, workspace_id: &str, package_id: &str) -> AppResult<(InstalledApp, IndustryPackageManifest)> {
    let installed = industry_package_repo::get_installed_app_by_package(conn, workspace_id, package_id)?
        .ok_or_else(|| AppError::Validation(format!("'{package_id}' isn't installed in this workspace yet - use Install, not Update")))?;
    let old_package = industry_package_repo::get_package_by_version(conn, workspace_id, package_id, &installed.installed_version)?
        .ok_or_else(|| AppError::NotFound("Currently installed package version".into()))?;
    let old_manifest = parse_manifest(&old_package.manifest_json)?;
    Ok((installed, old_manifest))
}

fn diff_keyed<'a, T>(old: &'a [T], new: &'a [T], key_of: impl Fn(&'a T) -> String, def_of: impl Fn(&'a T) -> serde_json::Value) -> Vec<crate::models::industry_package::PackageUpdateDiffEntry> {
    use crate::models::industry_package::PackageUpdateDiffEntry;
    let old_by_key: std::collections::HashMap<String, &T> = old.iter().map(|x| (key_of(x), x)).collect();
    let new_by_key: std::collections::HashMap<String, &T> = new.iter().map(|x| (key_of(x), x)).collect();
    let mut entries = Vec::new();
    for (key, item) in &new_by_key {
        match old_by_key.get(key) {
            None => entries.push(PackageUpdateDiffEntry { key: key.clone(), kind: "added".to_string() }),
            Some(old_item) => {
                if def_of(item) != def_of(old_item) {
                    entries.push(PackageUpdateDiffEntry { key: key.clone(), kind: "modified".to_string() });
                }
            }
        }
    }
    for key in old_by_key.keys() {
        if !new_by_key.contains_key(key) {
            entries.push(PackageUpdateDiffEntry { key: key.clone(), kind: "removed".to_string() });
        }
    }
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    entries
}

/// Compares a newly-imported package version against the version
/// currently installed, and reports what would change - the update-with-
/// diff review step spec 1.1/13.3 call for, shown before `apply_update`
/// ever runs. Objects (by key) and fields (by `entity_type.key`, since a
/// field's identity always needs both - see `ManifestField`'s own doc
/// comment) get a real per-item Added/Modified/Removed diff, since both
/// carry the deterministic keys a package author controls. Relationships,
/// business rules, workflows, screen layouts and reports have no such
/// stable identity across versions (a manifest re-declares them fresh
/// each time, matched only by position) - for those, this reports how
/// many *new* entries exist beyond the old count, which is also exactly
/// what `apply_update` is able to safely add (see that function's own
/// doc comment for why positions the old manifest already declared are
/// left untouched rather than guessed at).
pub fn plan_update(conn: &Connection, workspace_id: &str, new_app_package_id: &str) -> AppResult<crate::models::industry_package::PackageUpdateDiff> {
    let new_package = get_owned_package(conn, workspace_id, new_app_package_id)?;
    let new_manifest = parse_manifest(&new_package.manifest_json)?;
    let (installed, old_manifest) = currently_installed_manifest(conn, workspace_id, &new_manifest.package_id)?;

    let objects = diff_keyed(
        &old_manifest.objects,
        &new_manifest.objects,
        |o| o.key.clone(),
        |o| serde_json::to_value(&o.definition).unwrap_or_default(),
    );
    let fields = diff_keyed(
        &old_manifest.fields,
        &new_manifest.fields,
        |f| format!("{}.{}", f.definition.entity_type, f.key),
        |f| serde_json::to_value(&f.definition).unwrap_or_default(),
    );

    Ok(crate::models::industry_package::PackageUpdateDiff {
        package_id: new_manifest.package_id.clone(),
        from_version: installed.installed_version.clone(),
        to_version: new_manifest.version.clone(),
        objects,
        fields,
        relationships_added: new_manifest.relationships.len().saturating_sub(old_manifest.relationships.len()) as i64,
        business_rules_added: new_manifest.business_rules.len().saturating_sub(old_manifest.business_rules.len()) as i64,
        workflows_added: new_manifest.workflows.len().saturating_sub(old_manifest.workflows.len()) as i64,
        screen_layouts_added: new_manifest.screen_layouts.len().saturating_sub(old_manifest.screen_layouts.len()) as i64,
        reports_added: new_manifest.reports.len().saturating_sub(old_manifest.reports.len()) as i64,
    })
}

/// The actual work of `apply_update`, run inside its transaction - the
/// update counterpart to `run_install`, reusing the same per-subsystem
/// service calls. Scope, deliberately narrower than a fresh install:
///   - Objects and fields matched by key/`(entity_type, key)` against the
///     old manifest are updated in place (their real `update`/
///     `update_definition` calls, preserving whatever `is_active` state
///     they're already in); keys the new manifest doesn't declare are
///     left alone (no destructive removal - matches every other
///     "deactivate, never delete" convention in this codebase); brand
///     new keys are created fresh, exactly like `run_install`.
///   - Relationships/business rules/workflows/screen layouts/reports at a
///     position the old manifest already had are left completely
///     untouched - only positions beyond the old count are created. A new
///     screen layout tab's `related` or a new workflow action's
///     `relationship_ref` is resolved only against relationships this
///     same update pass just added (`new_relationship_index`); a
///     reference to an already-installed (old-position) relationship is
///     dropped rather than guessed at, since reconstructing exactly which
///     already-installed relationship corresponds to which old manifest
///     index isn't reliably recoverable after the fact - the same
///     documented tradeoff `export_local_workspace` makes for a
///     cross-workspace relationship id it doesn't own.
fn run_update(
    conn: &Connection,
    workspace_id: &str,
    old_manifest: &IndustryPackageManifest,
    new_manifest: &IndustryPackageManifest,
    installed_app_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    let publisher = super::publisher_service::resolve_for_package_id(conn, workspace_id, &new_manifest.package_id)?;
    let mut new_artifacts: Vec<(&'static str, String)> = Vec::new();

    for obj in &new_manifest.objects {
        match custom_object_service::get_by_key(conn, workspace_id, &obj.key)? {
            Some(existing) => {
                let update = crate::models::custom_object::CustomObjectDefinitionUpdate {
                    singular_label: obj.definition.singular_label.clone(),
                    plural_label: obj.definition.plural_label.clone(),
                    icon: obj.definition.icon.clone(),
                    prefix: obj.definition.prefix.clone(),
                    digits: obj.definition.digits,
                    is_active: existing.is_active,
                };
                custom_object_service::update(conn, &existing.id, &update, actor_user_id)?;
            }
            None => {
                let created = custom_object_service::create_with_key(conn, workspace_id, &obj.key, &obj.definition, actor_user_id)?;
                new_artifacts.push(("custom_object", created.id));
            }
        }
    }

    for field in &new_manifest.fields {
        let existing = custom_field_service::list_definitions(conn, workspace_id, &field.definition.entity_type, false)?
            .into_iter()
            .find(|d| d.key == field.key);
        match existing {
            Some(existing) => {
                let update = crate::models::custom_field::CustomFieldDefinitionUpdate {
                    label: field.definition.label.clone(),
                    options: field.definition.options.clone(),
                    required: field.definition.required,
                    show_in_list: field.definition.show_in_list,
                    sort_order: field.definition.sort_order,
                    is_active: existing.is_active,
                    min_value: field.definition.min_value.clone(),
                    max_value: field.definition.max_value.clone(),
                    max_length: field.definition.max_length,
                    regex_pattern: field.definition.regex_pattern.clone(),
                    is_searchable: field.definition.is_searchable,
                    is_filterable: field.definition.is_filterable,
                    is_reportable: field.definition.is_reportable,
                    default_value: field.definition.default_value.clone(),
                    is_unique: field.definition.is_unique,
                    help_text: field.definition.help_text.clone(),
                    placeholder: field.definition.placeholder.clone(),
                    is_hidden_by_default: field.definition.is_hidden_by_default,
                };
                custom_field_service::update_definition(conn, &existing.id, &update, actor_user_id)?;
            }
            None => {
                let created = custom_field_service::create_definition_with_key(conn, workspace_id, &field.key, &field.definition, actor_user_id)?;
                new_artifacts.push(("custom_field", created.id));
            }
        }
    }

    // New relationships only (position >= old count) - index-mapped so a
    // new screen layout tab / workflow action added in this same update
    // can resolve a `relationship_ref`/`related` pointing at one of them.
    let old_relationship_count = old_manifest.relationships.len();
    let mut new_relationship_index: std::collections::HashMap<usize, RelationshipDefinition> = std::collections::HashMap::new();
    for (i, rel_input) in new_manifest.relationships.iter().enumerate() {
        if i < old_relationship_count {
            continue;
        }
        let created = relationship_service::create(conn, workspace_id, rel_input, actor_user_id)?;
        new_artifacts.push(("relationship_definition", created.id.clone()));
        new_relationship_index.insert(i, created);
    }

    let old_rule_count = old_manifest.business_rules.len();
    for rule_input in new_manifest.business_rules.iter().skip(old_rule_count) {
        let created = business_rule_service::create_rule(conn, workspace_id, rule_input, actor_user_id)?;
        new_artifacts.push(("business_rule", created.id));
    }

    let old_workflow_count = old_manifest.workflows.len();
    for workflow_input in new_manifest.workflows.iter().skip(old_workflow_count) {
        // Only a `relationship_ref` resolvable against a newly-added
        // relationship survives - see this function's own doc comment.
        let mut resolved = workflow_input.clone();
        for action in &mut resolved.actions {
            if action.action_type != "create_record" && action.action_type != "update_related_record" {
                continue;
            }
            let Ok(mut params) = serde_json::from_str::<serde_json::Value>(&action.params_json) else { continue };
            let idx = params.get("relationship_ref").and_then(|v| v.as_u64()).map(|v| v as usize);
            match idx.and_then(|i| new_relationship_index.get(&i)) {
                Some(rel) => {
                    if let Some(obj) = params.as_object_mut() {
                        obj.remove("relationship_ref");
                        obj.insert("relationship_definition_id".to_string(), serde_json::Value::String(rel.id.clone()));
                    }
                    action.params_json = params.to_string();
                }
                None if idx.is_some() => {
                    if let Some(obj) = params.as_object_mut() {
                        obj.remove("relationship_ref");
                        action.params_json = params.to_string();
                    }
                }
                None => {}
            }
        }
        let created = workflow_service::create_rule(conn, workspace_id, &resolved, actor_user_id)?;
        new_artifacts.push(("workflow_definition", created.id));
    }

    let old_layout_count = old_manifest.screen_layouts.len();
    for layout in new_manifest.screen_layouts.iter().skip(old_layout_count) {
        let mut draft = layout.draft.clone();
        for tab in &mut draft.tabs {
            tab.related = tab
                .related
                .iter()
                .filter_map(|raw| raw.parse::<usize>().ok())
                .filter_map(|idx| new_relationship_index.get(&idx).map(|rel| rel.key.clone()))
                .collect();
        }
        let created = screen_layout_service::create_layout(
            conn,
            workspace_id,
            &ScreenLayoutInput { entity_type: layout.entity_type.clone(), name: layout.name.clone(), initial_fields: vec![] },
            actor_user_id,
        )?;
        let created =
            screen_layout_service::update_layout(conn, &created.id, &ScreenLayoutUpdate { name: layout.name.clone(), roles: vec![], draft }, actor_user_id)?;
        if layout.publish {
            screen_layout_service::publish_layout(conn, &created.id, actor_user_id)?;
        }
        new_artifacts.push(("screen_layout", created.id));
    }

    let old_report_count = old_manifest.reports.len();
    for report_input in new_manifest.reports.iter().skip(old_report_count) {
        let created = custom_report_service::create(conn, workspace_id, report_input, actor_user_id)?;
        new_artifacts.push(("custom_report", created.id));
    }

    for (artifact_type, metadata_id) in &new_artifacts {
        industry_package_repo::insert_artifact(conn, &new_uuid(), installed_app_id, artifact_type, metadata_id, &new_manifest.version)?;
        super::solution_component_service::retag(conn, workspace_id, artifact_type, metadata_id, &publisher.id, installed_app_id, actor_user_id)?;
    }

    industry_package_repo::update_installed_version(conn, installed_app_id, &new_manifest.version, actor_user_id)?;
    Ok(())
}

/// Applies a newly-imported package version over the currently installed
/// one - the same safety-backup + transaction + `app_install_runs`
/// bookkeeping `install` uses, calling `run_update` instead of
/// `run_install`. This is what finally replaces the "reinstalling an
/// already-installed package_id is rejected outright" behavior this
/// module's own doc comment used to list as deferred - review the diff
/// from `plan_update` first; this applies it unconditionally once called.
pub fn apply_update(conn: &Connection, workspace_id: &str, new_app_package_id: &str, actor_user_id: Option<&str>) -> AppResult<InstalledApp> {
    require_admin(conn, actor_user_id)?;
    let new_package = get_owned_package(conn, workspace_id, new_app_package_id)?;
    let new_manifest = parse_manifest(&new_package.manifest_json)?;
    let (installed, old_manifest) = currently_installed_manifest(conn, workspace_id, &new_manifest.package_id)?;

    let run_id = new_uuid();
    industry_package_repo::start_run(conn, &run_id, workspace_id, &new_manifest.package_id, &new_manifest.version, "update", actor_user_id)?;

    let outcome: AppResult<String> = (|| {
        validate(conn, workspace_id, &new_manifest)?;
        let backup_path = take_safety_backup(conn, actor_user_id)?;
        let tx = conn.unchecked_transaction()?;
        run_update(&tx, workspace_id, &old_manifest, &new_manifest, &installed.id, actor_user_id)?;
        tx.commit()?;
        Ok(backup_path)
    })();

    match outcome {
        Ok(backup_path) => {
            industry_package_repo::complete_run(conn, &run_id, "succeeded", None, Some(&backup_path))?;
            industry_package_repo::get_installed_app(conn, &installed.id)?.ok_or_else(|| AppError::NotFound("Installed app".into()))
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
