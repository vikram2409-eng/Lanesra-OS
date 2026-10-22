//! WP-0189: integration administration on the REST API, over real HTTP.
//! `admin.integration.read` lists metadata and never a secret;
//! `admin.integration.manage` pauses, reactivates and runs, and cannot list.
//!
//! Covers: REQ-INT-005, REQ-INT-006 (WP-0189)

use std::net::SocketAddr;

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::integration::ApiClientInput;
use lanesra_core::services::api_client_service;
use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use reqwest::Method;
use serde_json::{json, Value};

struct Ws {
    addr: SocketAddr,
    state: SharedState,
    admin: reqwest::Client,
    workspace_id: String,
    admin_id: String,
    _dir: tempfile::TempDir,
}

async fn workspace() -> Ws {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("lanesra.sqlite3");
    let conn = open_workspace_db(&db_path).unwrap();
    let state = ServerState::new(conn, db_path, SecurityConfig::default());
    let frontend_dir = dir.path().join("frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state.clone(), frontend_dir);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let admin = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let setup = invoke(
        &admin,
        addr,
        "first_run_setup",
        json!({"setup": {
            "business_name": "Ops Co", "legal_name": null, "currency_code": "USD",
            "locale": "en-US", "timezone": "UTC", "default_tax_rate_bp": 0,
            "admin_username": "admin", "admin_display_name": "Admin",
            "admin_password": "supersecretpw", "load_sample_data": false
        }}),
    )
    .await;
    assert_eq!(setup["ok"], true, "{setup}");
    Ws {
        addr,
        state,
        admin,
        workspace_id: setup["data"][0]["id"].as_str().unwrap().to_string(),
        admin_id: setup["data"][1]["id"].as_str().unwrap().to_string(),
        _dir: dir,
    }
}

async fn invoke(c: &reqwest::Client, addr: SocketAddr, command: &str, args: Value) -> Value {
    c.post(format!("http://{addr}/api/invoke/{command}"))
        .json(&args)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

fn api_key(ws: &Ws, scopes: &[&str]) -> String {
    let conn = ws.state.conn.lock().unwrap();
    api_client_service::create(
        &conn,
        &ws.workspace_id,
        &ApiClientInput {
            name: format!("Ops {}", scopes.join("+")),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
            allowed_cidr: None,
            owner_user_id: None,
        },
        Some(&ws.admin_id),
    )
    .unwrap()
    .api_key
}

async fn call(addr: SocketAddr, key: &str, method: Method, path: &str) -> (u16, Value) {
    let response = reqwest::Client::new()
        .request(method, format!("http://{addr}{path}"))
        .bearer_auth(key)
        .send()
        .await
        .unwrap();
    let status = response.status().as_u16();
    let text = response.text().await.unwrap();
    (
        status,
        serde_json::from_str(&text).unwrap_or(json!({"raw": text})),
    )
}

const OPENAPI: &str = r#"{"openapi":"3.0.0","info":{"title":"Items","version":"1"},
  "paths":{"/items":{"get":{"operationId":"listItems","summary":"List items"}}}}"#;

