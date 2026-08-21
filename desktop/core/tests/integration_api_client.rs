//! Integration Hub (spec §8): proves `api_client_service` - issuing a
//! `{client_id}.{secret}` key shown exactly once, `authenticate` against
//! the stored hash, scope enforcement, rotation invalidating the old
//! secret, and revoke/reactivate.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::integration::ApiClientInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{api_client_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "API Client Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

#[test]
fn issued_key_authenticates_and_carries_its_scopes() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let issued = api_client_service::create(&conn, &workspace_id, &ApiClientInput { name: "Zapier".into(), scopes: vec!["objects.read".into(), "objects.write".into()], allowed_cidr: None, owner_user_id: None }, Some(&admin_id)).unwrap();
    assert!(issued.api_key.starts_with(&format!("{}.", issued.client.client_id)));

    let authenticated = api_client_service::authenticate(&conn, &workspace_id, &issued.api_key).unwrap();
    assert_eq!(authenticated.id, issued.client.id);
    assert!(api_client_service::has_scope(&authenticated, "objects.read"));
    assert!(!api_client_service::has_scope(&authenticated, "webhooks.manage"));
}

#[test]
fn an_unknown_client_id_a_wrong_secret_and_a_malformed_key_are_all_rejected() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let issued = api_client_service::create(&conn, &workspace_id, &ApiClientInput { name: "Zapier".into(), scopes: vec!["objects.read".into()], allowed_cidr: None, owner_user_id: None }, Some(&admin_id)).unwrap();

    assert!(api_client_service::authenticate(&conn, &workspace_id, "client_bogus.notasecret").is_err());
    assert!(api_client_service::authenticate(&conn, &workspace_id, "not-even-shaped-like-a-key").is_err());
    let wrong_secret = format!("{}.wrongsecret", issued.client.client_id);
    assert!(api_client_service::authenticate(&conn, &workspace_id, &wrong_secret).is_err());
}

#[test]
fn rotate_secret_invalidates_the_old_key_and_issues_a_new_one() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let issued = api_client_service::create(&conn, &workspace_id, &ApiClientInput { name: "Zapier".into(), scopes: vec!["objects.read".into()], allowed_cidr: None, owner_user_id: None }, Some(&admin_id)).unwrap();
    let rotated = api_client_service::rotate_secret(&conn, &workspace_id, &issued.client.id, Some(&admin_id)).unwrap();
    assert_eq!(rotated.client.client_id, issued.client.client_id, "rotation keeps the same client_id");
    assert_ne!(rotated.api_key, issued.api_key);

    assert!(api_client_service::authenticate(&conn, &workspace_id, &issued.api_key).is_err(), "the pre-rotation key must no longer work");
    assert!(api_client_service::authenticate(&conn, &workspace_id, &rotated.api_key).is_ok());
}

#[test]
fn revoked_clients_cannot_authenticate_until_reactivated() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let issued = api_client_service::create(&conn, &workspace_id, &ApiClientInput { name: "Zapier".into(), scopes: vec!["objects.read".into()], allowed_cidr: None, owner_user_id: None }, Some(&admin_id)).unwrap();

    api_client_service::revoke(&conn, &workspace_id, &issued.client.id, Some(&admin_id)).unwrap();
    assert!(api_client_service::authenticate(&conn, &workspace_id, &issued.api_key).is_err());

    api_client_service::reactivate(&conn, &workspace_id, &issued.client.id, Some(&admin_id)).unwrap();
    assert!(api_client_service::authenticate(&conn, &workspace_id, &issued.api_key).is_ok());
}

#[test]
fn an_unknown_scope_is_rejected_at_creation() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let err = api_client_service::create(&conn, &workspace_id, &ApiClientInput { name: "Bad".into(), scopes: vec!["not.a.real.scope".into()], allowed_cidr: None, owner_user_id: None }, Some(&admin_id));
    assert!(err.is_err());
}

#[test]
fn non_admins_cannot_manage_api_clients() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let err = api_client_service::create(&conn, &workspace_id, &ApiClientInput { name: "Zapier".into(), scopes: vec!["objects.read".into()], allowed_cidr: None, owner_user_id: None }, None);
    assert!(err.is_err());
}

#[test]
fn delete_removes_the_client_and_list_reflects_it() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let issued = api_client_service::create(&conn, &workspace_id, &ApiClientInput { name: "Zapier".into(), scopes: vec!["objects.read".into()], allowed_cidr: None, owner_user_id: None }, Some(&admin_id)).unwrap();
    assert_eq!(api_client_service::list_for_workspace(&conn, &workspace_id).unwrap().len(), 1);
    api_client_service::delete(&conn, &workspace_id, &issued.client.id, Some(&admin_id)).unwrap();
    assert!(api_client_service::list_for_workspace(&conn, &workspace_id).unwrap().is_empty());
}
