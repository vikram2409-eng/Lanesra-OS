//! Integration Hub (spec §7/§9): proves `api_object_service`'s generic
//! dispatch - the one surface both the inbound REST API and the CSV
//! wizard sit on - across a built-in writable entity (Company), a
//! built-in read-only entity (Opportunity - a compound document with its
//! own conversion workflow), Task (whose own service functions take an
//! extra `workspace_id` argument api_object_service must thread through),
//! and a Custom Object (fully dynamic, no per-type code at all).

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::integration::ApiListQuery;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{api_object_service, custom_object_service, opportunity_service, workspace_service};
use serde_json::json;

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "API Object Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

#[test]
fn list_object_keys_includes_every_readable_builtin_and_every_active_custom_object() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    custom_object_service::create(&conn, &workspace_id, &CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 }, Some(&admin_id)).unwrap();

    let keys = api_object_service::list_object_keys(&conn, &workspace_id).unwrap();
    let names: Vec<&str> = keys.iter().map(|k| k.object_key.as_str()).collect();
    assert!(names.contains(&"Company"));
    assert!(names.contains(&"Opportunity"));
    assert!(names.contains(&"vendor"));
}

#[test]
fn company_full_crud_round_trips_through_the_generic_dispatcher() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let created = api_object_service::create_record(&conn, &workspace_id, "Company", &json!({"name": "Acme", "status": "Prospect"}), None).unwrap();
    let id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["name"], "Acme");

    let fetched = api_object_service::get_record(&conn, &workspace_id, "Company", &id).unwrap();
    assert_eq!(fetched["id"], id);

    let updated = api_object_service::update_record(&conn, &workspace_id, "Company", &id, &json!({"name": "Acme Updated", "status": "Active Customer"}), None).unwrap();
    assert_eq!(updated["name"], "Acme Updated");

    let listed = api_object_service::list_records(&conn, &workspace_id, "Company", &ApiListQuery::default()).unwrap();
    assert_eq!(listed.total, 1);

    api_object_service::archive_record(&conn, &workspace_id, "Company", &id, None).unwrap();
    // Archived records are excluded from the default list, same as the UI.
    let after_archive = api_object_service::list_records(&conn, &workspace_id, "Company", &ApiListQuery::default()).unwrap();
    assert_eq!(after_archive.total, 0);
}

#[test]
fn task_writes_thread_the_workspace_id_task_service_itself_requires() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let created = api_object_service::create_record(&conn, &workspace_id, "Task", &json!({"title": "Follow up", "priority": "Normal", "status": "Not Started"}), None).unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    let updated = api_object_service::update_record(&conn, &workspace_id, "Task", &id, &json!({"title": "Follow up urgently", "priority": "High", "status": "In Progress"}), None).unwrap();
    assert_eq!(updated["title"], "Follow up urgently");

    api_object_service::archive_record(&conn, &workspace_id, "Task", &id, None).unwrap();
    let after = api_object_service::list_records(&conn, &workspace_id, "Task", &ApiListQuery::default()).unwrap();
    assert_eq!(after.total, 0);
}

#[test]
fn custom_object_records_work_with_zero_built_in_specific_code() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    custom_object_service::create(&conn, &workspace_id, &CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 }, Some(&admin_id)).unwrap();

    let created = api_object_service::create_record(&conn, &workspace_id, "vendor", &json!({"primary_name": "Acme Supplies", "status": "Active"}), Some(&admin_id)).unwrap();
    assert_eq!(created["primary_name"], "Acme Supplies");
    let id = created["id"].as_str().unwrap().to_string();

    let metadata = api_object_service::get_metadata(&conn, &workspace_id, "vendor").unwrap();
    assert_eq!(metadata.object_key, "vendor");
    assert!(metadata.is_custom);

    let updated = api_object_service::update_record(&conn, &workspace_id, "vendor", &id, &json!({"primary_name": "Acme Supplies Inc", "status": "Active"}), Some(&admin_id)).unwrap();
    assert_eq!(updated["primary_name"], "Acme Supplies Inc");
}

#[test]
fn opportunity_is_read_only_through_the_generic_dispatcher() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let company = api_object_service::create_record(&conn, &workspace_id, "Company", &json!({"name": "Acme", "status": "Prospect"}), Some(&admin_id)).unwrap();
    let company_id = company["id"].as_str().unwrap();
    let opportunity = opportunity_service::create(
        &conn,
        &OpportunityInput {
            company_id: company_id.to_string(), primary_contact_id: None, name: "Big Deal".into(), stage: "Qualified".into(),
            status: "Open".into(), value_cents: 100_00, currency_code: "USD".into(), probability_bp: 5000,
            expected_close_date: Some("2026-01-01".into()), owner_user_id: None, lost_reason: None, next_step: None,
        },
        Some(&admin_id),
    ).unwrap();

    // Reads work fine...
    let fetched = api_object_service::get_record(&conn, &workspace_id, "Opportunity", &opportunity.id).unwrap();
    assert_eq!(fetched["id"], opportunity.id);
    let listed = api_object_service::list_records(&conn, &workspace_id, "Opportunity", &ApiListQuery::default()).unwrap();
    assert_eq!(listed.total, 1);

    // ...but every write is explicitly rejected, not silently attempted.
    assert!(api_object_service::create_record(&conn, &workspace_id, "Opportunity", &json!({}), Some(&admin_id)).is_err());
    assert!(api_object_service::update_record(&conn, &workspace_id, "Opportunity", &opportunity.id, &json!({}), Some(&admin_id)).is_err());
    assert!(api_object_service::archive_record(&conn, &workspace_id, "Opportunity", &opportunity.id, Some(&admin_id)).is_err());
}

#[test]
fn an_unknown_object_key_is_a_clean_not_found_not_a_panic() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    assert!(api_object_service::get_metadata(&conn, &workspace_id, "NotARealObject").is_err());
    assert!(api_object_service::list_records(&conn, &workspace_id, "NotARealObject", &ApiListQuery::default()).is_err());
}

#[test]
fn filter_and_pagination_are_applied_to_list_records() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    for (name, status) in [("Acme", "Prospect"), ("Globex", "Active Customer"), ("Initech", "Active Customer")] {
        api_object_service::create_record(&conn, &workspace_id, "Company", &json!({"name": name, "status": status}), Some(&admin_id)).unwrap();
    }
    let filtered = api_object_service::list_records(&conn, &workspace_id, "Company", &ApiListQuery { filter: Some(json!({"status": "Active Customer"})), ..Default::default() }).unwrap();
    assert_eq!(filtered.total, 2);

    let paged = api_object_service::list_records(&conn, &workspace_id, "Company", &ApiListQuery { page: Some(1), page_size: Some(2), sort: Some(vec!["name".into()]), ..Default::default() }).unwrap();
    assert_eq!(paged.records.len(), 2);
    assert_eq!(paged.total, 3, "total reflects the whole result set, not just this page");
}
