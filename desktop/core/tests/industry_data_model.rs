//! Industry Data Model foundations: package manifest format, the
//! InstalledApp ownership/versioning registry, and transactional
//! install/rollback machinery - see `industry_package_service`'s own
//! module doc comment for what's built here versus deferred to a later
//! phase (per-industry package content, update-with-diff, destructive
//! uninstall).

use lanesra_core::db::open_in_memory_db;
use lanesra_core::domain::AppError;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{custom_object_service, industry_package_service, user_service, workspace_service};
use serde_json::json;

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Industry Pack Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin User".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn user_with_role(conn: &rusqlite::Connection, ws: &str, admin: &str, username: &str, role: &str) -> String {
    user_service::create(
        conn, ws,
        &NewUser { username: username.into(), display_name: username.into(), password: "anothersecretpw".into(), roles: vec![role.into()] },
        Some(admin),
    )
    .unwrap()
    .id
}

/// A full manifest exercising every sub-system `run_install` touches:
/// object, field, relationship, business rule, workflow, screen layout,
/// report, dashboard (chart widget resolving its report_ref), numbering
/// override, app (with a recommended permission), and one seed record
/// carrying a custom field value.
fn full_manifest_json(package_id: &str, version: &str, min_lanesra_version: &str) -> String {
    json!({
        "format_version": 1,
        "package_id": package_id,
        "name": "Field Service Test Pack",
        "industry": "Field Service",
        "version": version,
        "min_lanesra_version": min_lanesra_version,
        "dependencies": [],
        "objects": [
            {
                "key": "svc_ticket",
                "singular_label": "Service Ticket",
                "plural_label": "Service Tickets",
                "icon": "🎫",
                "prefix": "TKT",
                "digits": 4
            }
        ],
        "fields": [
            {
                "key": "priority",
                "entity_type": "svc_ticket",
                "label": "Priority",
                "field_type": "select",
                "options": ["Low", "High"],
                "required": false,
                "show_in_list": true,
                "sort_order": 0,
                "min_value": null,
                "max_value": null,
                "max_length": null,
                "regex_pattern": null,
                "is_searchable": false,
                "is_filterable": true,
                "is_reportable": true,
                "default_value": "Low",
                "is_unique": false,
                "help_text": null,
                "placeholder": null,
                "is_hidden_by_default": false
            }
        ],
        "relationships": [
            {
                "source_entity_type": "svc_ticket",
                "target_entity_type": "Company",
                "relationship_type": "many_to_one",
                "forward_label": "Company",
                "reverse_label": "Service Tickets",
                "is_required": false,
                "show_related_list": true,
                "delete_behavior": "restrict",
                "sort_order": 0
            }
        ],
        "business_rules": [
            {
                "entity_type": "svc_ticket",
                "name": "Warn on active tickets",
                "description": null,
                "match_type": "all",
                "priority": 0,
                "effective_start_date": null,
                "effective_end_date": null,
                "conditions": [
                    { "field_source": "builtin", "field_key": "status", "operator": "equals", "value": "Active" }
                ],
                "actions": [
                    { "action_type": "show_warning", "target_field_key": null, "target_field_source": "custom", "action_value": null, "message": "Ticket needs review" }
                ]
            }
        ],
        "workflows": [
            {
                "entity_type": "svc_ticket",
                "name": "Notify on create",
                "description": null,
                "trigger_type": "record_created",
                "trigger_status": null,
                "trigger_field_key": null,
                "trigger_field_source": "custom",
                "trigger_offset_days": 0,
                "match_type": "all",
                "priority": 0,
                "conditions": [],
                "actions": [
                    { "action_type": "add_notification", "params_json": "{\"message\":\"New ticket created\",\"audience\":\"all_admins\"}" }
                ]
            }
        ],
        "screen_layouts": [
            {
                "entity_type": "svc_ticket",
                "name": "Default",
                "draft": { "tabs": [ { "id": "tab1", "title": "Details", "sections": [], "related": ["0"] } ] },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Tickets by status", "entity_type": "svc_ticket", "group_by_source": "builtin", "group_by_field": "status", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Service Dashboard",
            "widgets": [ { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } } ],
            "publish": true
        },
        "numbering_overrides": [
            { "entity_type": "Task", "prefix": "SVC", "digits": 5 }
        ],
        "app": {
            "name": "Field Service",
            "icon": "🔧",
            "description": "Field service test app",
            "object_keys": ["svc_ticket"],
            "use_package_dashboard": true,
            "publish": true,
            "recommended_permissions": [ { "role": "Sales", "level": "editor" } ]
        },
        "seed_data": [
            {
                "object_key": "svc_ticket",
                "record": { "object_key": "svc_ticket", "primary_name": "Sample Ticket", "status": "Active", "owner_user_id": null, "notes": null },
                "field_values": { "priority": "High" }
            }
        ]
    })
    .to_string()
}

