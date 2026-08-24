//! The Dental/Clinic Practice Administration reference package
//! (`services::reference_packages::practice_admin_manifest_json`) - the
//! fifth real manifest run through the Industry Data Model foundation,
//! sequenced right after Professional Services per the dev spec. See
//! that module's own doc comment for what's included and what's
//! deliberately left out (no schedule-collision detection, no
//! relationship-existence business rules, no recall-due reminders, no
//! real time-of-day field type).

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::reference_packages::practice_admin_manifest_json;
use lanesra_core::services::{custom_field_service, custom_record_service, industry_package_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Brightsmile Dental".into(),
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

fn install_practice_admin(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: practice_admin_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

fn record(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    let json_text = practice_admin_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.practice_admin");
    assert_eq!(value["objects"].as_array().unwrap().len(), 8);
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_practice_admin(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.practice_admin");
    assert_eq!(installed.name, "Practice Administration");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 8);
    assert_eq!(count_of("custom_field"), 29);
    assert_eq!(count_of("relationship_definition"), 12);
    assert_eq!(count_of("business_rule"), 3);
    assert_eq!(count_of("workflow_definition"), 4);
    assert_eq!(count_of("screen_layout"), 1);
    assert_eq!(count_of("custom_report"), 1);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 0); // no seed data - see the module's own doc comment
}

#[test]
fn appointment_start_time_and_duration_are_required() {
    let (conn, ws, admin) = setup_workspace();
    install_practice_admin(&conn, &ws, &admin);

    let appt = custom_record_service::create(&conn, &ws, &record("appointment", "Cleaning"), Some(&admin)).unwrap();
    let mut incomplete = HashMap::new();
    incomplete.insert("appt_date".to_string(), "2026-08-20".to_string());
    let err = custom_field_service::set_entity_values(&conn, "appointment", &appt.id, &incomplete, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Start Time") || err.to_string().contains("Duration"));

    incomplete.insert("start_time_text".to_string(), "9:00 AM".to_string());
    incomplete.insert("duration_minutes".to_string(), "30".to_string());
    custom_field_service::set_entity_values(&conn, "appointment", &appt.id, &incomplete, Some(&admin)).unwrap();
}

#[test]
fn complete_appointment_rule_requires_a_completed_date() {
    let (conn, ws, admin) = setup_workspace();
    install_practice_admin(&conn, &ws, &admin);

    let appt = custom_record_service::create(&conn, &ws, &record("appointment", "Cleaning"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("appt_date".to_string(), "2026-08-20".to_string());
    values.insert("start_time_text".to_string(), "9:00 AM".to_string());
    values.insert("duration_minutes".to_string(), "30".to_string());
    custom_field_service::set_entity_values(&conn, "appointment", &appt.id, &values, Some(&admin)).unwrap();

    let mut completing = custom_field_service::get_entity_values(&conn, &appt.id).unwrap();
    completing.insert("stage".to_string(), "Completed".to_string());
    let err = custom_field_service::set_entity_values(&conn, "appointment", &appt.id, &completing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Completed Date"));

    completing.insert("completed_date".to_string(), "2026-08-20".to_string());
    custom_field_service::set_entity_values(&conn, "appointment", &appt.id, &completing, Some(&admin)).unwrap();
}

#[test]
fn appointment_confirmation_workflow_creates_a_task_on_creation() {
    let (conn, ws, admin) = setup_workspace();
    install_practice_admin(&conn, &ws, &admin);

    let tasks_before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    custom_record_service::create(&conn, &ws, &record("appointment", "Cleaning"), Some(&admin)).unwrap();
    let tasks_after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(tasks_after, tasks_before + 1, "creating an appointment should create a confirmation task for reception");
}

#[test]
fn no_show_workflow_creates_a_follow_up_task() {
    let (conn, ws, admin) = setup_workspace();
    install_practice_admin(&conn, &ws, &admin);

    let appt = custom_record_service::create(&conn, &ws, &record("appointment", "Cleaning"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("appt_date".to_string(), "2026-08-20".to_string());
    values.insert("start_time_text".to_string(), "9:00 AM".to_string());
    values.insert("duration_minutes".to_string(), "30".to_string());
    custom_field_service::set_entity_values(&conn, "appointment", &appt.id, &values, Some(&admin)).unwrap();

    let tasks_before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    let mut no_show = custom_field_service::get_entity_values(&conn, &appt.id).unwrap();
    no_show.insert("stage".to_string(), "No Show".to_string());
    custom_field_service::set_entity_values(&conn, "appointment", &appt.id, &no_show, Some(&admin)).unwrap();
    let tasks_after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(tasks_after, tasks_before + 1, "marking an appointment No Show should create a follow-up task for reception");
}

#[test]
fn treatment_accepted_workflow_creates_a_billing_prep_task() {
    let (conn, ws, admin) = setup_workspace();
    install_practice_admin(&conn, &ws, &admin);

    let plan = custom_record_service::create(&conn, &ws, &record("treatment_plan", "Crown and filling"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("plan_date".to_string(), "2026-08-15".to_string());
    custom_field_service::set_entity_values(&conn, "treatment_plan", &plan.id, &values, Some(&admin)).unwrap();

    let tasks_before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    let mut accepting = custom_field_service::get_entity_values(&conn, &plan.id).unwrap();
    accepting.insert("stage".to_string(), "Accepted".to_string());
    custom_field_service::set_entity_values(&conn, "treatment_plan", &plan.id, &accepting, Some(&admin)).unwrap();

    let tasks_after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(tasks_after, tasks_before + 1, "accepting a treatment plan should create a billing-preparation task");
}

#[test]
fn claim_payment_rule_requires_a_paid_amount_and_payment_date() {
    let (conn, ws, admin) = setup_workspace();
    install_practice_admin(&conn, &ws, &admin);

    let claim = custom_record_service::create(&conn, &ws, &record("billing_claim", "Cleaning claim"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("claim_status".to_string(), "Paid".to_string());
    let err = custom_field_service::set_entity_values(&conn, "billing_claim", &claim.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Paid Amount") || err.to_string().contains("Payment Date"));

    values.insert("paid_amount".to_string(), "180".to_string());
    values.insert("payment_date".to_string(), "2026-08-22".to_string());
    custom_field_service::set_entity_values(&conn, "billing_claim", &claim.id, &values, Some(&admin)).unwrap();
}

#[test]
fn claim_denial_notes_rule_requires_a_denial_reason() {
    let (conn, ws, admin) = setup_workspace();
    install_practice_admin(&conn, &ws, &admin);

    let claim = custom_record_service::create(&conn, &ws, &record("billing_claim", "Cleaning claim"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("claim_status".to_string(), "Denied".to_string());
    let err = custom_field_service::set_entity_values(&conn, "billing_claim", &claim.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Denial Reason"));

    values.insert("denial_reason".to_string(), "Missing pre-authorization".to_string());
    custom_field_service::set_entity_values(&conn, "billing_claim", &claim.id, &values, Some(&admin)).unwrap();
}

#[test]
fn billing_claim_submitted_workflow_creates_a_tracking_task() {
    let (conn, ws, admin) = setup_workspace();
    install_practice_admin(&conn, &ws, &admin);

    let claim = custom_record_service::create(&conn, &ws, &record("billing_claim", "Cleaning claim"), Some(&admin)).unwrap();
    let tasks_before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();

    let mut values = HashMap::new();
    values.insert("claim_status".to_string(), "Submitted".to_string());
    custom_field_service::set_entity_values(&conn, "billing_claim", &claim.id, &values, Some(&admin)).unwrap();

    let tasks_after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(tasks_after, tasks_before + 1, "the 'Billing claim submitted' workflow should have created a tracking task");
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

    let input = ImportPackageInput { manifest_json: practice_admin_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
