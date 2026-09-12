//! The Policy Administration & Claims Management reference package
//! (`services::reference_packages::policy_admin_claims_manifest_json`) -
//! the eleventh reference package, added after the original ten-vertical
//! spec. See that module's own doc comment for what's included and what's
//! deliberately left out (no polymorphic Named Insured, no Underwriting
//! Referral object, no rollup reserve/payment totals).

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::reference_packages::policy_admin_claims_manifest_json;
use lanesra_core::services::{custom_field_service, custom_record_service, industry_package_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Acme Mutual Insurance".into(),
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

fn install_policy_admin_claims(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: policy_admin_claims_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

fn record(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    let json_text = policy_admin_claims_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.policy_admin_claims");
    assert_eq!(value["objects"].as_array().unwrap().len(), 10);
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_policy_admin_claims(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.policy_admin_claims");
    assert_eq!(installed.name, "Policy Administration & Claims Management");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 10);
    assert_eq!(count_of("custom_field"), 57);
    assert_eq!(count_of("relationship_definition"), 14);
    assert_eq!(count_of("business_rule"), 4);
    assert_eq!(count_of("workflow_definition"), 5);
    assert_eq!(count_of("screen_layout"), 2);
    assert_eq!(count_of("custom_report"), 3);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 0); // no seed data - see the module's own doc comment
}

#[test]
fn policy_date_validation_blocks_an_expiration_on_or_before_the_effective_date() {
    let (conn, ws, admin) = setup_workspace();
    install_policy_admin_claims(&conn, &ws, &admin);

    let policy = custom_record_service::create(&conn, &ws, &record("policy", "PA-1000001"), Some(&admin)).unwrap();

    let mut backwards = HashMap::new();
    backwards.insert("product_line".to_string(), "Personal Auto".to_string());
    backwards.insert("policy_type".to_string(), "New Business".to_string());
    backwards.insert("effective_date".to_string(), "2026-06-01".to_string());
    backwards.insert("expiration_date".to_string(), "2026-01-01".to_string());
    let err = custom_field_service::set_entity_values(&conn, "policy", &policy.id, &backwards, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("expiration"), "unexpected error: {err}");

    let mut valid = backwards.clone();
    valid.insert("expiration_date".to_string(), "2026-12-31".to_string());
    custom_field_service::set_entity_values(&conn, "policy", &policy.id, &valid, Some(&admin)).unwrap();
}

#[test]
fn claim_payout_documentation_rule_requires_a_paid_amount_when_closed_paid() {
    let (conn, ws, admin) = setup_workspace();
    install_policy_admin_claims(&conn, &ws, &admin);

    let claim = custom_record_service::create(&conn, &ws, &record("claim", "Rear-end collision"), Some(&admin)).unwrap();
    let mut base = HashMap::new();
    base.insert("date_of_loss".to_string(), "2026-02-01".to_string());
    base.insert("date_reported".to_string(), "2026-02-02".to_string());
    base.insert("loss_type".to_string(), "Collision".to_string());
    base.insert("description_of_loss".to_string(), "Rear-ended at a stoplight".to_string());
    custom_field_service::set_entity_values(&conn, "claim", &claim.id, &base, Some(&admin)).unwrap();

    let mut closing = custom_field_service::get_entity_values(&conn, &claim.id).unwrap();
    closing.insert("claim_status".to_string(), "Closed - Paid".to_string());
    let err = custom_field_service::set_entity_values(&conn, "claim", &claim.id, &closing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Total Paid"), "unexpected error: {err}");

    closing.insert("paid_amount".to_string(), "4500".to_string());
    custom_field_service::set_entity_values(&conn, "claim", &claim.id, &closing, Some(&admin)).unwrap();
}

#[test]
fn attorney_representation_rule_requires_an_attorney_name() {
    let (conn, ws, admin) = setup_workspace();
    install_policy_admin_claims(&conn, &ws, &admin);

    let claimant = custom_record_service::create(&conn, &ws, &record("claimant", "Jordan Rivera"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("claimant_type".to_string(), "Third Party".to_string());
    values.insert("represented_by_attorney".to_string(), "true".to_string());
    let err = custom_field_service::set_entity_values(&conn, "claimant", &claimant.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Attorney Name"), "unexpected error: {err}");

    values.insert("attorney_name".to_string(), "Alex Chen, Esq.".to_string());
    custom_field_service::set_entity_values(&conn, "claimant", &claimant.id, &values, Some(&admin)).unwrap();
}

#[test]
fn subrogation_recovery_documentation_rule_requires_a_recovered_amount() {
    let (conn, ws, admin) = setup_workspace();
    install_policy_admin_claims(&conn, &ws, &admin);

    let case = custom_record_service::create(&conn, &ws, &record("subrogation", "At-fault driver recovery"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("subrogation_status".to_string(), "Recovered".to_string());
    let err = custom_field_service::set_entity_values(&conn, "subrogation", &case.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Recovered Amount"), "unexpected error: {err}");

    values.insert("recovered_amount".to_string(), "3200".to_string());
    custom_field_service::set_entity_values(&conn, "subrogation", &case.id, &values, Some(&admin)).unwrap();
}

#[test]
fn policy_bound_workflow_creates_a_document_issuance_task() {
    let (conn, ws, admin) = setup_workspace();
    install_policy_admin_claims(&conn, &ws, &admin);

    let policy = custom_record_service::create(&conn, &ws, &record("policy", "PA-1000002"), Some(&admin)).unwrap();
    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    let mut values = HashMap::new();
    values.insert("product_line".to_string(), "Personal Auto".to_string());
    values.insert("policy_type".to_string(), "New Business".to_string());
    values.insert("effective_date".to_string(), "2026-01-01".to_string());
    values.insert("expiration_date".to_string(), "2026-12-31".to_string());
    values.insert("policy_stage".to_string(), "Bound".to_string());
    custom_field_service::set_entity_values(&conn, "policy", &policy.id, &values, Some(&admin)).unwrap();
    let after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after, before + 1, "the 'Policy bound' workflow should have created a task");
}

#[test]
fn claim_intake_workflow_creates_an_investigation_task() {
    let (conn, ws, admin) = setup_workspace();
    install_policy_admin_claims(&conn, &ws, &admin);

    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    custom_record_service::create(&conn, &ws, &record("claim", "Kitchen fire"), Some(&admin)).unwrap();
    let after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after, before + 1, "the 'Claim intake' workflow should have created a task");
}

#[test]
fn siu_referral_escalation_workflow_creates_a_task_only_on_confirmed_referral() {
    let (conn, ws, admin) = setup_workspace();
    install_policy_admin_claims(&conn, &ws, &admin);

    let claim = custom_record_service::create(&conn, &ws, &record("claim", "Suspicious theft claim"), Some(&admin)).unwrap();
    let after_intake = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();

    let mut under_review = HashMap::new();
    under_review.insert("date_of_loss".to_string(), "2026-02-01".to_string());
    under_review.insert("date_reported".to_string(), "2026-02-02".to_string());
    under_review.insert("loss_type".to_string(), "Theft".to_string());
    under_review.insert("description_of_loss".to_string(), "Reported stolen from a parking lot".to_string());
    under_review.insert("fraud_referral".to_string(), "Under Review".to_string());
    custom_field_service::set_entity_values(&conn, "claim", &claim.id, &under_review, Some(&admin)).unwrap();
    let after_under_review = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after_under_review, after_intake, "'Under Review' alone should not trigger the SIU escalation");

    let mut confirmed = custom_field_service::get_entity_values(&conn, &claim.id).unwrap();
    confirmed.insert("fraud_referral".to_string(), "Confirmed - SIU".to_string());
    custom_field_service::set_entity_values(&conn, "claim", &claim.id, &confirmed, Some(&admin)).unwrap();
    let after_confirmed = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after_confirmed, after_under_review + 1, "the 'SIU referral escalation' workflow should have created a task");
}

#[test]
fn claim_closure_follow_up_workflow_creates_a_task_for_either_closed_state() {
    let (conn, ws, admin) = setup_workspace();
    install_policy_admin_claims(&conn, &ws, &admin);

    let claim = custom_record_service::create(&conn, &ws, &record("claim", "Denied water damage claim"), Some(&admin)).unwrap();
    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    let mut values = HashMap::new();
    values.insert("date_of_loss".to_string(), "2026-02-01".to_string());
    values.insert("date_reported".to_string(), "2026-02-02".to_string());
    values.insert("loss_type".to_string(), "Water Damage".to_string());
    values.insert("description_of_loss".to_string(), "Burst pipe under the kitchen sink".to_string());
    values.insert("claim_status".to_string(), "Closed - Denied".to_string());
    custom_field_service::set_entity_values(&conn, "claim", &claim.id, &values, Some(&admin)).unwrap();
    let after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after, before + 1, "the 'Claim closure follow-up' workflow should have created a task");
}

#[test]
fn endorsement_processed_workflow_creates_a_billing_task() {
    let (conn, ws, admin) = setup_workspace();
    install_policy_admin_claims(&conn, &ws, &admin);

    let endorsement = custom_record_service::create(&conn, &ws, &record("endorsement", "Add second vehicle"), Some(&admin)).unwrap();
    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    let mut values = HashMap::new();
    values.insert("endorsement_type".to_string(), "Add Insured Risk".to_string());
    values.insert("effective_date".to_string(), "2026-03-01".to_string());
    values.insert("endorsement_status".to_string(), "Processed".to_string());
    custom_field_service::set_entity_values(&conn, "endorsement", &endorsement.id, &values, Some(&admin)).unwrap();
    let after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after, before + 1, "the 'Endorsement processed' workflow should have created a task");
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

    let input = ImportPackageInput { manifest_json: policy_admin_claims_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
