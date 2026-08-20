//! Solution Packages & Admin IA design spec, Phase 2: the Publisher
//! registry (migration 0029) and the publisher-scoped namespace
//! validation `industry_package_service::import_package` now enforces -
//! see `publisher_service`'s own module doc comment.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::domain::AppError;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::publisher::PublisherInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{industry_package_service, publisher_service, user_service, workspace_service};
use serde_json::json;

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Publisher Co".into(),
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

fn user_with_role(conn: &rusqlite::Connection, ws: &str, admin: &str, username: &str, role: &str) -> String {
    user_service::create(
        conn, ws,
        &NewUser { username: username.into(), display_name: username.into(), password: "anothersecretpw".into(), roles: vec![role.into()] },
        Some(admin),
    )
    .unwrap()
    .id
}

fn tiny_manifest(package_id: &str) -> String {
    json!({
        "format_version": 1,
        "package_id": package_id,
        "name": "Tiny Pack",
        "industry": "Testing",
        "version": "1.0.0",
        "min_lanesra_version": "0.1.0",
    })
    .to_string()
}

#[test]
fn first_run_setup_seeds_the_two_default_publishers() {
    let (conn, ws, _admin) = setup_workspace();
    let publishers = publisher_service::list(&conn, &ws).unwrap();
    let keys: Vec<&str> = publishers.iter().map(|p| p.key.as_str()).collect();
    assert!(keys.contains(&"lanesra"));
    assert!(keys.contains(&"local"));
    let lanesra = publishers.iter().find(|p| p.key == "lanesra").unwrap();
    assert!(lanesra.is_official);
    assert!(!lanesra.is_local);
    let local = publishers.iter().find(|p| p.key == "local").unwrap();
    assert!(local.is_local);
    assert!(!local.is_official);
}

#[test]
fn ensure_defaults_is_idempotent() {
    let (conn, ws, _admin) = setup_workspace();
    publisher_service::ensure_defaults(&conn, &ws).unwrap();
    publisher_service::ensure_defaults(&conn, &ws).unwrap();
    let publishers = publisher_service::list(&conn, &ws).unwrap();
    assert_eq!(publishers.iter().filter(|p| p.key == "lanesra").count(), 1);
    assert_eq!(publishers.iter().filter(|p| p.key == "local").count(), 1);
}

#[test]
fn creating_a_publisher_requires_admin() {
    let (conn, ws, admin) = setup_workspace();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let input = PublisherInput { key: "acme".into(), name: "Acme Corp".into(), description: None };
    let err = publisher_service::create(&conn, &ws, &input, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}

#[test]
fn reserved_keys_are_rejected() {
    let (conn, ws, admin) = setup_workspace();
    for reserved in ["lanesra", "local"] {
        let input = PublisherInput { key: reserved.into(), name: "Whatever".into(), description: None };
        let err = publisher_service::create(&conn, &ws, &input, Some(&admin)).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "expected Validation for key '{reserved}', got {err:?}");
    }
}

#[test]
fn malformed_keys_are_rejected() {
    // "Acme" is deliberately not in this list - create() lowercases the
    // key before validating (a UX nicety, not a bug), so it normalizes to
    // the perfectly valid "acme" rather than failing.
    let (conn, ws, admin) = setup_workspace();
    for bad in ["A", "1acme", "acme corp", "ac-me", "a"] {
        let input = PublisherInput { key: bad.into(), name: "Whatever".into(), description: None };
        let err = publisher_service::create(&conn, &ws, &input, Some(&admin)).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "expected Validation for key '{bad}', got {err:?}");
    }
}

#[test]
fn duplicate_keys_are_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let input = PublisherInput { key: "acme".into(), name: "Acme Corp".into(), description: None };
    publisher_service::create(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = publisher_service::create(&conn, &ws, &input, Some(&admin)).unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
}

#[test]
fn a_valid_publisher_can_be_created_and_is_listed() {
    let (conn, ws, admin) = setup_workspace();
    let input = PublisherInput { key: "acme".into(), name: "Acme Corp".into(), description: Some("A test publisher".into()) };
    let created = publisher_service::create(&conn, &ws, &input, Some(&admin)).unwrap();
    assert_eq!(created.key, "acme");
    assert!(!created.is_official);
    assert!(!created.is_local);

    let publishers = publisher_service::list(&conn, &ws).unwrap();
    assert!(publishers.iter().any(|p| p.key == "acme"));
}

#[test]
fn importing_a_package_under_an_unregistered_namespace_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let input = ImportPackageInput { manifest_json: tiny_manifest("acme.inspection") };
    let err = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("acme"));
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn importing_a_package_succeeds_once_its_publisher_is_registered() {
    let (conn, ws, admin) = setup_workspace();
    publisher_service::create(&conn, &ws, &PublisherInput { key: "acme".into(), name: "Acme Corp".into(), description: None }, Some(&admin)).unwrap();

    let input = ImportPackageInput { manifest_json: tiny_manifest("acme.inspection") };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    assert_eq!(package.package_id, "acme.inspection");
    assert!(package.is_managed);

    let acme = publisher_service::list(&conn, &ws).unwrap().into_iter().find(|p| p.key == "acme").unwrap();
    assert_eq!(package.publisher_id.as_deref(), Some(acme.id.as_str()));
}

#[test]
fn bundled_reference_packages_import_out_of_the_box_under_the_lanesra_publisher() {
    // No admin setup step required - ensure_defaults auto-seeds "lanesra"
    // the first time it's needed, so every bundled starter (all namespaced
    // "lanesra.<name>") keeps working exactly as before this phase.
    let (conn, ws, admin) = setup_workspace();
    let input = ImportPackageInput { manifest_json: tiny_manifest("lanesra.field_service") };
    let package = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap();
    assert!(package.is_managed);

    let lanesra = publisher_service::list(&conn, &ws).unwrap().into_iter().find(|p| p.key == "lanesra").unwrap();
    assert_eq!(package.publisher_id.as_deref(), Some(lanesra.id.as_str()));
}

#[test]
fn a_package_id_without_a_namespace_dot_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let input = ImportPackageInput { manifest_json: tiny_manifest("no_namespace_here") };
    let err = industry_package_service::import_package(&conn, &ws, &input, Some(&admin)).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}
