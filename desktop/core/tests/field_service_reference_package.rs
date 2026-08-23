//! The Field Service reference package (`services::reference_packages::
//! field_service_manifest_json`) - proves the Industry Data Model
//! foundation against a real, full-featured manifest rather than only
//! the synthetic ones `industry_data_model.rs`'s tests use, and pins
//! down the two behaviors that module's own doc comment calls out as
//! non-obvious: the "Completion validation" business rule actually
//! blocking an incomplete save, and the "Work completed updates asset"
//! workflow's `relationship_ref` resolving correctly end-to-end.

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{custom_field_service, custom_record_service, industry_package_service, relationship_service, user_service, workspace_service};
use lanesra_core::services::reference_packages::field_service_manifest_json;

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Acme Field Service".into(),
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

fn install_field_service(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: field_service_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    // A cheap, fast sanity check independent of the database - if this
    // fails, every other test in this file will fail for the same
    // reason, so it's worth pinning down on its own.
    let json_text = field_service_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.field_service");
    assert_eq!(value["objects"].as_array().unwrap().len(), 10);
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_field_service(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.field_service");
    assert_eq!(installed.name, "Field Service");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 10);
    assert_eq!(count_of("relationship_definition"), 13);
    assert_eq!(count_of("business_rule"), 4);
    assert_eq!(count_of("workflow_definition"), 3);
    assert_eq!(count_of("screen_layout"), 1);
    assert_eq!(count_of("custom_report"), 1);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 3); // seed data
    assert!(count_of("custom_field") > 20);
}

