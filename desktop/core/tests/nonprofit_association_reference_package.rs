//! The Nonprofit & Association reference package (`services::
//! reference_packages::nonprofit_association_manifest_json`) - the ninth
//! real manifest run through the Industry Data Model foundation,
//! sequenced right after Legal Practice per the dev spec. See that
//! module's own doc comment for what's included and what's deliberately
//! left out (no capacity-vs-registration-count check, no membership
//! renewal/expiry reminders, no campaign-total aggregation).

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::task_repo;
use lanesra_core::services::reference_packages::nonprofit_association_manifest_json;
use lanesra_core::services::{custom_field_service, custom_record_service, industry_package_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Riverbend Community Alliance".into(),
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

fn install_nonprofit(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: nonprofit_association_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

fn record(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    let json_text = nonprofit_association_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.nonprofit_association");
    assert_eq!(value["objects"].as_array().unwrap().len(), 9);
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_nonprofit(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.nonprofit_association");
    assert_eq!(installed.name, "Nonprofit & Association");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 9);
    assert_eq!(count_of("custom_field"), 31);
    assert_eq!(count_of("relationship_definition"), 11);
    assert_eq!(count_of("business_rule"), 3);
    assert_eq!(count_of("workflow_definition"), 2);
    assert_eq!(count_of("screen_layout"), 1);
    assert_eq!(count_of("custom_report"), 1);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 0); // no seed data - see the module's own doc comment
}

#[test]
fn membership_activation_rule_requires_start_and_end_dates() {
    let (conn, ws, admin) = setup_workspace();
    install_nonprofit(&conn, &ws, &admin);

    let membership = custom_record_service::create(&conn, &ws, &record("membership", "Jordan Lee - Annual"), Some(&admin)).unwrap();

    let mut activating = HashMap::new();
    activating.insert("membership_stage".to_string(), "Active".to_string());
    let err = custom_field_service::set_entity_values(&conn, "membership", &membership.id, &activating, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Start Date") || err.to_string().contains("End Date"));

    activating.insert("start_date".to_string(), "2026-01-01".to_string());
    activating.insert("end_date".to_string(), "2026-12-31".to_string());
    custom_field_service::set_entity_values(&conn, "membership", &membership.id, &activating, Some(&admin)).unwrap();
}

#[test]
fn donation_validation_rule_blocks_a_zero_amount() {
    let (conn, ws, admin) = setup_workspace();
    install_nonprofit(&conn, &ws, &admin);

    let donation = custom_record_service::create(&conn, &ws, &record("donation", "Gift from Jordan Lee"), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert("amount".to_string(), "0".to_string());
    let err = custom_field_service::set_entity_values(&conn, "donation", &donation.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("greater than zero"));

    values.insert("amount".to_string(), "50".to_string());
    custom_field_service::set_entity_values(&conn, "donation", &donation.id, &values, Some(&admin)).unwrap();
}

#[test]
fn renewal_integrity_rule_blocks_a_renewal_date_before_the_start_date() {
    let (conn, ws, admin) = setup_workspace();
    install_nonprofit(&conn, &ws, &admin);

    let membership = custom_record_service::create(&conn, &ws, &record("membership", "Jordan Lee - Annual"), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert("start_date".to_string(), "2026-01-01".to_string());
    values.insert("renewal_date".to_string(), "2025-12-01".to_string());
    let err = custom_field_service::set_entity_values(&conn, "membership", &membership.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Renewal date"));

    values.insert("renewal_date".to_string(), "2026-11-01".to_string());
    custom_field_service::set_entity_values(&conn, "membership", &membership.id, &values, Some(&admin)).unwrap();
}

#[test]
fn donation_received_workflow_creates_an_acknowledgement_task() {
    let (conn, ws, admin) = setup_workspace();
    install_nonprofit(&conn, &ws, &admin);

    let donation = custom_record_service::create(&conn, &ws, &record("donation", "Gift from Jordan Lee"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("amount".to_string(), "100".to_string());
    custom_field_service::set_entity_values(&conn, "donation", &donation.id, &values, Some(&admin)).unwrap();

    let tasks_before = task_repo::list(&conn, &ws).unwrap().len();
    let mut completing = custom_field_service::get_entity_values(&conn, &donation.id).unwrap();
    completing.insert("donation_status".to_string(), "Completed".to_string());
    custom_field_service::set_entity_values(&conn, "donation", &donation.id, &completing, Some(&admin)).unwrap();
    let tasks_after = task_repo::list(&conn, &ws).unwrap();
    assert_eq!(tasks_after.len(), tasks_before + 1, "completing a donation should create an acknowledgement task");
}

#[test]
fn program_participation_workflow_creates_an_onboarding_task() {
    let (conn, ws, admin) = setup_workspace();
    install_nonprofit(&conn, &ws, &admin);

    let tasks_before = task_repo::list(&conn, &ws).unwrap().len();
    custom_record_service::create(&conn, &ws, &record("program_participation", "Jordan Lee - Reading Program"), Some(&admin)).unwrap();
    let tasks_after = task_repo::list(&conn, &ws).unwrap();
    assert_eq!(tasks_after.len(), tasks_before + 1, "registering a program participant should create an onboarding task");
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

    let input = ImportPackageInput { manifest_json: nonprofit_association_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