#[test]
fn importing_a_package_requires_admin() {
    let (conn, ws, admin) = setup_workspace();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let input = ImportPackageInput { manifest_json: full_manifest_json("lanesra.field_service", "1.0.0", "0.1.0") };
    let err = industry_package_service::import_package(&conn, &ws, &input, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}

#[test]
fn import_then_install_creates_every_kind_of_artifact_and_the_installed_app_row() {
    let (conn, ws, admin) = setup_workspace();
    let input = ImportPackageInput { manifest_json: full_manifest_json("lanesra.field_service", "1.0.0", "0.1.0") };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    assert_eq!(package.package_id, "lanesra.field_service");

    let installed = industry_package_service::install(&conn, &ws, &package.id, Some(&admin)).unwrap();
    assert_eq!(installed.package_id, "lanesra.field_service");
    assert_eq!(installed.status, "active");
    assert_eq!(installed.name, "Field Service");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 1);
    assert_eq!(installed.recommended_permissions[0].role, "Sales");

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let mut types: Vec<&str> = detail.artifacts.iter().map(|a| a.artifact_type.as_str()).collect();
    types.sort();
    assert_eq!(
        types,
        vec![
            "business_rule",
            "custom_field",
            "custom_object",
            "custom_record",
            "custom_report",
            "dashboard_layout",
            "numbering_override",
            "relationship_definition",
            "screen_layout",
            "workflow_definition",
        ]
    );
    for artifact in &detail.artifacts {
        assert_eq!(artifact.origin_version, "1.0.0");
        assert!(!artifact.is_locally_customized);
    }

    // Installing the same package_id a second time is a conflict, not a
    // silent second install (update-with-diff is explicitly out of scope
    // for this foundation phase).
    let second_import = ImportPackageInput { manifest_json: full_manifest_json("lanesra.field_service", "1.0.1", "0.1.0") };
    let second_package = industry_package_service::import_package(&conn, &ws, &second_import, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &second_package.id, Some(&admin)).unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));

    let runs = industry_package_service::list_runs(&conn, &ws).unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs.iter().filter(|r| r.status == "succeeded").count(), 1);
    assert_eq!(runs.iter().filter(|r| r.status == "failed").count(), 1);
}

#[test]
fn a_failure_mid_install_rolls_back_everything_the_transaction_touched() {
    let (conn, ws, admin) = setup_workspace();

    // Two objects sharing the same key - the second collides, which
    // create_with_key hard-fails on rather than silently renaming (see
    // that function's own doc comment). The whole transaction - including
    // the first object, which succeeded before the failure - must roll
    // back, and no installed_apps row should exist at all.
    let manifest = json!({
        "format_version": 1,
        "package_id": "lanesra.broken",
        "name": "Broken Pack",
        "industry": "Testing",
        "version": "1.0.0",
        "min_lanesra_version": "0.1.0",
        "objects": [
            { "key": "dup_obj", "singular_label": "Dup One", "plural_label": "Dup Ones", "icon": "📦", "prefix": "DUP", "digits": 4 },
            { "key": "dup_obj", "singular_label": "Dup Two", "plural_label": "Dup Twos", "icon": "📦", "prefix": "DU2", "digits": 4 }
        ]
    })
    .to_string();

    let input = ImportPackageInput { manifest_json: manifest };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("already exists"));

    // Nothing from the failed transaction survived - not even the first
    // object, which succeeded before the second one's collision.
    let objects = custom_object_service::list(&conn, &ws, false).unwrap();
    assert!(objects.is_empty());
    assert!(industry_package_service::list_installed(&conn, &ws).unwrap().is_empty());

    let runs = industry_package_service::list_runs(&conn, &ws).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, "failed");
    assert!(runs[0].error_message.as_deref().unwrap().contains("already exists"));
    assert!(runs[0].backup_snapshot_path.is_none());
}

