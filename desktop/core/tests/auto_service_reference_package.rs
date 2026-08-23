//! The Auto Repair & Service Garage reference package (`services::
//! reference_packages::auto_service_manifest_json`) - the tenth and final
//! real manifest run through the Industry Data Model foundation,
//! sequenced right after Nonprofit & Association per the dev spec. See
//! that module's own doc comment for what's included and what's
//! deliberately left out (no "require owner/customer" relationship-
//! existence check on Vehicle, no zero-price-permission exception on
//! Authorization, no recommendation-due reminder, no vehicle service
//! history rollup, no "if no line assignment" qualifier on the
//! technician task, no field-value copying on the check-in workflow).

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::task_repo;
use lanesra_core::services::reference_packages::auto_service_manifest_json;
use lanesra_core::services::{custom_field_service, custom_record_service, industry_package_service, relationship_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Overlook Auto Service".into(),
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

fn install_auto_service(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: auto_service_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

fn record(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    let json_text = auto_service_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.auto_service");
    assert_eq!(value["objects"].as_array().unwrap().len(), 7);
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_auto_service(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.auto_service");
    assert_eq!(installed.name, "Auto Repair & Service Garage");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 7);
    assert_eq!(count_of("custom_field"), 35);
    assert_eq!(count_of("relationship_definition"), 11);
    assert_eq!(count_of("business_rule"), 4);
    assert_eq!(count_of("workflow_definition"), 5);
    assert_eq!(count_of("screen_layout"), 1);
    assert_eq!(count_of("custom_report"), 1);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 0); // no seed data - see the module's own doc comment
}

#[test]
fn repair_completion_rule_requires_completion_date_and_odometer_out() {
    let (conn, ws, admin) = setup_workspace();
    install_auto_service(&conn, &ws, &admin);

    let ro = custom_record_service::create(&conn, &ws, &record("repair_order", "RO for Jordan Lee's Civic"), Some(&admin)).unwrap();

    let mut completing = HashMap::new();
    completing.insert("ro_stage".to_string(), "Completed".to_string());
    let err = custom_field_service::set_entity_values(&conn, "repair_order", &ro.id, &completing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Completion Date") || err.to_string().contains("Odometer Out"));

    completing.insert("completion_date".to_string(), "2026-08-19".to_string());
    completing.insert("odometer_out".to_string(), "45210".to_string());
    custom_field_service::set_entity_values(&conn, "repair_order", &ro.id, &completing, Some(&admin)).unwrap();
}

#[test]
fn authorization_rule_requires_a_price() {
    let (conn, ws, admin) = setup_workspace();
    install_auto_service(&conn, &ws, &admin);

    let line = custom_record_service::create(&conn, &ws, &record("repair_line", "Front brake pads"), Some(&admin)).unwrap();

    let mut authorizing = HashMap::new();
    authorizing.insert("line_stage".to_string(), "Authorized".to_string());
    let err = custom_field_service::set_entity_values(&conn, "repair_line", &line.id, &authorizing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Price"));

    authorizing.insert("price".to_string(), "185.00".to_string());
    custom_field_service::set_entity_values(&conn, "repair_line", &line.id, &authorizing, Some(&admin)).unwrap();
}

#[test]
fn odometer_validation_rule_blocks_out_less_than_in() {
    let (conn, ws, admin) = setup_workspace();
    install_auto_service(&conn, &ws, &admin);

    let ro = custom_record_service::create(&conn, &ws, &record("repair_order", "RO for Jordan Lee's Civic"), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert("odometer_in".to_string(), "45200".to_string());
    values.insert("odometer_out".to_string(), "45100".to_string());
    let err = custom_field_service::set_entity_values(&conn, "repair_order", &ro.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Odometer out cannot be less than odometer in"));

    values.insert("odometer_out".to_string(), "45260".to_string());
    custom_field_service::set_entity_values(&conn, "repair_order", &ro.id, &values, Some(&admin)).unwrap();
}

#[test]
fn appointment_check_in_workflow_creates_and_links_a_repair_order() {
    let (conn, ws, admin) = setup_workspace();
    install_auto_service(&conn, &ws, &admin);

    let appt = custom_record_service::create(&conn, &ws, &record("vehicle_appointment", "Jordan Lee - oil change"), Some(&admin)).unwrap();

    let mut checking_in = custom_field_service::get_entity_values(&conn, &appt.id).unwrap();
    checking_in.insert("appt_stage".to_string(), "Checked In".to_string());
    custom_field_service::set_entity_values(&conn, "vehicle_appointment", &appt.id, &checking_in, Some(&admin)).unwrap();

    let linked = relationship_service::related_records_for(&conn, &ws, "vehicle_appointment", &appt.id).unwrap();
    let linked_ros: Vec<_> = linked.iter().filter(|r| r.entity_type == "repair_order").collect();
    assert_eq!(linked_ros.len(), 1, "checking in an appointment should create exactly one linked repair order");
}

#[test]
fn repair_authorized_workflow_creates_a_technician_assignment_task() {
    let (conn, ws, admin) = setup_workspace();
    install_auto_service(&conn, &ws, &admin);

    let ro = custom_record_service::create(&conn, &ws, &record("repair_order", "RO for Jordan Lee's Civic"), Some(&admin)).unwrap();

    let tasks_before = task_repo::list(&conn, &ws).unwrap().len();
    let mut authorizing = custom_field_service::get_entity_values(&conn, &ro.id).unwrap();
    authorizing.insert("ro_stage".to_string(), "Authorized".to_string());
    custom_field_service::set_entity_values(&conn, "repair_order", &ro.id, &authorizing, Some(&admin)).unwrap();
    let tasks_after = task_repo::list(&conn, &ws).unwrap();
    assert_eq!(tasks_after.len(), tasks_before + 1, "authorizing a repair order should create a technician assignment task");
}

#[test]
fn repair_completed_workflow_closes_the_originating_appointment() {
    let (conn, ws, admin) = setup_workspace();
    install_auto_service(&conn, &ws, &admin);

    let appt = custom_record_service::create(&conn, &ws, &record("vehicle_appointment", "Jordan Lee - oil change"), Some(&admin)).unwrap();
    let mut checking_in = custom_field_service::get_entity_values(&conn, &appt.id).unwrap();
    checking_in.insert("appt_stage".to_string(), "Checked In".to_string());
    custom_field_service::set_entity_values(&conn, "vehicle_appointment", &appt.id, &checking_in, Some(&admin)).unwrap();

    let linked = relationship_service::related_records_for(&conn, &ws, "vehicle_appointment", &appt.id).unwrap();
    let ro_id = linked.iter().find(|r| r.entity_type == "repair_order").expect("check-in should have created a repair order").entity_id.clone();

    let mut completing = custom_field_service::get_entity_values(&conn, &ro_id).unwrap();
    completing.insert("completion_date".to_string(), "2026-08-19".to_string());
    completing.insert("odometer_out".to_string(), "45260".to_string());
    completing.insert("ro_stage".to_string(), "Completed".to_string());
    custom_field_service::set_entity_values(&conn, "repair_order", &ro_id, &completing, Some(&admin)).unwrap();

    let appt_after = custom_field_service::get_entity_values(&conn, &appt.id).unwrap();
    assert_eq!(appt_after.get("appt_stage").map(String::as_str), Some("Completed"), "completing the repair order should close its originating appointment");
}

#[test]
fn no_show_workflow_creates_a_reschedule_task() {
    let (conn, ws, admin) = setup_workspace();
    install_auto_service(&conn, &ws, &admin);

    let appt = custom_record_service::create(&conn, &ws, &record("vehicle_appointment", "Jordan Lee - oil change"), Some(&admin)).unwrap();

    let tasks_before = task_repo::list(&conn, &ws).unwrap().len();
    let mut no_show = custom_field_service::get_entity_values(&conn, &appt.id).unwrap();
    no_show.insert("appt_stage".to_string(), "No Show".to_string());
    custom_field_service::set_entity_values(&conn, "vehicle_appointment", &appt.id, &no_show, Some(&admin)).unwrap();
    let tasks_after = task_repo::list(&conn, &ws).unwrap();
    assert_eq!(tasks_after.len(), tasks_before + 1, "a no-show appointment should create a reschedule task");
}

#[test]
fn parts_order_receipt_rule_requires_a_received_date() {
    let (conn, ws, admin) = setup_workspace();
    install_auto_service(&conn, &ws, &admin);

    let order = custom_record_service::create(&conn, &ws, &record("parts_order", "Front brake pads order"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("part_name".to_string(), "Front brake pads".to_string());
    custom_field_service::set_entity_values(&conn, "parts_order", &order.id, &values, Some(&admin)).unwrap();

    values.insert("order_status".to_string(), "Received".to_string());
    let err = custom_field_service::set_entity_values(&conn, "parts_order", &order.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Received Date"));

    values.insert("received_date".to_string(), "2026-08-21".to_string());
    custom_field_service::set_entity_values(&conn, "parts_order", &order.id, &values, Some(&admin)).unwrap();
}

#[test]
fn parts_backordered_workflow_creates_a_follow_up_task() {
    let (conn, ws, admin) = setup_workspace();
    install_auto_service(&conn, &ws, &admin);

    let order = custom_record_service::create(&conn, &ws, &record("parts_order", "Front brake pads order"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("part_name".to_string(), "Front brake pads".to_string());
    custom_field_service::set_entity_values(&conn, "parts_order", &order.id, &values, Some(&admin)).unwrap();

    let tasks_before = task_repo::list(&conn, &ws).unwrap().len();
    values.insert("order_status".to_string(), "Backordered".to_string());
    custom_field_service::set_entity_values(&conn, "parts_order", &order.id, &values, Some(&admin)).unwrap();
    let tasks_after = task_repo::list(&conn, &ws).unwrap();
    assert_eq!(tasks_after.len(), tasks_before + 1, "a backordered part should create a follow-up task");
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

    let input = ImportPackageInput { manifest_json: auto_service_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
