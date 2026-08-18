//! The Recruitment & Staffing reference package (`services::
//! reference_packages::recruitment_manifest_json`) - the sixth real
//! manifest run through the Industry Data Model foundation, sequenced
//! right after Dental/Clinic Practice Administration per the dev spec.
//! See that module's own doc comment for what's included and what's
//! deliberately left out (no duplicate-candidate/application detection,
//! no "must be in the future" interview validation, no aggregate
//! job-filled automation).

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::notification_repo;
use lanesra_core::services::reference_packages::recruitment_manifest_json;
use lanesra_core::services::{custom_field_service, custom_record_service, industry_package_service, relationship_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Northgate Staffing".into(),
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

fn install_recruitment(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: recruitment_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

fn record(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    let json_text = recruitment_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.recruitment");
    assert_eq!(value["objects"].as_array().unwrap().len(), 8);
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_recruitment(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.recruitment");
    assert_eq!(installed.name, "Recruitment & Staffing");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 8);
    assert_eq!(count_of("custom_field"), 28);
    assert_eq!(count_of("relationship_definition"), 14);
    assert_eq!(count_of("business_rule"), 1);
    assert_eq!(count_of("workflow_definition"), 3);
    assert_eq!(count_of("screen_layout"), 1);
    assert_eq!(count_of("custom_report"), 1);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 0); // no seed data - see the module's own doc comment
}

#[test]
fn placement_start_rule_requires_a_start_date() {
    let (conn, ws, admin) = setup_workspace();
    install_recruitment(&conn, &ws, &admin);

    let placement = custom_record_service::create(&conn, &ws, &record("placement", "Backend Engineer @ Acme"), Some(&admin)).unwrap();

    let mut activating = HashMap::new();
    activating.insert("stage".to_string(), "Active".to_string());
    let err = custom_field_service::set_entity_values(&conn, "placement", &placement.id, &activating, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Start Date"));

    activating.insert("start_date".to_string(), "2026-09-01".to_string());
    custom_field_service::set_entity_values(&conn, "placement", &placement.id, &activating, Some(&admin)).unwrap();
}

#[test]
fn application_interview_stage_workflow_creates_a_scheduling_task() {
    let (conn, ws, admin) = setup_workspace();
    install_recruitment(&conn, &ws, &admin);

    let application = custom_record_service::create(&conn, &ws, &record("application", "Jane Doe -> Backend Engineer"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("score".to_string(), "80".to_string());
    custom_field_service::set_entity_values(&conn, "application", &application.id, &values, Some(&admin)).unwrap();

    let tasks_before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    let mut interviewing = custom_field_service::get_entity_values(&conn, &application.id).unwrap();
    interviewing.insert("stage".to_string(), "Interview".to_string());
    custom_field_service::set_entity_values(&conn, "application", &application.id, &interviewing, Some(&admin)).unwrap();
    let tasks_after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(tasks_after, tasks_before + 1, "moving an application to Interview should create a scheduling task");
}

#[test]
fn interview_scheduled_workflow_notifies_admins() {
    let (conn, ws, admin) = setup_workspace();
    install_recruitment(&conn, &ws, &admin);

    let notifications_before = notification_repo::list_for_user(&conn, &ws, &admin, false).unwrap().len();
    custom_record_service::create(&conn, &ws, &record("interview", "Screen with Jane Doe"), Some(&admin)).unwrap();
    let notifications_after = notification_repo::list_for_user(&conn, &ws, &admin, false).unwrap().len();
    assert_eq!(notifications_after, notifications_before + 1, "creating an interview should notify admins");
}

#[test]
fn offer_accepted_workflow_places_the_application_and_drafts_a_placement() {
    let (conn, ws, admin) = setup_workspace();
    install_recruitment(&conn, &ws, &admin);

    let application = custom_record_service::create(&conn, &ws, &record("application", "Jane Doe -> Backend Engineer"), Some(&admin)).unwrap();
    let offer = custom_record_service::create(&conn, &ws, &record("offer", "Offer to Jane Doe"), Some(&admin)).unwrap();

    let relationships = relationship_service::list(&conn, &ws, true).unwrap();
    let offer_to_application = relationships.iter().find(|r| r.source_entity_type == "offer" && r.target_entity_type == "application").expect("the manifest defines an offer -> application relationship");
    relationship_service::link(&conn, &ws, &offer_to_application.id, "offer", &offer.id, "application", &application.id, Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert("amount".to_string(), "95000".to_string());
    custom_field_service::set_entity_values(&conn, "offer", &offer.id, &values, Some(&admin)).unwrap();

    let placements_before = custom_record_service::list(&conn, &ws, "placement").unwrap().len();
    let mut accepting = custom_field_service::get_entity_values(&conn, &offer.id).unwrap();
    accepting.insert("stage".to_string(), "Accepted".to_string());
    custom_field_service::set_entity_values(&conn, "offer", &offer.id, &accepting, Some(&admin)).unwrap();

    let application_values = custom_field_service::get_entity_values(&conn, &application.id).unwrap();
    assert_eq!(application_values.get("stage").map(String::as_str), Some("Placed"), "accepting the offer should move the linked application to Placed");

    let placements_after = custom_record_service::list(&conn, &ws, "placement").unwrap();
    assert_eq!(placements_after.len(), placements_before + 1, "accepting the offer should draft a Placement");

    let placement = placements_after.into_iter().next().unwrap();
    let linked = relationship_service::related_records_for(&conn, &ws, "placement", &placement.id).unwrap();
    assert!(
        linked.iter().any(|r| r.entity_type == "offer" && r.entity_id == offer.id),
        "the drafted placement should be linked back to the accepted offer"
    );
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

    let input = ImportPackageInput { manifest_json: recruitment_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