#[test]
fn completion_validation_rule_blocks_an_incomplete_work_order_and_allows_a_complete_one() {
    let (conn, ws, admin) = setup_workspace();
    install_field_service(&conn, &ws, &admin);

    let wo = custom_record_service::create(
        &conn,
        &ws,
        &CustomRecordInput { object_key: "work_order".into(), primary_name: "AC not cooling".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    )
    .unwrap();

    // "description" is required on every save (see set_entity_values'
    // own "whole current state, not a patch" contract) - set it once so
    // later calls that only mean to change stage don't spuriously fail
    // on a field that already had a value.
    let mut values = HashMap::new();
    values.insert("description".to_string(), "Customer reports no cold air".to_string());
    custom_field_service::set_entity_values(&conn, "work_order", &wo.id, &values, Some(&admin)).unwrap();

    // Marking it Completed without resolution/completion_date is blocked.
    let mut incomplete = custom_field_service::get_entity_values(&conn, &wo.id).unwrap();
    incomplete.insert("stage".to_string(), "Completed".to_string());
    let err = custom_field_service::set_entity_values(&conn, "work_order", &wo.id, &incomplete, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Completion Date") || err.to_string().contains("Resolution"));

    // Providing both lets it through.
    let mut complete = incomplete;
    complete.insert("completion_date".to_string(), "2026-01-15".to_string());
    complete.insert("resolution".to_string(), "Replaced the capacitor".to_string());
    custom_field_service::set_entity_values(&conn, "work_order", &wo.id, &complete, Some(&admin)).unwrap();

    let stored = custom_field_service::get_entity_values(&conn, &wo.id).unwrap();
    assert_eq!(stored.get("stage").map(String::as_str), Some("Completed"));
}

#[test]
fn work_completed_workflow_propagates_completion_date_onto_the_linked_asset() {
    let (conn, ws, admin) = setup_workspace();
    install_field_service(&conn, &ws, &admin);

    let asset = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: "asset".into(), primary_name: "Rooftop Unit 3".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();
    let wo = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: "work_order".into(), primary_name: "AC not cooling".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();

    // Link the work order to its asset - the same "Asset" related-list
    // link a dispatcher would use, exercising the exact relationship the
    // workflow's `relationship_ref` resolves against at install time.
    let relationships = relationship_service::list(&conn, &ws, true).unwrap();
    let wo_to_asset = relationships
        .iter()
        .find(|r| r.source_entity_type == "work_order" && r.target_entity_type == "asset")
        .expect("the manifest defines a work_order -> asset relationship");
    relationship_service::link(&conn, &ws, &wo_to_asset.id, "work_order", &wo.id, "asset", &asset.id, Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert("description".to_string(), "Customer reports no cold air".to_string());
    custom_field_service::set_entity_values(&conn, "work_order", &wo.id, &values, Some(&admin)).unwrap();

    let mut complete = custom_field_service::get_entity_values(&conn, &wo.id).unwrap();
    complete.insert("stage".to_string(), "Completed".to_string());
    complete.insert("completion_date".to_string(), "2026-01-15".to_string());
    complete.insert("resolution".to_string(), "Replaced the capacitor".to_string());
    custom_field_service::set_entity_values(&conn, "work_order", &wo.id, &complete, Some(&admin)).unwrap();

    let asset_values = custom_field_service::get_entity_values(&conn, &asset.id).unwrap();
    assert_eq!(asset_values.get("last_service_date").map(String::as_str), Some("2026-01-15"));
}

#[test]
fn new_work_order_workflow_creates_a_dispatcher_task_and_a_notification() {
    let (conn, ws, admin) = setup_workspace();
    install_field_service(&conn, &ws, &admin);

    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: "work_order".into(), primary_name: "AC not cooling".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();
    let after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after, before + 1, "the 'New work order created' workflow should have created a dispatcher task");

    let notifications = lanesra_core::repositories::notification_repo::list_for_user(&conn, &ws, &admin, false).unwrap();
    assert!(notifications.iter().any(|n| n.message.contains("New work order created")));
}

#[test]
fn appointment_completion_rule_requires_actual_times_and_outcome() {
    let (conn, ws, admin) = setup_workspace();
    install_field_service(&conn, &ws, &admin);

    let wo = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: "work_order".into(), primary_name: "AC not cooling".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();
    let _ = wo; // the appointment doesn't need to be linked for this rule - it's a same-record check

    let appt = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: "service_appointment".into(), primary_name: "Visit 1".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();

    let mut incomplete = HashMap::new();
    incomplete.insert("appt_stage".to_string(), "Completed".to_string());
    let err = custom_field_service::set_entity_values(&conn, "service_appointment", &appt.id, &incomplete, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Actual") || err.to_string().contains("Outcome"));

    let mut complete = incomplete;
    complete.insert("actual_start".to_string(), "2026-01-15".to_string());
    complete.insert("actual_end".to_string(), "2026-01-15".to_string());
    complete.insert("outcome".to_string(), "Repaired".to_string());
    custom_field_service::set_entity_values(&conn, "service_appointment", &appt.id, &complete, Some(&admin)).unwrap();
}

#[test]
fn warranty_claim_resolution_rules_block_an_unresolved_claim_and_allow_a_resolved_one() {
    let (conn, ws, admin) = setup_workspace();
    install_field_service(&conn, &ws, &admin);

    let claim = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: "warranty_claim".into(), primary_name: "Compressor failure".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();

    // Denying a claim with no resolution notes is blocked by "Claim
    // resolution notes" - the only one of the two rules that applies to
    // a non-Reimbursed resolution.
    let mut values = HashMap::new();
    values.insert("claim_status".to_string(), "Denied".to_string());
    let err = custom_field_service::set_entity_values(&conn, "warranty_claim", &claim.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Resolution Notes"));

    // Providing notes lets the denial through.
    values.insert("resolution_notes".to_string(), "Out of warranty period".to_string());
    custom_field_service::set_entity_values(&conn, "warranty_claim", &claim.id, &values, Some(&admin)).unwrap();

    // Moving to Reimbursed (notes already on file) is blocked by "Claim
    // reimbursement amount" until an approved amount is recorded too.
    values.insert("claim_status".to_string(), "Reimbursed".to_string());
    let err = custom_field_service::set_entity_values(&conn, "warranty_claim", &claim.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Amount Approved"));

    values.insert("amount_approved".to_string(), "450".to_string());
    custom_field_service::set_entity_values(&conn, "warranty_claim", &claim.id, &values, Some(&admin)).unwrap();

    let stored = custom_field_service::get_entity_values(&conn, &claim.id).unwrap();
    assert_eq!(stored.get("claim_status").map(String::as_str), Some("Reimbursed"));
}

#[test]
fn warranty_claim_submitted_workflow_creates_a_review_task_and_a_notification() {
    let (conn, ws, admin) = setup_workspace();
    install_field_service(&conn, &ws, &admin);

    let claim = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: "warranty_claim".into(), primary_name: "Compressor failure".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();

    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    let mut values = custom_field_service::get_entity_values(&conn, &claim.id).unwrap();
    values.insert("claim_status".to_string(), "Submitted".to_string());
    custom_field_service::set_entity_values(&conn, "warranty_claim", &claim.id, &values, Some(&admin)).unwrap();
    let after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after, before + 1, "the 'Warranty claim submitted' workflow should have created a review task");

    let notifications = lanesra_core::repositories::notification_repo::list_for_user(&conn, &ws, &admin, false).unwrap();
    assert!(notifications.iter().any(|n| n.message.contains("warranty claim was submitted")));
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

    let input = ImportPackageInput { manifest_json: field_service_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
