//! The Real Estate Brokerage reference package (`services::
//! reference_packages::real_estate_manifest_json`) - the seventh real
//! manifest run through the Industry Data Model foundation, sequenced
//! right after Recruitment & Staffing per the dev spec. See that
//! module's own doc comment for what's included and what's deliberately
//! left out (no "only one accepted offer per listing" check, no listing
//! Property/agent relationship-existence requirements, no listing-expiry
//! automation).

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::task_repo;
use lanesra_core::services::reference_packages::real_estate_manifest_json;
use lanesra_core::services::{custom_field_service, custom_record_service, industry_package_service, relationship_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Riverside Realty".into(),
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

fn install_real_estate(conn: &rusqlite::Connection, ws: &str, admin: &str) -> lanesra_core::models::industry_package::InstalledApp {
    let input = ImportPackageInput { manifest_json: real_estate_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap()
}

fn record(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

#[test]
fn the_manifest_itself_parses_and_is_internally_consistent() {
    let json_text = real_estate_manifest_json();
    let value: serde_json::Value = serde_json::from_str(&json_text).expect("manifest is valid JSON");
    assert_eq!(value["package_id"], "lanesra.real_estate");
    assert_eq!(value["objects"].as_array().unwrap().len(), 7);
}

#[test]
fn installs_cleanly_and_creates_every_kind_of_artifact() {
    let (conn, ws, admin) = setup_workspace();
    let installed = install_real_estate(&conn, &ws, &admin);

    assert_eq!(installed.package_id, "lanesra.real_estate");
    assert_eq!(installed.name, "Real Estate Brokerage");
    assert_eq!(installed.status, "active");
    assert!(installed.app_definition_id.is_some());
    assert_eq!(installed.recommended_permissions.len(), 5);

    let detail = industry_package_service::get_installed_detail(&conn, &installed.id).unwrap();
    let count_of = |t: &str| detail.artifacts.iter().filter(|a| a.artifact_type == t).count();
    assert_eq!(count_of("custom_object"), 7);
    assert_eq!(count_of("custom_field"), 23);
    assert_eq!(count_of("relationship_definition"), 10);
    assert_eq!(count_of("business_rule"), 3);
    assert_eq!(count_of("workflow_definition"), 3);
    assert_eq!(count_of("screen_layout"), 1);
    assert_eq!(count_of("custom_report"), 1);
    assert_eq!(count_of("dashboard_layout"), 1);
    assert_eq!(count_of("custom_record"), 0); // no seed data - see the module's own doc comment
}

#[test]
fn offer_integrity_rule_requires_amount_and_expiry_once_out_of_draft() {
    let (conn, ws, admin) = setup_workspace();
    install_real_estate(&conn, &ws, &admin);

    let offer = custom_record_service::create(&conn, &ws, &record("purchase_offer", "Offer on 12 Elm St"), Some(&admin)).unwrap();

    let mut submitting = HashMap::new();
    submitting.insert("offer_stage".to_string(), "Submitted".to_string());
    let err = custom_field_service::set_entity_values(&conn, "purchase_offer", &offer.id, &submitting, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Amount") || err.to_string().contains("Expiry Date"));

    submitting.insert("amount".to_string(), "450000".to_string());
    submitting.insert("expiry_date".to_string(), "2026-09-01".to_string());
    custom_field_service::set_entity_values(&conn, "purchase_offer", &offer.id, &submitting, Some(&admin)).unwrap();
}

#[test]
fn listing_activation_rule_requires_price_and_list_date() {
    let (conn, ws, admin) = setup_workspace();
    install_real_estate(&conn, &ws, &admin);

    let listing = custom_record_service::create(&conn, &ws, &record("listing", "12 Elm St"), Some(&admin)).unwrap();

    let mut activating = HashMap::new();
    activating.insert("listing_stage".to_string(), "Active".to_string());
    let err = custom_field_service::set_entity_values(&conn, "listing", &listing.id, &activating, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("List Price") || err.to_string().contains("List Date"));

    activating.insert("list_price".to_string(), "450000".to_string());
    activating.insert("list_date".to_string(), "2026-08-01".to_string());
    custom_field_service::set_entity_values(&conn, "listing", &listing.id, &activating, Some(&admin)).unwrap();
}

#[test]
fn transaction_close_rule_requires_closing_date_and_final_price() {
    let (conn, ws, admin) = setup_workspace();
    install_real_estate(&conn, &ws, &admin);

    let txn = custom_record_service::create(&conn, &ws, &record("transaction", "12 Elm St sale"), Some(&admin)).unwrap();

    let mut closing = HashMap::new();
    closing.insert("transaction_status".to_string(), "Closed".to_string());
    let err = custom_field_service::set_entity_values(&conn, "transaction", &txn.id, &closing, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Closing Date") || err.to_string().contains("Final Price"));

    closing.insert("closing_date".to_string(), "2026-09-15".to_string());
    closing.insert("final_price".to_string(), "445000".to_string());
    custom_field_service::set_entity_values(&conn, "transaction", &txn.id, &closing, Some(&admin)).unwrap();
}

#[test]
fn showing_scheduled_workflow_creates_a_follow_up_task() {
    let (conn, ws, admin) = setup_workspace();
    install_real_estate(&conn, &ws, &admin);

    let tasks_before = task_repo::list(&conn, &ws).unwrap().len();
    custom_record_service::create(&conn, &ws, &record("showing", "Showing at 12 Elm St"), Some(&admin)).unwrap();
    let tasks_after = task_repo::list(&conn, &ws).unwrap();
    assert_eq!(tasks_after.len(), tasks_before + 1, "creating a showing should create a follow-up task");
}

#[test]
fn offer_accepted_workflow_moves_the_listing_and_opens_a_transaction() {
    let (conn, ws, admin) = setup_workspace();
    install_real_estate(&conn, &ws, &admin);

    let listing = custom_record_service::create(&conn, &ws, &record("listing", "12 Elm St"), Some(&admin)).unwrap();
    let offer = custom_record_service::create(&conn, &ws, &record("purchase_offer", "Offer on 12 Elm St"), Some(&admin)).unwrap();

    let relationships = relationship_service::list(&conn, &ws, true).unwrap();
    let offer_to_listing = relationships.iter().find(|r| r.source_entity_type == "purchase_offer" && r.target_entity_type == "listing").expect("the manifest defines a purchase_offer -> listing relationship");
    relationship_service::link(&conn, &ws, &offer_to_listing.id, "purchase_offer", &offer.id, "listing", &listing.id, Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert("amount".to_string(), "450000".to_string());
    values.insert("expiry_date".to_string(), "2026-09-01".to_string());
    custom_field_service::set_entity_values(&conn, "purchase_offer", &offer.id, &values, Some(&admin)).unwrap();

    let transactions_before = custom_record_service::list(&conn, &ws, "transaction").unwrap().len();
    let mut accepting = custom_field_service::get_entity_values(&conn, &offer.id).unwrap();
    accepting.insert("offer_stage".to_string(), "Accepted".to_string());
    custom_field_service::set_entity_values(&conn, "purchase_offer", &offer.id, &accepting, Some(&admin)).unwrap();

    let listing_values = custom_field_service::get_entity_values(&conn, &listing.id).unwrap();
    assert_eq!(listing_values.get("listing_stage").map(String::as_str), Some("Pending"), "accepting the offer should move the linked listing to Pending");

    let transactions_after = custom_record_service::list(&conn, &ws, "transaction").unwrap();
    assert_eq!(transactions_after.len(), transactions_before + 1, "accepting the offer should open a Transaction");

    let txn = transactions_after.into_iter().next().unwrap();
    let linked = relationship_service::related_records_for(&conn, &ws, "transaction", &txn.id).unwrap();
    assert!(
        linked.iter().any(|r| r.entity_type == "purchase_offer" && r.entity_id == offer.id),
        "the opened transaction should be linked back to the accepted offer"
    );
}

#[test]
fn transaction_closed_workflow_closes_the_listing_and_creates_a_task() {
    let (conn, ws, admin) = setup_workspace();
    install_real_estate(&conn, &ws, &admin);

    let listing = custom_record_service::create(&conn, &ws, &record("listing", "12 Elm St"), Some(&admin)).unwrap();
    let txn = custom_record_service::create(&conn, &ws, &record("transaction", "12 Elm St sale"), Some(&admin)).unwrap();

    let relationships = relationship_service::list(&conn, &ws, true).unwrap();
    let txn_to_listing = relationships.iter().find(|r| r.source_entity_type == "transaction" && r.target_entity_type == "listing").expect("the manifest defines a transaction -> listing relationship");
    relationship_service::link(&conn, &ws, &txn_to_listing.id, "transaction", &txn.id, "listing", &listing.id, Some(&admin)).unwrap();

    let tasks_before = task_repo::list(&conn, &ws).unwrap().len();
    let mut closing = HashMap::new();
    closing.insert("transaction_status".to_string(), "Closed".to_string());
    closing.insert("closing_date".to_string(), "2026-09-15".to_string());
    closing.insert("final_price".to_string(), "445000".to_string());
    custom_field_service::set_entity_values(&conn, "transaction", &txn.id, &closing, Some(&admin)).unwrap();

    let listing_values = custom_field_service::get_entity_values(&conn, &listing.id).unwrap();
    assert_eq!(listing_values.get("listing_stage").map(String::as_str), Some("Closed"), "closing the transaction should close the linked listing");

    let tasks_after = task_repo::list(&conn, &ws).unwrap();
    assert_eq!(tasks_after.len(), tasks_before + 1, "closing the transaction should create a post-closing task");
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

    let input = ImportPackageInput { manifest_json: real_estate_manifest_json() };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = industry_package_service::install(&conn, &ws, &package.id, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}