/// Seeds a connection carrying a secret, credentials in its URL and a token
/// header in its config; a connector; an external object; and a job. Returns
/// (connection id, job id, the planted secret values).
async fn seed(ws: &Ws) -> (String, String, Vec<String>) {
    let planted = format!("planted-{}", ["s3cr3t", "v4lue"].concat());
    let url_password = ["url", "pass"].join("-");
    let header_token = ["header", "token", "x9"].join("-");
    let connection = invoke(
        &ws.admin,
        ws.addr,
        "create_connection",
        json!({"input": {
            "name": "Upstream CRM", "connection_type": "rest",
            "base_url": format!("http://svc:{url_password}@127.0.0.1:9/api?key={header_token}"),
            "auth_mode": "bearer", "secret_value": planted,
            "config_json": format!(r#"{{"default_headers":[["X-Token","{header_token}"]]}}"#),
            "owner_user_id": null
        }}),
    )
    .await;
    assert_eq!(connection["ok"], true, "{connection}");
    let connection_id = connection["data"]["id"].as_str().unwrap().to_string();
    let connector = invoke(
        &ws.admin,
        ws.addr,
        "import_connector",
        json!({"input": {"name": "Items", "description": null, "spec_text": OPENAPI,
                          "spec_format": "json", "selected_operation_ids": ["listItems"]}}),
    )
    .await;
    assert_eq!(connector["ok"], true, "{connector}");
    let external = invoke(
        &ws.admin,
        ws.addr,
        "create_external_object",
        json!({"input": {"object_key": "crm_companies", "display_name": "CRM Companies",
                          "connection_id": connection_id, "resource_path": "/companies",
                          "field_map": [], "cache_ttl_seconds": null}}),
    )
    .await;
    assert_eq!(external["ok"], true, "{external}");
    let job = invoke(
        &ws.admin,
        ws.addr,
        "create_integration_job",
        json!({"input": {"name": "Sync CRM", "external_object_id": external["data"]["id"],
                          "target_object_key": "Company", "match_key": "name",
                          "cursor_field": null, "interval_minutes": 30}}),
    )
    .await;
    assert_eq!(job["ok"], true, "{job}");
    (
        connection_id,
        job["data"]["id"].as_str().unwrap().to_string(),
        vec![planted, url_password, header_token],
    )
}

#[tokio::test]
async fn read_lists_everything_connected_without_any_secret() {
    // Covers: REQ-INT-005 (WP-0189 AC-1), REQ-INT-006
    let ws = workspace().await;
    let (connection_id, job_id, planted) = seed(&ws).await;
    let key = api_key(&ws, &["admin.integration.read"]);
    let mut seen = String::new();
    for path in [
        "/api/v1/integration/connections",
        "/api/v1/integration/connectors",
        "/api/v1/integration/jobs",
        "/api/v1/integration/api-clients",
    ] {
        let (status, body) = call(ws.addr, &key, Method::GET, path).await;
        assert_eq!(status, 200, "{path}: {body}");
        assert!(
            !body["data"].as_array().unwrap().is_empty(),
            "{path}: {body}"
        );
        seen.push_str(&body.to_string());
    }
    assert!(seen.contains(&connection_id) && seen.contains(&job_id));
    for value in &planted {
        assert!(!seen.contains(value.as_str()), "leaked {value}");
    }
    for column in [
        "secret_id",
        "ciphertext",
        "nonce",
        "secret_hash",
        "key_hash",
        "api_key",
        "signing_secret",
        "config_json",
        "secret_value",
    ] {
        assert!(!seen.contains(&format!("\"{column}\"")), "exposes {column}");
    }
    let (_, connections) = call(
        ws.addr,
        &key,
        Method::GET,
        "/api/v1/integration/connections",
    )
    .await;
    assert_eq!(connections["data"][0]["base_url"], "http://127.0.0.1:9/api");
    assert_eq!(connections["data"][0]["has_secret"], true);
}

#[tokio::test]
async fn manage_pauses_reactivates_and_runs_and_the_run_is_logged() {
    // Covers: REQ-INT-005 (WP-0189 AC-2), REQ-INT-006
    let ws = workspace().await;
    let (connection_id, job_id, _) = seed(&ws).await;
    let key = api_key(&ws, &["admin.integration.manage"]);
    let path = |what: &str, id: &str, verb: &str| format!("/api/v1/integration/{what}/{id}/{verb}");

    let (status, paused) = call(ws.addr, &key, Method::POST, &path("jobs", &job_id, "pause")).await;
    assert_eq!(status, 200, "{paused}");
    assert_eq!(paused["data"]["status"], "paused");
    let (_, active) = call(
        ws.addr,
        &key,
        Method::POST,
        &path("jobs", &job_id, "reactivate"),
    )
    .await;
    assert_eq!(active["data"]["status"], "active");

    // A paused connection makes no outbound call: the job run is refused
    // before it reaches the network, and the attempt is still logged.
    let (_, paused) = call(
        ws.addr,
        &key,
        Method::POST,
        &path("connections", &connection_id, "pause"),
    )
    .await;
    assert_eq!(paused["data"]["status"], "paused", "{paused}");
    let (status, run) = call(ws.addr, &key, Method::POST, &path("jobs", &job_id, "run")).await;
    assert_eq!(status, 200, "{run}");
    assert_ne!(run["data"]["status"], "success", "{run}");
    assert!(run.to_string().contains("paused"), "{run}");
    let (_, active) = call(
        ws.addr,
        &key,
        Method::POST,
        &path("connections", &connection_id, "reactivate"),
    )
    .await;
    assert_eq!(active["data"]["status"], "active");

    let conn = ws.state.conn.lock().unwrap();
    let logged: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM integration_executions WHERE execution_type = 'integration_job'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        logged >= 1,
        "the job run is in the integration log ({logged})"
    );
    let audited: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE event_type LIKE 'integration_%' AND user_id IS NULL AND details_json LIKE '%api_client_id%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        audited, 5,
        "pause, reactivate, run and the connection pause and reactivate"
    );
}

#[tokio::test]
async fn read_and_manage_do_not_grant_each_other() {
    // Covers: REQ-INT-005 (WP-0189 AC-3)
    let ws = workspace().await;
    let (connection_id, job_id, _) = seed(&ws).await;
    let read = api_key(&ws, &["admin.integration.read"]);
    let manage = api_key(
        &ws,
        &[
            "admin.integration.manage",
            "objects.write",
            "webhooks.manage",
        ],
    );
    for path in [
        "/api/v1/integration/connections",
        "/api/v1/integration/connectors",
        "/api/v1/integration/jobs",
        "/api/v1/integration/api-clients",
    ] {
        assert_eq!(
            call(ws.addr, &manage, Method::GET, path).await.0,
            403,
            "{path}"
        );
    }
    for path in [
        format!("/api/v1/integration/connections/{connection_id}/pause"),
        format!("/api/v1/integration/connections/{connection_id}/reactivate"),
        format!("/api/v1/integration/jobs/{job_id}/pause"),
        format!("/api/v1/integration/jobs/{job_id}/reactivate"),
        format!("/api/v1/integration/jobs/{job_id}/run"),
    ] {
        assert_eq!(
            call(ws.addr, &read, Method::POST, &path).await.0,
            403,
            "{path}"
        );
    }
}
