//! The Lanesra Industry Foundation reference package
//! (`services::reference_packages::lanesra_industry_foundation_manifest_json`)
//! - shared cross-industry plumbing (Party, Party Role, Party
//! Relationship, Location, Asset, Agreement, ...) other packages can
//! optionally depend on, unlike the eleven business-vertical packages
//! above it in that module. See that function's own doc comment for what
//! it ships and what it deliberately leaves out.

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::reference_packages::lanesra_industry_foundation_manifest_json;
use lanesra_core::services::{custom_field_service, custom_record_service, industry_package_service, relationship_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Acme Holdings".into(),
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

fn install_foundation(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: lanesra_industry_foundation_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

fn record(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    let json_text = lanesra_industry_foundation_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.industry_foundation");
    assert_eq!(value["objects"].as_array().unwrap().len(), 15);
    assert!(value["dependencies"].as_array().unwrap().is_empty(), "the foundation package itself has no dependencies");
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_foundation(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.industry_foundation");
    assert_eq!(installed.name, "Lanesra Industry Foundation");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 15);
    assert_eq!(count_of("relationship_definition"), 17);
    assert_eq!(count_of("business_rule"), 5);
    assert_eq!(count_of("workflow_definition"), 3);
    assert_eq!(count_of("screen_layout"), 1);
    assert_eq!(count_of("custom_report"), 2);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 0); // no seed data - see the module's own doc comment
}

#[test]
fn party_relationship_gets_two_distinct_relationships_to_the_same_target_type() {
    // Pins down the exact claim this package's own doc comment makes: two
    // relationship_definitions with the identical (source, target) pair
    // don't collide - `relationship_service::slugify` auto-suffixes the
    // second one's key instead of rejecting it.
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    let defs = relationship_service::list(&conn, &ws, true).unwrap();
    let from_party_to_party: Vec<_> = defs
        .iter()
        .filter(|d| d.source_entity_type == "party_relationship" && d.target_entity_type == "party")
        .collect();
    assert_eq!(from_party_to_party.len(), 2, "expected both From Party and To Party relationships to exist");
    let keys: Vec<&str> = from_party_to_party.iter().map(|d| d.key.as_str()).collect();
    assert_ne!(keys[0], keys[1], "the two relationships must have distinct keys");
    let labels: Vec<&str> = from_party_to_party.iter().map(|d| d.forward_label.as_str()).collect();
    assert!(labels.contains(&"From Party"));
    assert!(labels.contains(&"To Party"));
}

#[test]
fn agreement_date_order_rule_blocks_an_expiration_on_or_before_the_effective_date() {
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    let agreement = custom_record_service::create(&conn, &ws, &record("agreement", "Producer appointment"), Some(&admin)).unwrap();
    let mut backwards = HashMap::new();
    backwards.insert("agreement_type".to_string(), "Producer Appointment".to_string());
    backwards.insert("effective_date".to_string(), "2026-06-01".to_string());
    backwards.insert("expiration_date".to_string(), "2026-01-01".to_string());
    let err = custom_field_service::set_entity_values(&conn, "agreement", &agreement.id, &backwards, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Expiration"), "unexpected error: {err}");

    let mut valid = backwards.clone();
    valid.insert("expiration_date".to_string(), "2026-12-31".to_string());
    custom_field_service::set_entity_values(&conn, "agreement", &agreement.id, &valid, Some(&admin)).unwrap();
}

#[test]
fn role_date_order_rule_allows_an_open_ended_role_with_no_valid_to() {
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    let role = custom_record_service::create(&conn, &ws, &record("party_role", "Ongoing producer role"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("role_type".to_string(), "Producer".to_string());
    values.insert("valid_from".to_string(), "2026-01-01".to_string());
    values.insert("role_status".to_string(), "Active".to_string());
    // No valid_to - an open-ended role must not be blocked by the date-order rule.
    custom_field_service::set_entity_values(&conn, "party_role", &role.id, &values, Some(&admin)).unwrap();
}

#[test]
fn role_date_order_rule_blocks_a_valid_to_on_or_before_valid_from() {
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    let role = custom_record_service::create(&conn, &ws, &record("party_role", "Backwards role"), Some(&admin)).unwrap();
    let mut backwards = HashMap::new();
    backwards.insert("role_type".to_string(), "Producer".to_string());
    backwards.insert("valid_from".to_string(), "2026-06-01".to_string());
    backwards.insert("valid_to".to_string(), "2026-01-01".to_string());
    backwards.insert("role_status".to_string(), "Active".to_string());
    let err = custom_field_service::set_entity_values(&conn, "party_role", &role.id, &backwards, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Valid To"), "unexpected error: {err}");

    let mut valid = backwards.clone();
    valid.insert("valid_to".to_string(), "2026-12-31".to_string());
    custom_field_service::set_entity_values(&conn, "party_role", &role.id, &valid, Some(&admin)).unwrap();
}

#[test]
fn granted_consent_rule_requires_a_captured_date() {
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    let consent = custom_record_service::create(&conn, &ws, &record("consent_preference", "Marketing opt-in"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("purpose".to_string(), "Marketing".to_string());
    values.insert("consent_state".to_string(), "Granted".to_string());
    let err = custom_field_service::set_entity_values(&conn, "consent_preference", &consent.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Captured Date"), "unexpected error: {err}");

    values.insert("captured_date".to_string(), "2026-01-15".to_string());
    custom_field_service::set_entity_values(&conn, "consent_preference", &consent.id, &values, Some(&admin)).unwrap();
}

#[test]
fn superseded_document_rule_requires_an_expiry_date() {
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    let doc = custom_record_service::create(&conn, &ws, &record("document_record", "Producer agreement v1"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("document_status".to_string(), "Superseded".to_string());
    let err = custom_field_service::set_entity_values(&conn, "document_record", &doc.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Expiry Date"), "unexpected error: {err}");

    values.insert("expiry_date".to_string(), "2026-01-15".to_string());
    custom_field_service::set_entity_values(&conn, "document_record", &doc.id, &values, Some(&admin)).unwrap();
}

#[test]
fn posted_transaction_rule_requires_a_transaction_date() {
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    let txn = custom_record_service::create(&conn, &ws, &record("financial_transaction", "Commission payout"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("transaction_type".to_string(), "Commission".to_string());
    values.insert("amount".to_string(), "500".to_string());
    values.insert("transaction_status".to_string(), "Posted".to_string());
    let err = custom_field_service::set_entity_values(&conn, "financial_transaction", &txn.id, &values, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Transaction Date"), "unexpected error: {err}");

    values.insert("transaction_date".to_string(), "2026-01-15".to_string());
    custom_field_service::set_entity_values(&conn, "financial_transaction", &txn.id, &values, Some(&admin)).unwrap();
}

#[test]
fn role_ended_workflow_creates_a_review_task() {
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    let role = custom_record_service::create(&conn, &ws, &record("party_role", "Ending role"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("role_type".to_string(), "Tenant".to_string());
    values.insert("role_status".to_string(), "Active".to_string());
    custom_field_service::set_entity_values(&conn, "party_role", &role.id, &values, Some(&admin)).unwrap();

    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    values.insert("role_status".to_string(), "Ended".to_string());
    custom_field_service::set_entity_values(&conn, "party_role", &role.id, &values, Some(&admin)).unwrap();
    let after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after, before + 1, "the 'Role ended' workflow should have created a task");
}

#[test]
fn agreement_expired_workflow_creates_a_review_task() {
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    let agreement = custom_record_service::create(&conn, &ws, &record("agreement", "Vendor agreement"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert("effective_date".to_string(), "2026-01-01".to_string());
    values.insert("expiration_date".to_string(), "2026-12-31".to_string());
    values.insert("agreement_status".to_string(), "Active".to_string());
    custom_field_service::set_entity_values(&conn, "agreement", &agreement.id, &values, Some(&admin)).unwrap();

    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    values.insert("agreement_status".to_string(), "Expired".to_string());
    custom_field_service::set_entity_values(&conn, "agreement", &agreement.id, &values, Some(&admin)).unwrap();
    let after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after, before + 1, "the 'Agreement expired' workflow should have created a task");
}

#[test]
fn new_data_quality_issue_workflow_creates_a_triage_task() {
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    let before = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    custom_record_service::create(&conn, &ws, &record("data_quality_issue", "Missing external key"), Some(&admin)).unwrap();
    let after = lanesra_core::repositories::task_repo::list(&conn, &ws).unwrap().len();
    assert_eq!(after, before + 1, "the 'New data quality issue' workflow should have created a task");
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

    let input = ImportPackageInput { manifest_json: lanesra_industry_foundation_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}

// --- Cross-package dependency + cross-package relationship: the load- -----
// bearing claim this whole package's reason for existing rests on, not
// exercised by any other test in this suite before now.

/// A minimal, inline (not a real shipped package) manifest that depends on
/// Lanesra Industry Foundation and relates one of its own objects to
/// Foundation's `party` object key - proving both `dependencies`
/// enforcement and cross-package relationship targeting actually work
/// end-to-end, not just in principle.
fn dependent_test_manifest_json(dependency_satisfied: bool) -> String {
    let dependencies = if dependency_satisfied {
        r#"[{"package_id":"lanesra.industry_foundation","version_constraint":">=1.0.0","is_required":true}]"#
    } else {
        r#"[{"package_id":"lanesra.industry_foundation","version_constraint":">=99.0.0","is_required":true}]"#
    };
    format!(
        r#"{{
        "format_version": 1,
        "package_id": "lanesra.test_dependent_widget",
        "name": "Test Dependent Widget Package",
        "industry": "Test",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": {dependencies},
        "objects": [
            {{ "key": "test_widget", "singular_label": "Test Widget", "plural_label": "Test Widgets", "icon": "🔩", "prefix": "TW", "digits": 4 }}
        ],
        "fields": [],
        "relationships": [
            {{ "source_entity_type": "test_widget", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "Party", "reverse_label": "Test Widgets", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 }}
        ],
        "business_rules": [],
        "workflows": [],
        "screen_layouts": [],
        "dashboard": null,
        "reports": [],
        "numbering_overrides": [],
        "app": null,
        "seed_data": []
    }}"#
    )
}

#[test]
fn a_dependent_package_cannot_install_without_the_required_foundation_dependency() {
    let (conn, ws, admin) = setup_workspace();
    // Foundation is NOT installed in this workspace at all.
    let input = ImportPackageInput { manifest_json: dependent_test_manifest_json(true) };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&admin)).unwrap_err();
    assert!(
        err.to_string().contains("lanesra.industry_foundation") && err.to_string().contains("installed and active first"),
        "unexpected error: {err}"
    );
}

#[test]
fn a_dependent_package_installs_once_foundation_is_installed_and_relates_to_its_party_object() {
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    let input = ImportPackageInput { manifest_json: dependent_test_manifest_json(true) };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let installed = industry_package_service::install(&conn, &ws, &package.id, Some(&admin)).unwrap();
    assert_eq!(installed.status, "active");

    // The cross-package relationship (test_widget -> Foundation's party) really exists.
    let defs = relationship_service::list(&conn, &ws, true).unwrap();
    let rel = defs
        .iter()
        .find(|d| d.source_entity_type == "test_widget" && d.target_entity_type == "party")
        .expect("the dependent package's relationship to Foundation's party object should exist");
    assert_eq!(rel.forward_label, "Party");

    // And it's usable end-to-end: create a party, create a widget, link them.
    let party = custom_record_service::create(&conn, &ws, &record("party", "Acme Corp Party"), Some(&admin)).unwrap();
    let widget = custom_record_service::create(&conn, &ws, &record("test_widget", "Widget One"), Some(&admin)).unwrap();
    relationship_service::link(&conn, &ws, &rel.id, "test_widget", &widget.id, "party", &party.id, Some(&admin)).unwrap();
}

#[test]
fn a_dependent_package_still_fails_if_the_installed_foundation_version_does_not_satisfy_the_constraint() {
    let (conn, ws, admin) = setup_workspace();
    install_foundation(&conn, &ws, &admin);

    // Foundation is installed and active at 1.0.0, but this dependent
    // manifest demands >=99.0.0 - version_satisfies must actually gate on
    // the constraint, not just presence-and-active.
    let input = ImportPackageInput { manifest_json: dependent_test_manifest_json(false) };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains(">=99.0.0"), "unexpected error: {err}");
}
