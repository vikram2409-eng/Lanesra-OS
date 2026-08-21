//! Integration Hub (spec §7/§8/§9): proves the inbound `/api/v1` REST API
//! end-to-end against a real HTTP client - not just a compiled-but-untested
//! router. Mirrors `http.rs`'s own spawn-a-real-server pattern; the one
//! addition is exposing the `SharedState` so a test can seed an API client
//! directly through `api_client_service` (that command isn't wired into
//! `/api/invoke` yet - see task tracking for the admin-command dispatch
//! pass still to come), the same way an admin would create one from the
//! Integration Hub's own API Access screen once that UI exists.

use std::net::SocketAddr;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::integration::ApiClientInput;
use lanesra_core::repositories::workspace_repo;
use lanesra_core::services::api_client_service;
use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};

async fn spawn_server_with_state() -> (SocketAddr, SharedState) {
    let conn = open_in_memory_db().unwrap();
    let state = ServerState::new(conn, std::env::temp_dir().join("lanesra-api-v1-test-unused.sqlite3"), SecurityConfig::default());
    let frontend_dir = std::env::temp_dir().join("lanesra-server-test-frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state.clone(), frontend_dir);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, state)
}

fn client_with_cookies() -> reqwest::Client {
    reqwest::Client::builder().cookie_store(true).build().unwrap()
}

async fn invoke(client: &reqwest::Client, addr: SocketAddr, command: &str, args: Value) -> Value {
    client.post(format!("http://{addr}/api/invoke/{command}")).json(&args).send().await.unwrap().json().await.unwrap()
}

async fn first_run(client: &reqwest::Client, addr: SocketAddr) -> Value {
    invoke(
        client,
        addr,
        "first_run_setup",
        json!({
            "setup": {
                "business_name": "Test Co", "legal_name": null, "currency_code": "USD", "locale": "en-US",
                "timezone": "UTC", "default_tax_rate_bp": 0, "admin_username": "admin",
                "admin_display_name": "Admin", "admin_password": "supersecretpw", "load_sample_data": false
            }
        }),
    )
    .await
}

/// Issues a real `{client_id}.{secret}` API key with the given scopes,
/// through the exact same `api_client_service::create` an admin UI would
/// call - not a hand-rolled test-only shortcut.
fn issue_api_key(state: &SharedState, workspace_id: &str, admin_user_id: &str, scopes: &[&str]) -> String {
    let conn = state.conn.lock().unwrap();
    let issued = api_client_service::create(
        &conn,
        workspace_id,
        &ApiClientInput { name: "Test Integration".into(), scopes: scopes.iter().map(|s| s.to_string()).collect(), allowed_cidr: None, owner_user_id: None },
        Some(admin_user_id),
    )
    .unwrap();
    issued.api_key
}

async fn setup_workspace_and_key(scopes: &[&str]) -> (SocketAddr, SharedState, String) {
    let (addr, state, key, _admin_id) = setup_workspace_and_key_with_admin(scopes).await;
    (addr, state, key)
}

async fn setup_workspace_and_key_with_admin(scopes: &[&str]) -> (SocketAddr, SharedState, String, String) {
    let (addr, state) = spawn_server_with_state().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    assert_eq!(setup["ok"], true);
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();
    let api_key = issue_api_key(&state, &workspace_id, &admin_id, scopes);
    (addr, state, api_key, admin_id)
}

#[tokio::test]
async fn a_valid_bearer_key_can_list_object_keys() {
    let (addr, _state, api_key) = setup_workspace_and_key(&["metadata.read"]).await;
    let response = reqwest::Client::new().get(format!("http://{addr}/api/v1/objects")).bearer_auth(&api_key).send().await.unwrap();
    assert!(response.status().is_success());
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["ok"], true);
    let keys: Vec<String> = body["data"].as_array().unwrap().iter().map(|o| o["object_key"].as_str().unwrap().to_string()).collect();
    assert!(keys.contains(&"Company".to_string()));
}

