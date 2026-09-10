//! AI & Agentic Layer, phase 2: proves the `POST /mcp` endpoint end-to-end
//! against a real HTTP client, mirroring `api_v1.rs`'s own test harness
//! and reusing the exact same API-client credential path (spec: MCP
//! auth is the same scoped bearer key the REST API already uses).

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
    let state = ServerState::new(conn, std::env::temp_dir().join("lanesra-mcp-test-unused.sqlite3"), SecurityConfig::default());
    let frontend_dir = std::env::temp_dir().join("lanesra-mcp-test-frontend");
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

fn issue_api_key(state: &SharedState, workspace_id: &str, admin_user_id: &str, scopes: &[&str]) -> String {
    let conn = state.conn.lock().unwrap();
    let issued = api_client_service::create(
        &conn,
        workspace_id,
        &ApiClientInput { name: "Test MCP Client".into(), scopes: scopes.iter().map(|s| s.to_string()).collect(), allowed_cidr: None, owner_user_id: None },
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

async fn rpc(http: &reqwest::Client, addr: SocketAddr, api_key: &str, method: &str, params: Value) -> Value {
    http.post(format!("http://{addr}/mcp"))
        .bearer_auth(api_key)
        .json(&json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn initialize_handshake_returns_protocol_version_and_server_info() {
    let (addr, _state, api_key) = setup_workspace_and_key(&["metadata.read"]).await;
    let body = rpc(&reqwest::Client::new(), addr, &api_key, "initialize", json!({})).await;
    assert_eq!(body["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(body["result"]["serverInfo"]["name"], "lanesra-mcp");
}

#[tokio::test]
async fn tools_list_returns_the_seven_object_dispatcher_tools() {
    let (addr, _state, api_key) = setup_workspace_and_key(&["metadata.read"]).await;
    let body = rpc(&reqwest::Client::new(), addr, &api_key, "tools/list", json!({})).await;
    let names: Vec<String> = body["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect();
    for expected in ["list_objects", "get_object_metadata", "list_records", "get_record", "create_record", "update_record", "archive_record"] {
        assert!(names.contains(&expected.to_string()), "missing tool '{expected}' in {names:?}");
    }
}

#[tokio::test]
async fn missing_bearer_token_is_rejected_at_the_transport_level() {
    let (addr, _state, _key) = setup_workspace_and_key(&["metadata.read"]).await;
    let response = reqwest::Client::new()
        .post(format!("http://{addr}/mcp"))
        .json(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn an_unknown_tool_name_is_a_json_rpc_protocol_error() {
    let (addr, _state, api_key) = setup_workspace_and_key(&["metadata.read", "objects.read"]).await;
    let body = rpc(&reqwest::Client::new(), addr, &api_key, "tools/call", json!({"name": "delete_everything", "arguments": {}})).await;
    assert!(body["error"]["message"].as_str().unwrap().contains("Unknown tool"), "{body:?}");
}

#[tokio::test]
async fn a_tool_call_without_the_required_scope_is_a_json_rpc_error() {
    // metadata.read only - no objects.write - so create_record must be
    // rejected before it ever reaches api_object_service.
    let (addr, _state, api_key) = setup_workspace_and_key(&["metadata.read"]).await;
    let body = rpc(
        &reqwest::Client::new(),
        addr,
        &api_key,
        "tools/call",
        json!({"name": "create_record", "arguments": {"object_key": "Company", "data": {"name": "Acme", "status": "Prospect"}}}),
    )
    .await;
    assert!(body["error"]["message"].as_str().unwrap().contains("objects.write"), "{body:?}");
}

#[tokio::test]
async fn full_crud_round_trip_through_tools_call() {
    let (addr, _state, api_key) = setup_workspace_and_key(&["metadata.read", "objects.read", "objects.write"]).await;
    let http = reqwest::Client::new();

    // Create
    let created = rpc(&http, addr, &api_key, "tools/call", json!({"name": "create_record", "arguments": {"object_key": "Company", "data": {"name": "Acme Corp", "status": "Prospect"}}})).await;
    assert_eq!(created["result"]["isError"], false, "{created:?}");
    let record: Value = serde_json::from_str(created["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    let id = record["id"].as_str().unwrap().to_string();
    assert_eq!(record["name"], "Acme Corp");

    // Get
    let fetched = rpc(&http, addr, &api_key, "tools/call", json!({"name": "get_record", "arguments": {"object_key": "Company", "id": id}})).await;
    let fetched_record: Value = serde_json::from_str(fetched["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(fetched_record["id"], id);

    // Update
    let updated = rpc(&http, addr, &api_key, "tools/call", json!({"name": "update_record", "arguments": {"object_key": "Company", "id": id, "data": {"name": "Acme Corp Updated", "status": "Active Customer"}}})).await;
    let updated_record: Value = serde_json::from_str(updated["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(updated_record["name"], "Acme Corp Updated");

    // List - the record we created shows up
    let listed = rpc(&http, addr, &api_key, "tools/call", json!({"name": "list_records", "arguments": {"object_key": "Company"}})).await;
    let page: Value = serde_json::from_str(listed["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(page["total"], 1);

    // Archive
    let archived = rpc(&http, addr, &api_key, "tools/call", json!({"name": "archive_record", "arguments": {"object_key": "Company", "id": id}})).await;
    assert_eq!(archived["result"]["isError"], false, "{archived:?}");

    // Metadata
    let metadata = rpc(&http, addr, &api_key, "tools/call", json!({"name": "get_object_metadata", "arguments": {"object_key": "Company"}})).await;
    let metadata_value: Value = serde_json::from_str(metadata["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(metadata_value["object_key"], "Company");
}

#[tokio::test]
async fn a_tool_level_error_comes_back_as_an_is_error_result_not_a_protocol_error() {
    let (addr, _state, api_key) = setup_workspace_and_key(&["metadata.read", "objects.read"]).await;
    let body = rpc(&reqwest::Client::new(), addr, &api_key, "tools/call", json!({"name": "get_record", "arguments": {"object_key": "Company", "id": "does-not-exist"}})).await;
    assert!(body["error"].is_null(), "expected a tool-level error, not a JSON-RPC protocol error: {body:?}");
    assert_eq!(body["result"]["isError"], true);
}

#[tokio::test]
async fn a_write_only_document_object_is_rejected_as_not_writable() {
    let (addr, _state, api_key) = setup_workspace_and_key(&["metadata.read", "objects.write"]).await;
    let body = rpc(&reqwest::Client::new(), addr, &api_key, "tools/call", json!({"name": "create_record", "arguments": {"object_key": "Opportunity", "data": {}}})).await;
    assert_eq!(body["result"]["isError"], true, "{body:?}");
}

#[tokio::test]
async fn notifications_initialized_gets_no_json_rpc_response_body() {
    let (addr, _state, api_key) = setup_workspace_and_key(&["metadata.read"]).await;
    let response = reqwest::Client::new()
        .post(format!("http://{addr}/mcp"))
        .bearer_auth(&api_key)
        .json(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::ACCEPTED);
}

#[tokio::test]
async fn a_revoked_client_is_rejected_the_same_as_on_the_rest_api() {
    let (addr, state, api_key, admin_id) = setup_workspace_and_key_with_admin(&["metadata.read"]).await;
    {
        let conn = state.conn.lock().unwrap();
        let workspace_id = workspace_repo::get_current(&conn).unwrap().unwrap().id;
        let clients = api_client_service::list_for_workspace(&conn, &workspace_id).unwrap();
        api_client_service::revoke(&conn, &workspace_id, &clients[0].id, Some(&admin_id)).unwrap();
    }
    let response = reqwest::Client::new()
        .post(format!("http://{addr}/mcp"))
        .bearer_auth(&api_key)
        .json(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}
