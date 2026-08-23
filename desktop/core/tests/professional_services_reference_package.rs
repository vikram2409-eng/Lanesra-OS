//! The Professional Services reference package (`services::
//! reference_packages::professional_services_manifest_json`) - the fourth
//! real manifest run through the Industry Data Model foundation,
//! sequenced right after Construction & Contractors per the dev spec.
//! See that module's own doc comment for what's included and what's
//! deliberately left out (no cross-record "Project Active" check on Time
//! submission, no bill-rate snapshot from Resource Assignment, no
//! "Milestone due soon" date-triggered reminder).

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::reference_packages::professional_services_manifest_json;
use lanesra_core::services::{company_service, custom_field_service, custom_record_service, industry_package_service, opportunity_service, relationship_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Northbridge Consulting".into(),
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

fn install_professional_services(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: professional_services_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

fn record(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

fn opportunity_input(company_id: &str, stage: &str) -> OpportunityInput {
    OpportunityInput {
        company_id: company_id.into(), primary_contact_id: None, name: "Digital Transformation Engagement".into(),
        stage: stage.into(), status: if stage == "Won" { "Won".into() } else { "Open".into() },
        value_cents: 250_000_00, currency_code: "USD".into(), probability_bp: 5000,
        expected_close_date: None, owner_user_id: None, lost_reason: None, next_step: None,
    }
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    let json_text = professional_services_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.professional_services");
    assert_eq!(value["objects"].as_array().unwrap().len(), 7);
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_professional_services(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.professional_services");
    assert_eq!(installed.name, "Professional Services");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 7);
    assert_eq!(count_of("custom_field"), 35);
    assert_eq!(count_of("relationship_definition"), 11);
    assert_eq!(count_of("business_rule"), 5);
    assert_eq!(count_of("workflow_definition"), 4);
    assert_eq!(count_of("screen_layout"), 1);
    assert_eq!(count_of("custom_report"), 1);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 0); // no seed data - see the module's own doc comment
}

#[test]
fn engagement_completion_rule_requires_an_actual_end_date() {
    let (conn, ws, admin) = setup_workspace();
    install_professional_services(&conn, &ws, &admin);

    let engagement = custom_record_service::create(&conn, &ws, &record("engagement", "Retail Analytics Rollout"), Some(&admin)).unwrap();

    let mut completing = HashMap::new();
    completing.insert("stage".to_string(), "Complete".to_string());
    let err = custom_field_service::set_entity_values(&conn, "engagement", &engagement.id, &completing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Actual End Date"));

    completing.insert("actual_end_date".to_string(), "2026-08-20".to_string());
    custom_field_service::set_entity_values(&conn, "engagement", &engagement.id, &completing, Some(&admin)).unwrap();
}

#[test]
fn time_submission_rule_requires_more_than_zero_hours() {
    let (conn, ws, admin) = setup_workspace();
    install_professional_services(&conn, &ws, &admin);

    let entry = custom_record_service::create(&conn, &ws, &record("time_entry", "Week of Aug 17"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("date".to_string(), "2026-08-17".to_string());
    values.insert("hours".to_string(), "0".to_string());
    custom_field_service::set_entity_values(&conn, "time_entry", &entry.id, &values, Some(&admin)).unwrap();

    let mut submitting = custom_field_service::get_entity_values(&conn, &entry.id).unwrap();
    submitting.insert("stage".to_string(), "Submitted".to_string());
    let err = custom_field_service::set_entity_values(&conn, "time_entry", &entry.id, &submitting, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("zero hours"));

    submitting.insert("hours".to_string(), "6.5".to_string());
    custom_field_service::set_entity_values(&conn, "time_entry", &entry.id, &submitting, Some(&admin)).unwrap();
}

#[test]
fn expense_submission_rule_requires_category_amount_and_date() {
    let (conn, ws, admin) = setup_workspace();
    install_professional_services(&conn, &ws, &admin);

    let expense = custom_record_service::create(&conn, &ws, &record("expense", "Client site visit"), Some(&admin)).unwrap();
    let mut submitting = HashMap::new();
    submitting.insert("stage".to_string(), "Submitted".to_string());
    let err = custom_field_service::set_entity_values(&conn, "expense", &expense.id, &submitting, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Category") || err.to_string().contains("Amount") || err.to_string().contains("Date"));

    submitting.insert("category".to_string(), "Travel".to_string());
    submitting.insert("amount".to_string(), "184.50".to_string());
    submitting.insert("date".to_string(), "2026-08-16".to_string());
    custom_field_service::set_entity_values(&conn, "expense", &expense.id, &submitting, Some(&admin)).unwrap();
}

#[test]
fn milestone_completion_rule_requires_a_completed_date() {
    let (conn, ws, admin) = setup_workspace();
    install_professional_services(&conn, &ws, &admin);

    let milestone = custom_record_service::create(&conn, &ws, &record("milestone", "Discovery workshop"), Some(&admin)).unwrap();
    let mut completing = HashMap::new();
    completing.insert("stage".to_string(), "Complete".to_string());
    let err = custom_field_service::set_entity_values(&conn, "milestone", &milestone.id, &completing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Completed Date"));

    completing.insert("completed_date".to_string(), "2026-08-18".to_string());
    custom_field_service::set_entity_values(&conn, "milestone", &milestone.id, &completing, Some(&admin)).unwrap();
}

#[test]
fn opportunity_won_creates_a_linked_engagement_only_when_the_flag_is_enabled() {
    let (conn, ws, admin) = setup_workspace();
    install_professional_services(&conn, &ws, &admin);

    let company = company_service::create(
        &conn, &ws,
        &CompanyInput { name: "Meridian Health Group".into(), status: "Prospect".into(), ..Default::default() },
        Some(&admin),
    ).unwrap();

    // Flag left off (the default) - winning it must not create an engagement.
    let opp_off = opportunity_service::create(&conn, &opportunity_input(&company.id, "Discovery"), Some(&admin)).unwrap();
    let before = custom_record_service::list(&conn, &ws, "engagement").unwrap().len();
    opportunity_service::update(&conn, &opp_off.id, &opportunity_input(&company.id, "Won"), Some(&admin)).unwrap();
    let after = custom_record_service::list(&conn, &ws, "engagement").unwrap().len();
    assert_eq!(after, before, "no engagement should be created when 'Create Engagement on Won' is off");

    // Flag on - winning it must create a linked engagement.
    let opp_on = opportunity_service::create(&conn, &opportunity_input(&company.id, "Discovery"), Some(&admin)).unwrap();
    let mut flag = HashMap::new();
    flag.insert("create_engagement_enabled".to_string(), "true".to_string());
    custom_field_service::set_entity_values(&conn, "Opportunity", &opp_on.id, &flag, Some(&admin)).unwrap();

    let before = custom_record_service::list(&conn, &ws, "engagement").unwrap().len();
    opportunity_service::update(&conn, &opp_on.id, &opportunity_input(&company.id, "Won"), Some(&admin)).unwrap();
    let after_records = custom_record_service::list(&conn, &ws, "engagement").unwrap();
    assert_eq!(after_records.len(), before + 1, "an engagement should be created when the flag is on and the opportunity is won");

    let engagement = after_records.into_iter().next().unwrap();
    let relationships = relationship_service::list(&conn, &ws, true).unwrap();
    let engagement_to_opp = relationships.iter().find(|r| r.source_entity_type == "engagement" && r.target_entity_type == "Opportunity").expect("the manifest defines an engagement -> Opportunity relationship");
    let linked = relationship_service::related_records_for(&conn, &ws, "engagement", &engagement.id).unwrap();
    assert!(
        linked.iter().any(|r| r.entity_type == "Opportunity" && r.entity_id == opp_on.id),
        "the newly created engagement should be linked back to the winning opportunity via relationship {}",
        engagement_to_opp.id
    );
}

#[test]
fn time_approved_workflow_marks_it_eligible_for_billing() {
    let (conn, ws, admin) = setup_workspace();
    install_professional_services(&conn, &ws, &admin);

    let entry = custom_record_service::create(&conn, &ws, &record("time_entry", "Week of Aug 17"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("date".to_string(), "2026-08-17".to_string());
    values.insert("hours".to_string(), "6.5".to_string());
    custom_field_service::set_entity_values(&conn, "time_entry", &entry.id, &values, Some(&admin)).unwrap();

    let before = custom_field_service::get_entity_values(&conn, &entry.id).unwrap();
    assert_eq!(before.get("billing_status").map(String::as_str), Some("Not Billed"));

    let mut approving = before;
    approving.insert("stage".to_string(), "Approved".to_string());
    custom_field_service::set_entity_values(&conn, "time_entry", &entry.id, &approving, Some(&admin)).unwrap();

    let after = custom_field_service::get_entity_values(&conn, &entry.id).unwrap();
    assert_eq!(after.get("billing_status").map(String::as_str), Some("Eligible"));
}

#[test]
fn engagement_complete_workflow_creates_two_tasks() {
    let (conn, ws, admin) = setup_workspace();
    install_professional_services(&conn, &ws, &admin);

    let engagement = custom_record_service::create(&conn, &ws, &record("engagement", "Retail Analytics Rollout"), Some(&admin)).unwrap();
    let tasks_before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();

    let mut values = HashMap::new();
    values.insert("stage".to_string(), "Complete".to_string());
    values.insert("actual_end_date".to_string(), "2026-08-20".to_string());
    custom_field_service::set_entity_values(&conn, "engagement", &engagement.id, &values, Some(&admin)).unwrap();

    let tasks_after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(tasks_after, tasks_before + 2, "completing an engagement should create a closure task and an invoice-preparation task");
}

#[test]
fn change_request_approval_rule_requires_an_approved_date_when_approved_or_implemented() {
    let (conn, ws, admin) = setup_workspace();
    install_professional_services(&conn, &ws, &admin);

    let cr = custom_record_service::create(&conn, &ws, &record("change_request", "Add mobile reporting module"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("description".to_string(), "Client requested a mobile reporting module".to_string());
    custom_field_service::set_entity_values(&conn, "change_request", &cr.id, &values, Some(&admin)).unwrap();

    values.insert("stage".to_string(), "Approved".to_string());
    let err = custom_field_service::set_entity_values(&conn, "change_request", &cr.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Approved Date"));

    values.insert("approved_date".to_string(), "2026-08-19".to_string());
    custom_field_service::set_entity_values(&conn, "change_request", &cr.id, &values, Some(&admin)).unwrap();

    // Implemented also requires it - already on file here, proving the rule
    // doesn't spuriously block a record that already satisfies it.
    values.insert("stage".to_string(), "Implemented".to_string());
    custom_field_service::set_entity_values(&conn, "change_request", &cr.id, &values, Some(&admin)).unwrap();
}

#[test]
fn change_request_submitted_workflow_creates_a_review_task() {
    let (conn, ws, admin) = setup_workspace();
    install_professional_services(&conn, &ws, &admin);

    let cr = custom_record_service::create(&conn, &ws, &record("change_request", "Add mobile reporting module"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("description".to_string(), "Client requested a mobile reporting module".to_string());
    custom_field_service::set_entity_values(&conn, "change_request", &cr.id, &values, Some(&admin)).unwrap();

    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    values.insert("stage".to_string(), "Submitted".to_string());
    custom_field_service::set_entity_values(&conn, "change_request", &cr.id, &values, Some(&admin)).unwrap();
    let after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after, before + 1, "the 'Change request submitted' workflow should have created a review task");
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

    let input = ImportPackageInput { manifest_json: professional_services_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
