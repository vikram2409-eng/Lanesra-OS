//! Solution Packages & Admin IA design spec, Phase 3: component-tagging,
//! the Local Workspace (Custom) grouping, `.lanesra` manifest export,
//! and update-with-diff (`plan_update`/`apply_update`) - see
//! `industry_package_service`'s own module doc comment and migration
//! 0030's comment for the full design.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::domain::AppError;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{custom_field_service, custom_object_service, industry_package_service, solution_component_service, user_service, workspace_service};
use serde_json::json;

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Solution Phase 3 Co".into(),
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

fn object_input(label: &str, icon: &str) -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: label.into(), plural_label: format!("{label}s"), icon: icon.into(), prefix: "WID".into(), digits: 4 }
}

fn field_json(key: &str, entity_type: &str, label: &str) -> serde_json::Value {
    json!({
        "key": key, "entity_type": entity_type, "label": label, "field_type": "text", "options": [],
        "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null,
        "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false,
        "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null,
        "placeholder": null, "is_hidden_by_default": false
    })
}

/// v1: `widget` (icon 🔧) + `gadget`, one field `widget.notes` ("Notes").
fn v1_manifest(package_id: &str) -> String {
    json!({
        "format_version": 1, "package_id": package_id, "name": "Widgets Pack", "industry": "Testing",
        "version": "1.0.0", "min_lanesra_version": "0.1.0",
        "objects": [
            {"key": "widget", "singular_label": "Widget", "plural_label": "Widgets", "icon": "🔧", "prefix": "WID", "digits": 4},
            {"key": "gadget", "singular_label": "Gadget", "plural_label": "Gadgets", "icon": "⚙️", "prefix": "GAD", "digits": 4}
        ],
        "fields": [field_json("notes", "widget", "Notes")],
    })
    .to_string()
}

/// v2: `widget` icon changed to 🔩 (modified), `gadget` dropped (removed,
/// but never destroyed), `extra` added. `widget.notes` relabeled
/// (modified), `widget.priority` added.
fn v2_manifest(package_id: &str) -> String {
    json!({
        "format_version": 1, "package_id": package_id, "name": "Widgets Pack", "industry": "Testing",
        "version": "1.1.0", "min_lanesra_version": "0.1.0",
        "objects": [
            {"key": "widget", "singular_label": "Widget", "plural_label": "Widgets", "icon": "🔩", "prefix": "WID", "digits": 4},
            {"key": "extra", "singular_label": "Extra", "plural_label": "Extras", "icon": "➕", "prefix": "EXT", "digits": 4}
        ],
        "fields": [field_json("notes", "widget", "Internal Notes"), field_json("priority", "widget", "Priority")],
    })
    .to_string()
}

// --- component-tagging ---------------------------------------------------

#[test]
fn a_hand_built_custom_object_is_tagged_to_the_local_publisher() {
    let (conn, ws, admin) = setup_workspace();
    let created = custom_object_service::create(&conn, &ws, &object_input("Vendor", "🏢"), Some(&admin)).unwrap();
    let local = solution_component_service::list_local(&conn, &ws).unwrap();
    assert!(local.iter().any(|c| c.artifact_type == "custom_object" && c.metadata_id == created.id));
}

#[test]
fn a_hand_built_custom_field_is_tagged_to_the_local_publisher() {
    let (conn, ws, admin) = setup_workspace();
    let obj = custom_object_service::create(&conn, &ws, &object_input("Vendor", "🏢"), Some(&admin)).unwrap();
    let input = CustomFieldDefinitionInput {
        entity_type: obj.key.clone(), label: "Rating".into(), field_type: "number".into(), options: vec![],
        required: false, show_in_list: true, sort_order: 0, min_value: None, max_value: None, max_length: None,
        regex_pattern: None, is_searchable: false, is_filterable: false, is_reportable: false, default_value: None,
        is_unique: false, help_text: None, placeholder: None, is_hidden_by_default: false,
    };
    let created = custom_field_service::create_definition(&conn, &ws, &input, Some(&admin)).unwrap();
    let local = solution_component_service::list_local(&conn, &ws).unwrap();
    assert!(local.iter().any(|c| c.artifact_type == "custom_field" && c.metadata_id == created.id));
}

