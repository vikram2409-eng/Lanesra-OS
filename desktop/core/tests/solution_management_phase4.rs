//! Solution Packages & Admin IA design spec, Phase 4: named, scoped
//! Solutions - the Dynamics-365-style "build a solution in test, export
//! it, import it in prod" workflow. See migration 0031's own comment and
//! `solution_service`/`industry_package_service::export_solution`'s doc
//! comments for the full design.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::domain::AppError;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::solution::{SolutionInput, SolutionMemberInput, SolutionUpdate};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{custom_field_service, custom_object_service, industry_package_service, solution_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Solution Phase 4 Co".into(),
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
    CustomObjectDefinitionInput { singular_label: label.into(), plural_label: format!("{label}s"), icon: icon.into(), prefix: "SOL".into(), digits: 4 }
}

fn solution_input(name: &str) -> SolutionInput {
    SolutionInput { name: name.into(), description: Some("A test solution".into()), version: None, publisher_id: None }
}

// --- create / update / delete --------------------------------------------

#[test]
fn create_solution_defaults_version_and_rejects_duplicate_names() {
    let (conn, ws, admin) = setup_workspace();
    let solution = solution_service::create(&conn, &ws, &solution_input("Field Ops Extensions"), Some(&admin)).unwrap();
    assert_eq!(solution.version, "1.0.0.0");
    assert_eq!(solution.publisher_name.as_deref(), Some("Local Workspace"));

    let err = solution_service::create(&conn, &ws, &solution_input("Field Ops Extensions"), Some(&admin)).unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
}

#[test]
fn create_solution_requires_admin() {
    let (conn, ws, admin) = setup_workspace();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let err = solution_service::create(&conn, &ws, &solution_input("Sales Extras"), Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}

#[test]
fn update_solution_renames_and_rebumps_version() {
    let (conn, ws, admin) = setup_workspace();
    let solution = solution_service::create(&conn, &ws, &solution_input("Field Ops Extensions"), Some(&admin)).unwrap();
    let updated = solution_service::update(
        &conn,
        &ws,
        &solution.id,
        &SolutionUpdate { name: "Field Ops Pack".into(), description: Some("renamed".into()), version: "1.1.0.0".into(), publisher_id: None },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(updated.name, "Field Ops Pack");
    assert_eq!(updated.version, "1.1.0.0");
}

#[test]
fn delete_solution_removes_it_but_leaves_the_underlying_component_untouched() {
    let (conn, ws, admin) = setup_workspace();
    let obj = custom_object_service::create(&conn, &ws, &object_input("Vendor", "🏢"), Some(&admin)).unwrap();
    let solution = solution_service::create(&conn, &ws, &solution_input("Field Ops Extensions"), Some(&admin)).unwrap();
    solution_service::add_component(&conn, &ws, &solution.id, &SolutionMemberInput { artifact_type: "custom_object".into(), metadata_id: obj.id.clone() }, Some(&admin)).unwrap();

    solution_service::delete(&conn, &ws, &solution.id, Some(&admin)).unwrap();

    assert!(solution_service::get(&conn, &ws, &solution.id).is_err(), "the solution itself should be gone");
    let still_there = custom_object_service::get_by_key(&conn, &ws, &obj.key).unwrap();
    assert!(still_there.is_some(), "deleting a Solution must never delete the components it curated");
}

// --- membership ------------------------------------------------------------

#[test]
fn adding_a_component_that_doesnt_exist_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let solution = solution_service::create(&conn, &ws, &solution_input("Field Ops Extensions"), Some(&admin)).unwrap();
    let err = solution_service::add_component(
        &conn,
        &ws,
        &solution.id,
        &SolutionMemberInput { artifact_type: "custom_object".into(), metadata_id: "nonexistent".into() },
        Some(&admin),
    )
    .unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}

#[test]
fn add_and_remove_component_updates_the_curated_membership() {
    let (conn, ws, admin) = setup_workspace();
    let obj = custom_object_service::create(&conn, &ws, &object_input("Vendor", "🏢"), Some(&admin)).unwrap();
    let solution = solution_service::create(&conn, &ws, &solution_input("Field Ops Extensions"), Some(&admin)).unwrap();

    solution_service::add_component(&conn, &ws, &solution.id, &SolutionMemberInput { artifact_type: "custom_object".into(), metadata_id: obj.id.clone() }, Some(&admin)).unwrap();
    let detail = solution_service::get_detail(&conn, &ws, &solution.id).unwrap();
    assert_eq!(detail.members.len(), 1);
    assert_eq!(detail.solution.member_count, 1);

    solution_service::remove_component(&conn, &ws, &solution.id, "custom_object", &obj.id, Some(&admin)).unwrap();
    let detail = solution_service::get_detail(&conn, &ws, &solution.id).unwrap();
    assert_eq!(detail.members.len(), 0);
}

// --- export: scoped, not everything ----------------------------------------

#[test]
fn export_solution_only_includes_curated_members_not_every_local_component() {
    let (conn, ws, admin) = setup_workspace();
    let curated = custom_object_service::create(&conn, &ws, &object_input("Vendor", "🏢"), Some(&admin)).unwrap();
    let uncurated = custom_object_service::create(&conn, &ws, &object_input("Asset", "🖥"), Some(&admin)).unwrap();

    let solution = solution_service::create(&conn, &ws, &solution_input("Field Ops Extensions"), Some(&admin)).unwrap();
    solution_service::add_component(&conn, &ws, &solution.id, &SolutionMemberInput { artifact_type: "custom_object".into(), metadata_id: curated.id.clone() }, Some(&admin)).unwrap();

    let exported = industry_package_service::export_solution(&conn, &ws, &solution.id, Some(&admin)).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&exported).unwrap();
    let object_keys: Vec<&str> = manifest["objects"].as_array().unwrap().iter().map(|o| o["key"].as_str().unwrap()).collect();
    assert_eq!(object_keys, vec!["vendor"]);
    assert!(!object_keys.contains(&uncurated.key.as_str()), "an uncurated local component must not leak into a scoped solution export");
    assert_eq!(manifest["package_id"], format!("local.solution.{}", solution.id));
    assert_eq!(manifest["name"], "Field Ops Extensions");
    assert_eq!(manifest["version"], "1.0.0.0");
}

