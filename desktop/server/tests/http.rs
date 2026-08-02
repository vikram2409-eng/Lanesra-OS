use std::net::SocketAddr;

use lanesra_core::db::open_in_memory_db;
use lanesra_server::{build_router, ServerState};
use serde_json::{json, Value};

async fn spawn_server() -> SocketAddr {
    let conn = open_in_memory_db().unwrap();
    let state = ServerState::new(conn);
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