#[test]
fn installed_package_components_are_retagged_away_from_local() {
    let (conn, ws, admin) = setup_workspace();
    let input = ImportPackageInput { manifest_json: v1_manifest("lanesra.widgets") };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    industry_package_service::install(&conn, &ws, &package.id, Some(&admin)).unwrap();

    let widget = custom_object_service::get_by_key(&conn, &ws, "widget").unwrap().unwrap();
    let local = solution_component_service::list_local(&conn, &ws).unwrap();
    assert!(!local.iter().any(|c| c.metadata_id == widget.id), "an installed component must not still read as local");

    let workspace_components = solution_component_service::list_for_workspace(&conn, &ws).unwrap();
    let tagged = workspace_components.iter().find(|c| c.component.metadata_id == widget.id).unwrap();
    assert_eq!(tagged.publisher_key, "lanesra");
    assert!(!tagged.is_local);
    assert_eq!(tagged.installed_app_name.as_deref(), Some("Widgets Pack"));
}

#[test]
fn local_workspace_summary_counts_only_local_components_by_type() {
    let (conn, ws, admin) = setup_workspace();
    custom_object_service::create(&conn, &ws, &object_input("Vendor", "🏢"), Some(&admin)).unwrap();
    custom_object_service::create(&conn, &ws, &object_input("Asset", "🖥"), Some(&admin)).unwrap();

    let input = ImportPackageInput { manifest_json: v1_manifest("lanesra.widgets") };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    industry_package_service::install(&conn, &ws, &package.id, Some(&admin)).unwrap();

    let summary = solution_component_service::local_workspace_summary(&conn, &ws).unwrap();
    assert_eq!(summary.component_count, 2, "the 2 installed widgets/fields must not be counted as local");
    let object_count = summary.components_by_type.iter().find(|(t, _)| t == "custom_object").map(|(_, n)| *n).unwrap_or(0);
    assert_eq!(object_count, 2);
}

// --- version history -------------------------------------------------------

#[test]
fn list_package_versions_returns_every_imported_version_oldest_first() {
    let (conn, ws, admin) = setup_workspace();
    let v1 = industry_package_service::import_package(&conn, &ws, &ImportPackageInput { manifest_json: v1_manifest("lanesra.widgets") }, Some(&admin)).unwrap();
    industry_package_service::install(&conn, &ws, &v1.id, Some(&admin)).unwrap();
    industry_package_service::import_package(&conn, &ws, &ImportPackageInput { manifest_json: v2_manifest("lanesra.widgets") }, Some(&admin)).unwrap();

    let versions = industry_package_service::list_package_versions(&conn, &ws, "lanesra.widgets").unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version, "1.0.0");
    assert_eq!(versions[1].version, "1.1.0");
}

// --- export ------------------------------------------------------------

#[test]
fn export_local_workspace_is_empty_when_nothing_has_been_hand_built() {
    let (conn, ws, admin) = setup_workspace();
    let json = industry_package_service::export_local_workspace(&conn, &ws, Some(&admin)).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(manifest["objects"].as_array().unwrap().len(), 0);
    assert_eq!(manifest["package_id"], "local.workspace_export");
}

#[test]
fn export_local_workspace_round_trips_into_a_fresh_workspace() {
    let (conn, ws, admin) = setup_workspace();
    let obj = custom_object_service::create(&conn, &ws, &object_input("Vendor", "🏢"), Some(&admin)).unwrap();
    let field_input = CustomFieldDefinitionInput {
        entity_type: obj.key.clone(), label: "Rating".into(), field_type: "number".into(), options: vec![],
        required: false, show_in_list: true, sort_order: 0, min_value: None, max_value: None, max_length: None,
        regex_pattern: None, is_searchable: false, is_filterable: false, is_reportable: false, default_value: None,
        is_unique: false, help_text: None, placeholder: None, is_hidden_by_default: false,
    };
    custom_field_service::create_definition(&conn, &ws, &field_input, Some(&admin)).unwrap();

    let exported = industry_package_service::export_local_workspace(&conn, &ws, Some(&admin)).unwrap();

    // A second, completely independent workspace - export must be
    // importable somewhere that has never seen this data before.
    let (conn2, ws2, admin2) = setup_workspace();
    let package = industry_package_service::import_package(&conn2, &ws2, &ImportPackageInput { manifest_json: exported }, Some(&admin2)).unwrap();
    assert_eq!(package.package_id, "local.workspace_export");
    let installed = industry_package_service::install(&conn2, &ws2, &package.id, Some(&admin2)).unwrap();
    assert_eq!(installed.package_id, "local.workspace_export");

    let reimported = custom_object_service::get_by_key(&conn2, &ws2, &obj.key).unwrap().unwrap();
    assert_eq!(reimported.singular_label, "Vendor");
    let fields = custom_field_service::list_definitions(&conn2, &ws2, &obj.key, true).unwrap();
    assert!(fields.iter().any(|f| f.label == "Rating"));
}

// --- update-with-diff ----------------------------------------------------

