//! The Property Management reference package (`services::reference_packages::
//! property_management_manifest_json`) - the second real manifest run
//! through the Industry Data Model foundation, sequenced right after
//! Field Service per the dev spec. See that module's own doc comment for
//! what's included and what's deliberately left out (no Field Service
//! integration, no lease-overlap/occupancy cross-record checks, no
//! date-triggered workflows).

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::contact::ContactInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::reference_packages::property_management_manifest_json;
use lanesra_core::services::{company_service, contact_service, custom_field_service, custom_record_service, industry_package_service, relationship_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Acme Property Group".into(),
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

fn install_property_management(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: property_management_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

fn record(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    let json_text = property_management_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.property_management");
    assert_eq!(value["objects"].as_array().unwrap().len(), 9);
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_property_management(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.property_management");
    assert_eq!(installed.name, "Property Management");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 9);
    assert_eq!(count_of("custom_field"), 34);
    assert_eq!(count_of("relationship_definition"), 13);
    assert_eq!(count_of("business_rule"), 3);
    assert_eq!(count_of("workflow_definition"), 4);
    assert_eq!(count_of("screen_layout"), 1);
    assert_eq!(count_of("custom_report"), 1);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 0); // no seed data - see the module's own doc comment
}

#[test]
fn lease_date_validation_blocks_an_end_date_on_or_before_the_start_date() {
    let (conn, ws, admin) = setup_workspace();
    install_property_management(&conn, &ws, &admin);

    let lease = custom_record_service::create(&conn, &ws, &record("lease", "Unit 4B Lease"), Some(&admin)).unwrap();

    let mut backwards = HashMap::new();
    backwards.insert("start_date".to_string(), "2026-06-01".to_string());
    backwards.insert("end_date".to_string(), "2026-01-01".to_string());
    let err = custom_field_service::set_entity_values(&conn, "lease", &lease.id, &backwards, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("end date"));

    let mut valid = HashMap::new();
    valid.insert("start_date".to_string(), "2026-01-01".to_string());
    valid.insert("end_date".to_string(), "2026-12-31".to_string());
    custom_field_service::set_entity_values(&conn, "lease", &lease.id, &valid, Some(&admin)).unwrap();
}

#[test]
fn maintenance_closure_rule_requires_resolution_and_completed_date() {
    let (conn, ws, admin) = setup_workspace();
    install_property_management(&conn, &ws, &admin);

    let request = custom_record_service::create(&conn, &ws, &record("maintenance_request", "Leaking faucet"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("description".to_string(), "Kitchen faucet leaking".to_string());
    custom_field_service::set_entity_values(&conn, "maintenance_request", &request.id, &values, Some(&admin)).unwrap();

    let mut closing = custom_field_service::get_entity_values(&conn, &request.id).unwrap();
    closing.insert("stage".to_string(), "Closed".to_string());
    let err = custom_field_service::set_entity_values(&conn, "maintenance_request", &request.id, &closing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Resolution") || err.to_string().contains("Completed Date"));

    closing.insert("resolution".to_string(), "Replaced the washer".to_string());
    closing.insert("completed_date".to_string(), "2026-01-20".to_string());
    custom_field_service::set_entity_values(&conn, "maintenance_request", &request.id, &closing, Some(&admin)).unwrap();
}

#[test]
fn lease_activation_and_termination_workflows_flip_the_units_occupancy() {
    let (conn, ws, admin) = setup_workspace();
    install_property_management(&conn, &ws, &admin);

    let unit = custom_record_service::create(&conn, &ws, &record("unit", "Unit 4B"), Some(&admin)).unwrap();
    let lease = custom_record_service::create(&conn, &ws, &record("lease", "Unit 4B Lease"), Some(&admin)).unwrap();

    let relationships = relationship_service::list(&conn, &ws, true).unwrap();
    let lease_to_unit = relationships
        .iter()
        .find(|r| r.source_entity_type == "lease" && r.target_entity_type == "unit")
        .expect("the manifest defines a lease -> unit relationship");
    relationship_service::link(&conn, &ws, &lease_to_unit.id, "lease", &lease.id, "unit", &unit.id, Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert("start_date".to_string(), "2026-01-01".to_string());
    values.insert("end_date".to_string(), "2026-12-31".to_string());
    custom_field_service::set_entity_values(&conn, "lease", &lease.id, &values, Some(&admin)).unwrap();

    let mut activate = custom_field_service::get_entity_values(&conn, &lease.id).unwrap();
    activate.insert("stage".to_string(), "Active".to_string());
    custom_field_service::set_entity_values(&conn, "lease", &lease.id, &activate, Some(&admin)).unwrap();
    let unit_values = custom_field_service::get_entity_values(&conn, &unit.id).unwrap();
    assert_eq!(unit_values.get("unit_stage").map(String::as_str), Some("Occupied"));

    let mut terminate = custom_field_service::get_entity_values(&conn, &lease.id).unwrap();
    terminate.insert("stage".to_string(), "Terminated".to_string());
    custom_field_service::set_entity_values(&conn, "lease", &lease.id, &terminate, Some(&admin)).unwrap();
    let unit_values = custom_field_service::get_entity_values(&conn, &unit.id).unwrap();
    assert_eq!(unit_values.get("unit_stage").map(String::as_str), Some("Vacant"));
}

#[test]
fn maintenance_intake_workflow_creates_a_coordinator_task() {
    let (conn, ws, admin) = setup_workspace();
    install_property_management(&conn, &ws, &admin);

    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    custom_record_service::create(&conn, &ws, &record("maintenance_request", "Leaking faucet"), Some(&admin)).unwrap();
    let after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after, before + 1, "the 'Maintenance intake' workflow should have created a coordinator task");
}

#[test]
fn showing_completion_rule_requires_an_interest_level_before_marking_a_showing_complete() {
    let (conn, ws, admin) = setup_workspace();
    install_property_management(&conn, &ws, &admin);

    let unit = custom_record_service::create(&conn, &ws, &record("unit", "Unit 4B"), Some(&admin)).unwrap();
    let company = company_service::create(
        &conn, &ws,
        &CompanyInput { name: "Prospect Household".into(), status: "Prospect".into(), ..Default::default() },
        Some(&admin),
    ).unwrap();
    let prospect = contact_service::create(
        &conn,
        &ContactInput { company_id: company.id.clone(), first_name: "Jordan".into(), last_name: "Prospect".into(), status: "Active".into(), ..Default::default() },
        Some(&admin),
    ).unwrap();
    let showing = custom_record_service::create(&conn, &ws, &record("unit_showing", "Unit 4B Showing"), Some(&admin)).unwrap();

    let relationships = relationship_service::list(&conn, &ws, true).unwrap();
    let showing_to_unit = relationships
        .iter()
        .find(|r| r.source_entity_type == "unit_showing" && r.target_entity_type == "unit")
        .expect("the manifest defines a unit_showing -> unit relationship");
    relationship_service::link(&conn, &ws, &showing_to_unit.id, "unit_showing", &showing.id, "unit", &unit.id, Some(&admin)).unwrap();
    let showing_to_contact = relationships
        .iter()
        .find(|r| r.source_entity_type == "unit_showing" && r.target_entity_type == "Contact")
        .expect("the manifest defines a unit_showing -> Contact relationship");
    relationship_service::link(&conn, &ws, &showing_to_contact.id, "unit_showing", &showing.id, "Contact", &prospect.id, Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert("showing_stage".to_string(), "Completed".to_string());
    let err = custom_field_service::set_entity_values(&conn, "unit_showing", &showing.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Interest Level"));

    values.insert("interest_level".to_string(), "Medium".to_string());
    custom_field_service::set_entity_values(&conn, "unit_showing", &showing.id, &values, Some(&admin)).unwrap();

    let stored = custom_field_service::get_entity_values(&conn, &showing.id).unwrap();
    assert_eq!(stored.get("showing_stage").map(String::as_str), Some("Completed"));
}

#[test]
fn high_interest_showing_follow_up_workflow_creates_a_task_only_when_both_conditions_are_met() {
    let (conn, ws, admin) = setup_workspace();
    install_property_management(&conn, &ws, &admin);

    let showing = custom_record_service::create(&conn, &ws, &record("unit_showing", "Unit 4B Showing"), Some(&admin)).unwrap();

    // Completed but only Medium interest - the workflow's second condition
    // isn't met, so no follow-up task should be created. The trigger field
    // (showing_stage) is set in its own call, after interest_level is
    // already on file, mirroring how the other field_changed-triggered
    // tests in this file establish a baseline before the field that
    // actually flips.
    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    let mut values = HashMap::new();
    values.insert("interest_level".to_string(), "Medium".to_string());
    custom_field_service::set_entity_values(&conn, "unit_showing", &showing.id, &values, Some(&admin)).unwrap();
    values.insert("showing_stage".to_string(), "Completed".to_string());
    custom_field_service::set_entity_values(&conn, "unit_showing", &showing.id, &values, Some(&admin)).unwrap();
    let after_medium = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after_medium, before, "a Medium-interest showing should not trigger the follow-up workflow");

    // Bumping interest to High and completing a different showing proves
    // the workflow fires when both conditions hold together at the point
    // showing_stage actually changes.
    let showing2 = custom_record_service::create(&conn, &ws, &record("unit_showing", "Unit 5A Showing"), Some(&admin)).unwrap();
    let mut high_values = HashMap::new();
    high_values.insert("interest_level".to_string(), "High".to_string());
    custom_field_service::set_entity_values(&conn, "unit_showing", &showing2.id, &high_values, Some(&admin)).unwrap();
    high_values.insert("showing_stage".to_string(), "Completed".to_string());
    custom_field_service::set_entity_values(&conn, "unit_showing", &showing2.id, &high_values, Some(&admin)).unwrap();
    let after_high = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after_high, after_medium + 1, "the 'High-interest showing follow-up' workflow should have created a task");
}

#[test]
fn a_non_administrator_cannot_install_the_reference_package() {
    let (conn, ws, admin) = setup_workspace();
    let sales = user_service::create(
        &conn, &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    )
    .unwrap()
    .id;

    let input = ImportPackageInput { manifest_json: property_management_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