#[tokio::test]
async fn missing_authorization_header_is_rejected() {
    let (addr, _state, _key) = setup_workspace_and_key(&["metadata.read"]).await;
    let response = reqwest::Client::new().get(format!("http://{addr}/api/v1/objects")).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_malformed_or_unknown_key_is_rejected() {
    let (addr, _state, _key) = setup_workspace_and_key(&["metadata.read"]).await;
    let response = reqwest::Client::new().get(format!("http://{addr}/api/v1/objects")).bearer_auth("client_bogus.notasecret").send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_key_without_the_required_scope_is_forbidden() {
    // Only metadata.read - no objects.write - so a create attempt must be
    // rejected before it ever reaches api_object_service.
    let (addr, _state, api_key) = setup_workspace_and_key(&["metadata.read"]).await;
    let response = reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/objects/Company/records"))
        .bearer_auth(&api_key)
        .json(&json!({"name": "Acme", "status": "Prospect"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn full_crud_round_trip_through_the_generic_object_api() {
    let (addr, _state, api_key) = setup_workspace_and_key(&["objects.read", "objects.write", "metadata.read"]).await;
    let http = reqwest::Client::new();

    // Create
    let created: Value = http
        .post(format!("http://{addr}/api/v1/objects/Company/records"))
        .bearer_auth(&api_key)
        .json(&json!({"name": "Acme Corp", "status": "Prospect"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(created["ok"], true, "{created:?}");
    let id = created["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(created["data"]["name"], "Acme Corp");

    // Get
    let fetched: Value = http.get(format!("http://{addr}/api/v1/objects/Company/records/{id}")).bearer_auth(&api_key).send().await.unwrap().json().await.unwrap();
    assert_eq!(fetched["data"]["id"], id);

    // Update (PATCH)
    let updated: Value = http
        .patch(format!("http://{addr}/api/v1/objects/Company/records/{id}"))
        .bearer_auth(&api_key)
        .json(&json!({"name": "Acme Corp Updated", "status": "Active Customer"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(updated["ok"], true, "{updated:?}");
    assert_eq!(updated["data"]["name"], "Acme Corp Updated");

    // List - the record we created shows up
    let listed: Value = http.get(format!("http://{addr}/api/v1/objects/Company/records")).bearer_auth(&api_key).send().await.unwrap().json().await.unwrap();
    assert_eq!(listed["data"]["total"], 1);

    // Archive (DELETE)
    let archived = http.delete(format!("http://{addr}/api/v1/objects/Company/records/{id}")).bearer_auth(&api_key).send().await.unwrap();
    assert!(archived.status().is_success());

    // Metadata
    let metadata: Value = http.get(format!("http://{addr}/api/v1/objects/Company/metadata")).bearer_auth(&api_key).send().await.unwrap().json().await.unwrap();
    assert_eq!(metadata["data"]["object_key"], "Company");
}

#[tokio::test]
async fn a_revoked_client_is_rejected() {
    let (addr, state, api_key, admin_id) = setup_workspace_and_key_with_admin(&["metadata.read"]).await;
    // Works before revocation.
    let before = reqwest::Client::new().get(format!("http://{addr}/api/v1/objects")).bearer_auth(&api_key).send().await.unwrap();
    assert!(before.status().is_success());

    {
        let conn = state.conn.lock().unwrap();
        let workspace_id = workspace_repo::get_current(&conn).unwrap().unwrap().id;
        let clients = api_client_service::list_for_workspace(&conn, &workspace_id).unwrap();
        api_client_service::revoke(&conn, &workspace_id, &clients[0].id, Some(&admin_id)).unwrap();
    }

    let after = reqwest::Client::new().get(format!("http://{addr}/api/v1/objects")).bearer_auth(&api_key).send().await.unwrap();
    assert_eq!(after.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rate_limit_is_enforced_per_client() {
    let (addr, state, api_key) = setup_workspace_and_key(&["metadata.read"]).await;
    {
        let conn = state.conn.lock().unwrap();
        let workspace_id = workspace_repo::get_current(&conn).unwrap().unwrap().id;
        lanesra_core::repositories::integration_settings_repo::ensure_default(&conn, &workspace_id).unwrap();
        lanesra_core::repositories::integration_settings_repo::update(&conn, &workspace_id, 2, 3000, 90, 7, false, None).unwrap();
    }
    let http = reqwest::Client::new();
    let mut saw_429 = false;
    for _ in 0..5 {
        let response = http.get(format!("http://{addr}/api/v1/objects")).bearer_auth(&api_key).send().await.unwrap();
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            saw_429 = true;
            break;
        }
    }
    assert!(saw_429, "expected at least one 429 once the per-minute limit (2) was exceeded");
}
