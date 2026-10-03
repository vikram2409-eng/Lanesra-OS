//! Admin Control Center Modernization (issue #197): `admin_search_service::
//! admin_search` is a dozen raw SQL `LIKE` queries over tables this module
//! doesn't own - this smoke test proves every one of them actually runs
//! against the real schema and that a known-created object surfaces.

use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{admin_search_service, custom_object_service, workspace_service};

fn setup_workspace(business_name: &str) -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: business_name.into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

#[test]
fn admin_search_runs_every_query_and_finds_a_matching_custom_object() {
    let (conn, ws, admin) = setup_workspace("Admin Search Co");

    // A query too short to search should just return empty, no queries run.
    let empty = admin_search_service::admin_search(&conn, &ws, "a", Some(&admin)).unwrap();
    assert!(empty.is_empty());

    let created = custom_object_service::create(
        &conn, &ws,
        &CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏷️".into(), prefix: "VEN".into(), digits: 4 },
        Some(&admin),
    )
    .unwrap();

    let results = admin_search_service::admin_search(&conn, &ws, "Vendor", Some(&admin)).unwrap();
    assert!(results.iter().any(|r| r.entity_id == created.id && r.admin_tab == "objects"), "{results:?}");
}

#[test]
fn non_admin_cannot_use_admin_search() {
    let (conn, ws, admin) = setup_workspace("Admin Search Guard Co");
    let non_admin_input = lanesra_core::models::user::NewUser {
        username: "sales_rep".into(),
        display_name: "Sales Rep".into(),
        password: "supersecretpassword".into(),
        roles: vec!["Sales".into()],
    };
    let non_admin = lanesra_core::services::user_service::create(&conn, &ws, &non_admin_input, Some(&admin)).unwrap();
    let err = admin_search_service::admin_search(&conn, &ws, "anything", Some(&non_admin.id)).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("admin"), "{err}");
}
