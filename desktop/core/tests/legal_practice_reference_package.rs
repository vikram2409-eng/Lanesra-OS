//! The Legal Practice reference package (`services::reference_packages::
//! legal_practice_manifest_json`) - the eighth real manifest run through
//! the Industry Data Model foundation, sequenced right after Real Estate
//! Brokerage per the dev spec. See that module's own doc comment for
//! what's included and what's deliberately left out (no "no open
//! mandatory deadlines" close check, no duplicate-party detection, no
//! 7/2/1-day deadline reminders, no archiving of open reminders on
//! matter close).

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::task_repo;
use lanesra_core::services::reference_packages::legal_practice_manifest_json;
use lanesra_core::services::{custom_field_service, custom_record_service, industry_package_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Harlow & Vance LLP".into(),
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

fn install_legal_practice(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: legal_practice_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

fn record(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    let json_text = legal_practice_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.legal_practice");
    assert_eq!(value["objects"].as_array().unwrap().len(), 6);
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_legal_practice(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.legal_practice");
    assert_eq!(installed.name, "Legal Practice");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 6);
    assert_eq!(count_of("custom_field"), 24);
    assert_eq!(count_of("relationship_definition"), 7);
    assert_eq!(count_of("business_rule"), 4);
    assert_eq!(count_of("workflow_definition"), 4);
    assert_eq!(count_of("screen_layout"), 1);
    assert_eq!(count_of("custom_report"), 1);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 0); // no seed data - see the module's own doc comment
}

#[test]
fn matter_close_rule_requires_a_closed_date() {
    let (conn, ws, admin) = setup_workspace();
    install_legal_practice(&conn, &ws, &admin);

    let matter = custom_record_service::create(&conn, &ws, &record("matter", "Smith v. Jones"), Some(&admin)).unwrap();

    let mut closing = HashMap::new();
    closing.insert("matter_stage".to_string(), "Closed".to_string());
    let err = custom_field_service::set_entity_values(&conn, "matter", &matter.id, &closing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Closed Date"));

    closing.insert("closed_date".to_string(), "2026-09-01".to_string());
    custom_field_service::set_entity_values(&conn, "matter", &matter.id, &closing, Some(&admin)).unwrap();
}

#[test]
fn time_entry_hours_rule_blocks_a_zero_hour_submission() {
    let (conn, ws, admin) = setup_workspace();
    install_legal_practice(&conn, &ws, &admin);

    let entry = custom_record_service::create(&conn, &ws, &record("matter_time_entry", "Draft entry"), Some(&admin)).unwrap();

    let mut submitting = HashMap::new();
    submitting.insert("time_status".to_string(), "Submitted".to_string());
    submitting.insert("description".to_string(), "Reviewed discovery documents".to_string());
    submitting.insert("hours".to_string(), "0".to_string());
    let err = custom_field_service::set_entity_values(&conn, "matter_time_entry", &entry.id, &submitting, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("more than zero hours"));

    submitting.insert("hours".to_string(), "2.5".to_string());
    custom_field_service::set_entity_values(&conn, "matter_time_entry", &entry.id, &submitting, Some(&admin)).unwrap();
}

#[test]
fn time_entry_description_rule_requires_a_description() {
    let (conn, ws, admin) = setup_workspace();
    install_legal_practice(&conn, &ws, &admin);

    let entry = custom_record_service::create(&conn, &ws, &record("matter_time_entry", "Draft entry"), Some(&admin)).unwrap();

    let mut submitting = HashMap::new();
    submitting.insert("time_status".to_string(), "Submitted".to_string());
    submitting.insert("hours".to_string(), "2.5".to_string());
    let err = custom_field_service::set_entity_values(&conn, "matter_time_entry", &entry.id, &submitting, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Description"));

    submitting.insert("description".to_string(), "Reviewed discovery documents".to_string());
    custom_field_service::set_entity_values(&conn, "matter_time_entry", &entry.id, &submitting, Some(&admin)).unwrap();
}

#[test]
fn deadline_complete_rule_requires_a_completed_date() {
    let (conn, ws, admin) = setup_workspace();
    install_legal_practice(&conn, &ws, &admin);

    let deadline = custom_record_service::create(&conn, &ws, &record("matter_deadline", "Response due"), Some(&admin)).unwrap();

    let mut completing = HashMap::new();
    completing.insert("deadline_status".to_string(), "Completed".to_string());
    let err = custom_field_service::set_entity_values(&conn, "matter_deadline", &deadline.id, &completing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Completed Date"));

    completing.insert("completed_date".to_string(), "2026-08-20".to_string());
    custom_field_service::set_entity_values(&conn, "matter_deadline", &deadline.id, &completing, Some(&admin)).unwrap();
}

#[test]
fn new_matter_workflow_creates_an_opening_checklist_task() {
    let (conn, ws, admin) = setup_workspace();
    install_legal_practice(&conn, &ws, &admin);

    let matter = custom_record_service::create(&conn, &ws, &record("matter", "Smith v. Jones"), Some(&admin)).unwrap();

    let tasks_before = task_repo::list(&conn, &ws).unwrap().len();
    let mut opening = custom_field_service::get_entity_values(&conn, &matter.id).unwrap();
    opening.insert("matter_stage".to_string(), "Open".to_string());
    custom_field_service::set_entity_values(&conn, "matter", &matter.id, &opening, Some(&admin)).unwrap();
    let tasks_after = task_repo::list(&conn, &ws).unwrap();
    assert_eq!(tasks_after.len(), tasks_before + 1, "opening a matter should create an opening checklist task");
}

#[test]
fn time_approved_workflow_marks_it_eligible_for_billing() {
    let (conn, ws, admin) = setup_workspace();
    install_legal_practice(&conn, &ws, &admin);

    let entry = custom_record_service::create(&conn, &ws, &record("matter_time_entry", "Research memo"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("hours".to_string(), "3".to_string());
    values.insert("description".to_string(), "Drafted research memo".to_string());
    custom_field_service::set_entity_values(&conn, "matter_time_entry", &entry.id, &values, Some(&admin)).unwrap();

    let mut approving = custom_field_service::get_entity_values(&conn, &entry.id).unwrap();
    approving.insert("time_status".to_string(), "Approved".to_string());
    custom_field_service::set_entity_values(&conn, "matter_time_entry", &entry.id, &approving, Some(&admin)).unwrap();

    let updated = custom_field_service::get_entity_values(&conn, &entry.id).unwrap();
    assert_eq!(updated.get("billing_status").map(String::as_str), Some("Eligible"), "approving time should mark it eligible for billing");
}

#[test]
fn matter_closing_and_closed_workflows_create_their_own_tasks() {
    let (conn, ws, admin) = setup_workspace();
    install_legal_practice(&conn, &ws, &admin);

    let matter = custom_record_service::create(&conn, &ws, &record("matter", "Smith v. Jones"), Some(&admin)).unwrap();

    let tasks_before_closing = task_repo::list(&conn, &ws).unwrap().len();
    let mut closing = custom_field_service::get_entity_values(&conn, &matter.id).unwrap();
    closing.insert("matter_stage".to_string(), "Closing".to_string());
    custom_field_service::set_entity_values(&conn, "matter", &matter.id, &closing, Some(&admin)).unwrap();
    let tasks_after_closing = task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(tasks_after_closing, tasks_before_closing + 1, "moving a matter to Closing should create a closing checklist task");

    let mut closed = custom_field_service::get_entity_values(&conn, &matter.id).unwrap();
    closed.insert("matter_stage".to_string(), "Closed".to_string());
    closed.insert("closed_date".to_string(), "2026-09-01".to_string());
    custom_field_service::set_entity_values(&conn, "matter", &matter.id, &closed, Some(&admin)).unwrap();
    let tasks_after_closed = task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(tasks_after_closed, tasks_after_closing + 1, "closing a matter should create a final billing review task");
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

    let input = ImportPackageInput { manifest_json: legal_practice_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
