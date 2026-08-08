use std::net::SocketAddr;
use std::path::PathBuf;

use lanesra_core::db::open_in_memory_db;
use lanesra_server::{build_router, ServerState};
use serde_json::{json, Value};

async fn spawn_server() -> SocketAddr {
    let conn = open_in_memory_db().unwrap();
    // db_path is unused by every test below - none of them exercise
    // restore_backup, the only command that touches it - so an in-memory
    // connection with a placeholder path is fine here.
    let state = ServerState::new(conn, std::env::temp_dir().join("lanesra-http-test-unused.sqlite3"));
    let frontend_dir = std::env::temp_dir().join("lanesra-server-test-frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state, frontend_dir);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

/// Unlike `spawn_server`, this uses a real file-backed database rather than
/// `:memory:` - required for backup/restore tests, since restore works by
/// replacing the database *file* out from under the live connection.
async fn spawn_file_backed_server() -> (SocketAddr, PathBuf) {
    let db_path = std::env::temp_dir().join(format!(
        "lanesra-http-test-{}.sqlite3",
        lanesra_core::domain::ids::new_uuid()
    ));
    let conn = lanesra_core::db::open_workspace_db(&db_path).unwrap();
    let state = ServerState::new(conn, db_path.clone());
    let frontend_dir = std::env::temp_dir().join("lanesra-server-test-frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state, frontend_dir);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, db_path)
}

fn client_with_cookies() -> reqwest::Client {
    reqwest::Client::builder().cookie_store(true).build().unwrap()
}

async fn invoke(client: &reqwest::Client, addr: SocketAddr, command: &str, args: Value) -> Value {
    client
        .post(format!("http://{addr}/api/invoke/{command}"))
        .json(&args)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn first_run(client: &reqwest::Client, addr: SocketAddr, username: &str, password: &str) -> Value {
    invoke(
        client,
        addr,
        "first_run_setup",
        json!({
            "setup": {
                "business_name": "Test Co",
                "legal_name": null,
                "currency_code": "USD",
                "locale": "en-US",
                "timezone": "UTC",
                "default_tax_rate_bp": 0,
                "admin_username": username,
                "admin_display_name": "Admin",
                "admin_password": password,
                "load_sample_data": false
            }
        }),
    )
    .await
}

#[tokio::test]
async fn health_check_responds() {
    let addr = spawn_server().await;
    let response = reqwest::get(format!("http://{addr}/api/health")).await.unwrap();
    assert!(response.status().is_success());
}

#[tokio::test]
async fn unauthenticated_requests_are_rejected() {
    let addr = spawn_server().await;
    let client = client_with_cookies();
    first_run(&client, addr, "admin", "supersecretpw").await;

    // A fresh client with no session cookie must not be able to read data,
    // even though a workspace now exists.
    let anon = reqwest::Client::new();
    let body = invoke(&anon, addr, "list_companies", json!({})).await;
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["kind"], "validation");
}

#[tokio::test]
async fn login_grants_a_session_that_can_read_data() {
    let addr = spawn_server().await;
    let setup_client = client_with_cookies();
    first_run(&setup_client, addr, "admin", "supersecretpw").await;

    let client = client_with_cookies();
    let login = invoke(
        &client,
        addr,
        "login",
        json!({"credentials": {"username": "admin", "password": "supersecretpw"}}),
    )
    .await;
    assert_eq!(login["ok"], true);

    let companies = invoke(&client, addr, "list_companies", json!({})).await;
    assert_eq!(companies["ok"], true);
    assert!(companies["data"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn two_sessions_are_independent() {
    let addr = spawn_server().await;
    let setup_client = client_with_cookies();
    first_run(&setup_client, addr, "admin", "supersecretpw").await;

    let session_a = client_with_cookies();
    invoke(
        &session_a,
        addr,
        "login",
        json!({"credentials": {"username": "admin", "password": "supersecretpw"}}),
    )
    .await;

    let session_b = client_with_cookies();
    let create_user = invoke(
        &session_a,
        addr,
        "create_user",
        json!({"input": {"username": "morgan", "display_name": "Morgan", "password": "anothersecretpw", "roles": ["Sales"]}}),
    )
    .await;
    assert_eq!(create_user["ok"], true);

    invoke(
        &session_b,
        addr,
        "login",
        json!({"credentials": {"username": "morgan", "password": "anothersecretpw"}}),
    )
    .await;

    let who_a = invoke(&session_a, addr, "current_user", json!({})).await;
    let who_b = invoke(&session_b, addr, "current_user", json!({})).await;
    assert_eq!(who_a["data"]["username"], "admin");
    assert_eq!(who_b["data"]["username"], "morgan");

    // Logging session A out must not affect session B.
    let logout_a = invoke(&session_a, addr, "logout", json!({})).await;
    assert_eq!(logout_a["ok"], true);

    let who_a_after = invoke(&session_a, addr, "current_user", json!({})).await;
    assert!(who_a_after["data"].is_null());

    let who_b_after = invoke(&session_b, addr, "current_user", json!({})).await;
    assert_eq!(who_b_after["data"]["username"], "morgan");
}

#[tokio::test]
async fn only_an_administrator_can_create_users() {
    let addr = spawn_server().await;
    let setup_client = client_with_cookies();
    first_run(&setup_client, addr, "admin", "supersecretpw").await;

    let admin_session = client_with_cookies();
    invoke(
        &admin_session,
        addr,
        "login",
        json!({"credentials": {"username": "admin", "password": "supersecretpw"}}),
    )
    .await;
    invoke(
        &admin_session,
        addr,
        "create_user",
        json!({"input": {"username": "sam", "display_name": "Sam", "password": "anothersecretpw", "roles": ["Sales"]}}),
    )
    .await;

    let sam_session = client_with_cookies();
    invoke(
        &sam_session,
        addr,
        "login",
        json!({"credentials": {"username": "sam", "password": "anothersecretpw"}}),
    )
    .await;

    let result = invoke(
        &sam_session,
        addr,
        "create_user",
        json!({"input": {"username": "eve", "display_name": "Eve", "password": "anothersecretpw", "roles": ["Sales"]}}),
    )
    .await;
    assert_eq!(result["ok"], false);
}

fn company_args(name: &str) -> Value {
    json!({"input": {
        "name": name, "status": "Prospect", "owner_user_id": null, "tax_number": null,
        "billing_address": null, "shipping_address": null, "tags": null, "notes": null
    }})
}

#[tokio::test]
async fn backup_then_restore_reverts_data_over_http() {
    let (addr, db_path) = spawn_file_backed_server().await;
    let admin = client_with_cookies();
    first_run(&admin, addr, "admin", "supersecretpw").await;
    invoke(&admin, addr, "login", json!({"credentials": {"username": "admin", "password": "supersecretpw"}})).await;

    invoke(&admin, addr, "create_company", company_args("Acme Ltd")).await;

    let backup = invoke(&admin, addr, "create_backup", json!({})).await;
    assert_eq!(backup["ok"], true);
    let package_base64 = backup["data"]["package_base64"].as_str().unwrap().to_string();

    invoke(&admin, addr, "create_company", company_args("Widgets Inc")).await;
    let before_restore = invoke(&admin, addr, "list_companies", json!({})).await;
    assert_eq!(before_restore["data"].as_array().unwrap().len(), 2);

    let restore = invoke(&admin, addr, "restore_backup", json!({"packageBase64": package_base64})).await;
    assert_eq!(restore["ok"], true, "restore failed: {restore:?}");

    let after_restore = invoke(&admin, addr, "list_companies", json!({})).await;
    let companies = after_restore["data"].as_array().unwrap();
    assert_eq!(companies.len(), 1);
    assert_eq!(companies[0]["name"], "Acme Ltd");

    // The admin's own session was created before the backup was taken, so
    // it's part of the snapshot and should still resolve after restore.
    let who = invoke(&admin, addr, "current_user", json!({})).await;
    assert_eq!(who["data"]["username"], "admin");

    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn only_an_administrator_can_restore_a_backup() {
    let (addr, db_path) = spawn_file_backed_server().await;
    let admin = client_with_cookies();
    first_run(&admin, addr, "admin", "supersecretpw").await;
    invoke(&admin, addr, "login", json!({"credentials": {"username": "admin", "password": "supersecretpw"}})).await;

    invoke(
        &admin,
        addr,
        "create_user",
        json!({"input": {"username": "sam", "display_name": "Sam", "password": "anothersecretpw", "roles": ["Sales"]}}),
    )
    .await;
    let backup = invoke(&admin, addr, "create_backup", json!({})).await;
    let package_base64 = backup["data"]["package_base64"].as_str().unwrap().to_string();

    let sam = client_with_cookies();
    invoke(&sam, addr, "login", json!({"credentials": {"username": "sam", "password": "anothersecretpw"}})).await;
    let restore = invoke(&sam, addr, "restore_backup", json!({"packageBase64": package_base64})).await;
    assert_eq!(restore["ok"], false, "a non-administrator must not be able to restore a backup");

    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn self_service_password_change_over_http() {
    let addr = spawn_server().await;
    let admin = client_with_cookies();
    first_run(&admin, addr, "admin", "supersecretpw").await;
    invoke(&admin, addr, "login", json!({"credentials": {"username": "admin", "password": "supersecretpw"}})).await;

    let wrong_current = invoke(
        &admin,
        addr,
        "change_my_password",
        json!({"input": {"current_password": "not the real one", "new_password": "brandnewsecretpw"}}),
    )
    .await;
    assert_eq!(wrong_current["ok"], false);

    let changed = invoke(
        &admin,
        addr,
        "change_my_password",
        json!({"input": {"current_password": "supersecretpw", "new_password": "brandnewsecretpw"}}),
    )
    .await;
    assert_eq!(changed["ok"], true);

    let new_login = client_with_cookies();
    let login_result = invoke(
        &new_login,
        addr,
        "login",
        json!({"credentials": {"username": "admin", "password": "brandnewsecretpw"}}),
    )
    .await;
    assert_eq!(login_result["ok"], true);

    let old_login = client_with_cookies();
    let old_login_result = invoke(
        &old_login,
        addr,
        "login",
        json!({"credentials": {"username": "admin", "password": "supersecretpw"}}),
    )
    .await;
    assert_eq!(old_login_result["ok"], false);
}