#[test]
fn install_is_blocked_by_an_unmet_required_dependency() {
    let (conn, ws, admin) = setup_workspace();
    let manifest = json!({
        "format_version": 1,
        "package_id": "lanesra.depends_on_core",
        "name": "Depends On Core",
        "industry": "Testing",
        "version": "1.0.0",
        "min_lanesra_version": "0.1.0",
        "dependencies": [
            { "package_id": "lanesra.core_pack", "version_constraint": ">=1.0.0", "is_required": true }
        ]
    })
    .to_string();
    let input = ImportPackageInput { manifest_json: manifest };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();

    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("lanesra.core_pack"));

    // The pre-flight validate() failure still leaves a readable run row -
    // see install()'s own doc comment on why validate happens inside the
    // recorded attempt.
    let runs = industry_package_service::list_runs(&conn, &ws).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, "failed");
}

#[test]
fn install_is_blocked_by_an_unmet_min_lanesra_version() {
    let (conn, ws, admin) = setup_workspace();
    let input = ImportPackageInput { manifest_json: full_manifest_json("lanesra.future", "1.0.0", "99.0.0") };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("99.0.0"));
}

#[test]
fn deactivate_hides_without_deleting_business_data_and_reactivate_restores_it() {
    let (conn, ws, admin) = setup_workspace();
    let input = ImportPackageInput { manifest_json: full_manifest_json("lanesra.field_service", "1.0.0", "0.1.0") };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let installed = industry_package_service::install(&conn, &ws, &package.id, Some(&admin)).unwrap();

    let deactivated = industry_package_service::deactivate(&conn, &installed.id, Some(&admin)).unwrap();
    assert_eq!(deactivated.status, "deactivated");
    assert!(deactivated.deactivated_at.is_some());

    // The custom object and its one seed record are untouched.
    let objects = custom_object_service::list(&conn, &ws, false).unwrap();
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].key, "svc_ticket");

    let reactivated = industry_package_service::reactivate(&conn, &installed.id, Some(&admin)).unwrap();
    assert_eq!(reactivated.status, "active");
    assert!(reactivated.deactivated_at.is_none());
}

#[test]
fn deactivate_and_reactivate_require_admin() {
    let (conn, ws, admin) = setup_workspace();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let input = ImportPackageInput { manifest_json: full_manifest_json("lanesra.field_service", "1.0.0", "0.1.0") };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let installed = industry_package_service::install(&conn, &ws, &package.id, Some(&admin)).unwrap();

    let err = industry_package_service::deactivate(&conn, &installed.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}

#[test]
fn explicit_object_keys_are_never_silently_renamed_on_collision() {
    // industry_package_service reuses custom_object_service::create_with_key,
    // which must hard-fail (not auto-"_2"-suffix) so every other manifest
    // entry that names this object by its exact key stays correct.
    let (conn, ws, admin) = setup_workspace();
    custom_object_service::create(
        &conn,
        &ws,
        &CustomObjectDefinitionInput {
            singular_label: "Existing".into(),
            plural_label: "Existings".into(),
            icon: "📦".into(),
            prefix: "EXS".into(),
            digits: 4,
        },
        Some(&admin),
    )
    .unwrap();
    let existing_key = custom_object_service::list(&conn, &ws, false).unwrap()[0].key.clone();

    let err = custom_object_service::create_with_key(
        &conn,
        &ws,
        &existing_key,
        &CustomObjectDefinitionInput {
            singular_label: "Other".into(),
            plural_label: "Others".into(),
            icon: "📦".into(),
            prefix: "OTH".into(),
            digits: 4,
        },
        Some(&admin),
    )
    .unwrap_err();
    assert!(err.to_string().contains("already exists"));
}
