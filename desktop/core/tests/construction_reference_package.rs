//! The Construction & Contractors reference package (`services::
//! reference_packages::construction_manifest_json`) - the third real
//! manifest run through the Industry Data Model foundation, sequenced
//! right after Property Management per the dev spec. See that module's
//! own doc comment for what's included and what's deliberately left out
//! (no vendor-integrity cross-record check, no auto-incrementing approved
//! change value, no conditional bulk auto-close of project tasks).

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::notification_repo;
use lanesra_core::services::reference_packages::construction_manifest_json;
use lanesra_core::services::{company_service, custom_field_service, custom_record_service, industry_package_service, opportunity_service, relationship_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Atlas General Contracting".into(),
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

fn install_construction(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: construction_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

fn record(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    let json_text = construction_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.construction");
    assert_eq!(value["objects"].as_array().unwrap().len(), 5);
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_construction(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.construction");
    assert_eq!(installed.name, "Construction & Contractors");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 5);
    assert_eq!(count_of("custom_field"), 28);
    assert_eq!(count_of("relationship_definition"), 10);
    assert_eq!(count_of("business_rule"), 3);
    assert_eq!(count_of("workflow_definition"), 4);
    assert_eq!(count_of("screen_layout"), 1);
    assert_eq!(count_of("custom_report"), 1);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 0); // no seed data - see the module's own doc comment
}

#[test]
fn close_project_rule_requires_an_actual_end_date() {
    let (conn, ws, admin) = setup_workspace();
    install_construction(&conn, &ws, &admin);

    let project = custom_record_service::create(&conn, &ws, &record("project", "Downtown Office Fitout"), Some(&admin)).unwrap();

    let mut closing = HashMap::new();
    closing.insert("stage".to_string(), "Closed".to_string());
    let err = custom_field_service::set_entity_values(&conn, "project", &project.id, &closing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Actual End Date"));

    closing.insert("actual_end_date".to_string(), "2026-08-01".to_string());
    custom_field_service::set_entity_values(&conn, "project", &project.id, &closing, Some(&admin)).unwrap();
}

#[test]
fn change_approval_rule_requires_approved_amount_and_date() {
    let (conn, ws, admin) = setup_workspace();
    install_construction(&conn, &ws, &admin);

    let co = custom_record_service::create(&conn, &ws, &record("change_order", "Add skylight"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("reason".to_string(), "Client requested skylight".to_string());
    values.insert("amount".to_string(), "4200".to_string());
    custom_field_service::set_entity_values(&conn, "change_order", &co.id, &values, Some(&admin)).unwrap();

    let mut approving = custom_field_service::get_entity_values(&conn, &co.id).unwrap();
    approving.insert("stage".to_string(), "Approved".to_string());
    let err = custom_field_service::set_entity_values(&conn, "change_order", &co.id, &approving, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Approved Amount") || err.to_string().contains("Approved Date"));

    approving.insert("approved_amount".to_string(), "4200".to_string());
    approving.insert("approved_date".to_string(), "2026-08-05".to_string());
    custom_field_service::set_entity_values(&conn, "change_order", &co.id, &approving, Some(&admin)).unwrap();
}

#[test]
fn work_package_complete_rule_requires_a_completion_date() {
    let (conn, ws, admin) = setup_workspace();
    install_construction(&conn, &ws, &admin);

    let wp = custom_record_service::create(&conn, &ws, &record("work_package", "Framing"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("trade_scope".to_string(), "Framing".to_string());
    custom_field_service::set_entity_values(&conn, "work_package", &wp.id, &values, Some(&admin)).unwrap();

    let mut completing = custom_field_service::get_entity_values(&conn, &wp.id).unwrap();
    completing.insert("stage".to_string(), "Complete".to_string());
    let err = custom_field_service::set_entity_values(&conn, "work_package", &wp.id, &completing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Completion Date"));

    completing.insert("completion_date".to_string(), "2026-08-10".to_string());
    custom_field_service::set_entity_values(&conn, "work_package", &wp.id, &completing, Some(&admin)).unwrap();
}

fn opportunity_input(company_id: &str, stage: &str) -> OpportunityInput {
    OpportunityInput {
        company_id: company_id.into(), primary_contact_id: None, name: "Retail Renovation".into(),
        stage: stage.into(), status: if stage == "Won" { "Won".into() } else { "Open".into() },
        value_cents: 300_000_00, currency_code: "USD".into(), probability_bp: 5000,
        expected_close_date: None, owner_user_id: None, lost_reason: None, next_step: None,
    }
}

#[test]
fn opportunity_won_creates_a_linked_project_only_when_the_flag_is_enabled() {
    let (conn, ws, admin) = setup_workspace();
    install_construction(&conn, &ws, &admin);

    let company = company_service::create(
        &conn, &ws,
        &CompanyInput { name: "Riverbend Developers".into(), status: "Prospect".into(), ..Default::default() },
        Some(&admin),
    ).unwrap();

    // Flag left off (the default) - winning it must not create a project.
    let opp_off = opportunity_service::create(&conn, &opportunity_input(&company.id, "Discovery"), Some(&admin)).unwrap();
    let before = custom_record_service::list(&conn, &ws, "project").unwrap().len();
    opportunity_service::update(&conn, &opp_off.id, &opportunity_input(&company.id, "Won"), Some(&admin)).unwrap();
    let after = custom_record_service::list(&conn, &ws, "project").unwrap().len();
    assert_eq!(after, before, "no project should be created when 'Create Project on Won' is off");

    // Flag on - winning it must create a linked project.
    let opp_on = opportunity_service::create(&conn, &opportunity_input(&company.id, "Discovery"), Some(&admin)).unwrap();
    let mut flag = HashMap::new();
    flag.insert("create_project_enabled".to_string(), "true".to_string());
    custom_field_service::set_entity_values(&conn, "Opportunity", &opp_on.id, &flag, Some(&admin)).unwrap();

    let before = custom_record_service::list(&conn, &ws, "project").unwrap().len();
    opportunity_service::update(&conn, &opp_on.id, &opportunity_input(&company.id, "Won"), Some(&admin)).unwrap();
    let after_records = custom_record_service::list(&conn, &ws, "project").unwrap();
    assert_eq!(after_records.len(), before + 1, "a project should be created when the flag is on and the opportunity is won");

    let project = after_records.into_iter().next().unwrap();
    let relationships = relationship_service::list(&conn, &ws, true).unwrap();
    let project_to_opp = relationships.iter().find(|r| r.source_entity_type == "project" && r.target_entity_type == "Opportunity").expect("the manifest defines a project -> Opportunity relationship");
    let linked = relationship_service::related_records_for(&conn, &ws, "project", &project.id).unwrap();
    assert!(
        linked.iter().any(|r| r.entity_type == "Opportunity" && r.entity_id == opp_on.id),
        "the newly created project should be linked back to the winning opportunity via relationship {}",
        project_to_opp.id
    );
}

#[test]
fn inspection_failed_workflow_creates_a_corrective_task_and_notifies_admins() {
    let (conn, ws, admin) = setup_workspace();
    install_construction(&conn, &ws, &admin);

    let inspection = custom_record_service::create(&conn, &ws, &record("inspection", "Rough-in Electrical"), Some(&admin)).unwrap();
    let tasks_before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    let notifications_before = notification_repo::list_for_user(&conn, &ws, &admin, false).unwrap().len();

    let mut values = HashMap::new();
    values.insert("stage".to_string(), "Failed".to_string());
    custom_field_service::set_entity_values(&conn, "inspection", &inspection.id, &values, Some(&admin)).unwrap();

    let tasks_after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    let notifications_after = notification_repo::list_for_user(&conn, &ws, &admin, false).unwrap().len();
    assert_eq!(tasks_after, tasks_before + 1, "a failed inspection should create a corrective task");
    assert_eq!(notifications_after, notifications_before + 1, "a failed inspection should notify admins");
}

#[test]
fn project_close_workflow_creates_a_final_billing_review_task() {
    let (conn, ws, admin) = setup_workspace();
    install_construction(&conn, &ws, &admin);

    let project = custom_record_service::create(&conn, &ws, &record("project", "Riverside Renovation"), Some(&admin)).unwrap();
    let tasks_before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();

    let mut values = HashMap::new();
    values.insert("stage".to_string(), "Closed".to_string());
    values.insert("actual_end_date".to_string(), "2026-08-15".to_string());
    custom_field_service::set_entity_values(&conn, "project", &project.id, &values, Some(&admin)).unwrap();

    let tasks_after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(tasks_after, tasks_before + 1, "closing a project should create a final billing review task");
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

    let input = ImportPackageInput { manifest_json: construction_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