#[test]
fn export_solution_round_trips_into_a_separate_prod_workspace() {
    let (conn, ws, admin) = setup_workspace();
    let obj = custom_object_service::create(&conn, &ws, &object_input("Vendor", "🏢"), Some(&admin)).unwrap();
    let field_input = CustomFieldDefinitionInput {
        entity_type: obj.key.clone(), label: "Rating".into(), field_type: "number".into(), options: vec![],
        required: false, show_in_list: true, sort_order: 0, min_value: None, max_value: None, max_length: None,
        regex_pattern: None, is_searchable: false, is_filterable: false, is_reportable: false, default_value: None,
        is_unique: false, help_text: None, placeholder: None, is_hidden_by_default: false,
    };
    let field = custom_field_service::create_definition(&conn, &ws, &field_input, Some(&admin)).unwrap();

    let solution = solution_service::create(&conn, &ws, &solution_input("Field Ops Extensions"), Some(&admin)).unwrap();
    solution_service::add_component(&conn, &ws, &solution.id, &SolutionMemberInput { artifact_type: "custom_object".into(), metadata_id: obj.id.clone() }, Some(&admin)).unwrap();
    solution_service::add_component(&conn, &ws, &solution.id, &SolutionMemberInput { artifact_type: "custom_field".into(), metadata_id: field.id.clone() }, Some(&admin)).unwrap();

    let exported = industry_package_service::export_solution(&conn, &ws, &solution.id, Some(&admin)).unwrap();

    // "prod" - a second, completely independent workspace that has never
    // seen this data before, the same standalone-instance-as-environment
    // story migration 0031's own comment describes.
    let (prod_conn, prod_ws, prod_admin) = setup_workspace();
    let package = industry_package_service::import_package(&prod_conn, &prod_ws, &ImportPackageInput { manifest_json: exported }, Some(&prod_admin)).unwrap();
    assert_eq!(package.package_id, format!("local.solution.{}", solution.id));
    let installed = industry_package_service::install(&prod_conn, &prod_ws, &package.id, Some(&prod_admin)).unwrap();
    assert_eq!(installed.installed_version, "1.0.0.0");

    let reimported = custom_object_service::get_by_key(&prod_conn, &prod_ws, &obj.key).unwrap().unwrap();
    assert_eq!(reimported.singular_label, "Vendor");
    let fields = custom_field_service::list_definitions(&prod_conn, &prod_ws, &obj.key, true).unwrap();
    assert!(fields.iter().any(|f| f.label == "Rating"));
}

#[test]
fn exporting_the_same_solution_twice_after_a_version_bump_produces_two_listable_releases() {
    let (conn, ws, admin) = setup_workspace();
    let obj = custom_object_service::create(&conn, &ws, &object_input("Vendor", "🏢"), Some(&admin)).unwrap();
    let solution = solution_service::create(&conn, &ws, &solution_input("Field Ops Extensions"), Some(&admin)).unwrap();
    solution_service::add_component(&conn, &ws, &solution.id, &SolutionMemberInput { artifact_type: "custom_object".into(), metadata_id: obj.id.clone() }, Some(&admin)).unwrap();

    let v1_manifest = industry_package_service::export_solution(&conn, &ws, &solution.id, Some(&admin)).unwrap();

    solution_service::update(
        &conn,
        &ws,
        &solution.id,
        &SolutionUpdate { name: solution.name.clone(), description: solution.description.clone(), version: "2.0.0.0".into(), publisher_id: None },
        Some(&admin),
    )
    .unwrap();
    let v2_manifest = industry_package_service::export_solution(&conn, &ws, &solution.id, Some(&admin)).unwrap();

    let (prod_conn, prod_ws, prod_admin) = setup_workspace();
    industry_package_service::import_package(&prod_conn, &prod_ws, &ImportPackageInput { manifest_json: v1_manifest }, Some(&prod_admin)).unwrap();
    industry_package_service::import_package(&prod_conn, &prod_ws, &ImportPackageInput { manifest_json: v2_manifest }, Some(&prod_admin)).unwrap();

    let package_id = format!("local.solution.{}", solution.id);
    let versions = industry_package_service::list_package_versions(&prod_conn, &prod_ws, &package_id).unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version, "1.0.0.0");
    assert_eq!(versions[1].version, "2.0.0.0");
}

#[test]
fn export_solution_requires_admin() {
    let (conn, ws, admin) = setup_workspace();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let solution = solution_service::create(&conn, &ws, &solution_input("Field Ops Extensions"), Some(&admin)).unwrap();
    let err = industry_package_service::export_solution(&conn, &ws, &solution.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