#[test]
fn plan_update_rejects_a_package_that_isnt_installed_yet() {
    let (conn, ws, admin) = setup_workspace();
    let package = industry_package_service::import_package(&conn, &ws, &ImportPackageInput { manifest_json: v1_manifest("lanesra.widgets") }, Some(&admin)).unwrap();
    let err = industry_package_service::plan_update(&conn, &ws, &package.id).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn plan_update_reports_added_modified_and_removed_objects_and_fields() {
    let (conn, ws, admin) = setup_workspace();
    let v1 = industry_package_service::import_package(&conn, &ws, &ImportPackageInput { manifest_json: v1_manifest("lanesra.widgets") }, Some(&admin)).unwrap();
    industry_package_service::install(&conn, &ws, &v1.id, Some(&admin)).unwrap();
    let v2 = industry_package_service::import_package(&conn, &ws, &ImportPackageInput { manifest_json: v2_manifest("lanesra.widgets") }, Some(&admin)).unwrap();

    let diff = industry_package_service::plan_update(&conn, &ws, &v2.id).unwrap();
    assert_eq!(diff.from_version, "1.0.0");
    assert_eq!(diff.to_version, "1.1.0");

    let object_kind = |k: &str| diff.objects.iter().find(|e| e.key == k).map(|e| e.kind.clone());
    assert_eq!(object_kind("widget"), Some("modified".to_string()));
    assert_eq!(object_kind("gadget"), Some("removed".to_string()));
    assert_eq!(object_kind("extra"), Some("added".to_string()));

    let field_kind = |k: &str| diff.fields.iter().find(|e| e.key == k).map(|e| e.kind.clone());
    assert_eq!(field_kind("widget.notes"), Some("modified".to_string()));
    assert_eq!(field_kind("widget.priority"), Some("added".to_string()));
}

#[test]
fn apply_update_updates_in_place_adds_new_and_never_deletes_removed() {
    let (conn, ws, admin) = setup_workspace();
    let v1 = industry_package_service::import_package(&conn, &ws, &ImportPackageInput { manifest_json: v1_manifest("lanesra.widgets") }, Some(&admin)).unwrap();
    industry_package_service::install(&conn, &ws, &v1.id, Some(&admin)).unwrap();
    let v2 = industry_package_service::import_package(&conn, &ws, &ImportPackageInput { manifest_json: v2_manifest("lanesra.widgets") }, Some(&admin)).unwrap();

    let updated = industry_package_service::apply_update(&conn, &ws, &v2.id, Some(&admin)).unwrap();
    assert_eq!(updated.installed_version, "1.1.0");

    // widget: modified in place, same row (no duplicate created)
    let widget = custom_object_service::get_by_key(&conn, &ws, "widget").unwrap().unwrap();
    assert_eq!(widget.icon, "🔩");

    // gadget: removed from the manifest, but never destroyed
    let gadget = custom_object_service::get_by_key(&conn, &ws, "gadget").unwrap();
    assert!(gadget.is_some(), "an object dropped from a newer manifest must not be deleted");

    // extra: newly created
    let extra = custom_object_service::get_by_key(&conn, &ws, "extra").unwrap();
    assert!(extra.is_some());

    // fields: notes relabeled in place, priority newly created
    let fields = custom_field_service::list_definitions(&conn, &ws, "widget", true).unwrap();
    let notes = fields.iter().find(|f| f.key == "notes").unwrap();
    assert_eq!(notes.label, "Internal Notes");
    assert!(fields.iter().any(|f| f.key == "priority"));

    // newly-created components from the update are tagged to the
    // installing publisher, not left dangling as 'local'.
    let workspace_components = solution_component_service::list_for_workspace(&conn, &ws).unwrap();
    let extra_id = custom_object_service::get_by_key(&conn, &ws, "extra").unwrap().unwrap().id;
    let tagged = workspace_components.iter().find(|c| c.component.metadata_id == extra_id).unwrap();
    assert_eq!(tagged.publisher_key, "lanesra");
}

#[test]
fn apply_update_requires_admin() {
    let (conn, ws, admin) = setup_workspace();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let v1 = industry_package_service::import_package(&conn, &ws, &ImportPackageInput { manifest_json: v1_manifest("lanesra.widgets") }, Some(&admin)).unwrap();
    industry_package_service::install(&conn, &ws, &v1.id, Some(&admin)).unwrap();
    let v2 = industry_package_service::import_package(&conn, &ws, &ImportPackageInput { manifest_json: v2_manifest("lanesra.widgets") }, Some(&admin)).unwrap();

    let err = industry_package_service::apply_update(&conn, &ws, &v2.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
