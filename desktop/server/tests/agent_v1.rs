//! AI & Agentic Layer, Phase 6b: the inbound webhook Trigger
//! (`server/src/agent_v1.rs`) end-to-end against a real HTTP client -
//! Bearer-authenticated and scope-checked exactly like every other
//! `/api/v1` route (mirrors `api_v1.rs`'s own test file), and running
//! **inline** (unlike a schedule Trigger or the `run_ai_agent` Workflow
//! Automation action, both of which only ever enqueue - see
//! `core/tests/ai_agent_orchestration.rs` for that half).
//!
//! Uses a real on-disk SQLite file (not the in-memory DB `api_v1.rs`'s own
//! test file uses), since `agent_v1`'s handler - like `admin_actions.rs`'s
//! own genuinely-async routes - opens its own connection to the same file
//! via `run_with_own_connection` rather than sharing `ServerState.conn`.
//! See `tests/admin_actions.rs`'s identical doc comment.

use std::io::{BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::net::TcpListener;

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::integration::ApiClientInput;
use lanesra_core::services::api_client_service;
use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};

async fn spawn_file_backed_server() -> (SocketAddr, SharedState, tempfile::TempDir) {
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
    (addr, state, dir)
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
                "business_name": "Agent Trigger Co", "legal_name": null, "currency_code": "USD", "locale": "en-US",
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
        &ApiClientInput { name: "Agent Trigger Client".into(), scopes: scopes.iter().map(|s| s.to_string()).collect(), allowed_cidr: None, owner_user_id: Some(admin_user_id.to_string()) },
        Some(admin_user_id),
    )
    .unwrap();
    issued.api_key
}

/// Minimal raw-socket stand-in for the Anthropic Messages API - always
/// answers one canned tool-free text reply, same shape `core/tests/
/// ai_agent_foundry.rs`'s own stub uses, just without needing to inspect
/// what was sent (nothing here needs that).
fn spawn_text_stub(text: &str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let body = serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            let _ = reader.read_line(&mut line);
            loop {
                let mut l = String::new();
                match reader.read_line(&mut l) {
                    Ok(0) => break,
                    Ok(_) if l == "\r\n" || l.trim().is_empty() => break,
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

#[tokio::test]
async fn a_webhook_trigger_call_runs_the_agent_inline_and_returns_its_answer() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    assert_eq!(setup["ok"], true, "{setup:?}");
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    let ai_port = spawn_text_stub("Handled via webhook.");
    let saved = invoke(
        &client, addr, "save_ai_settings",
        json!({"input": {"provider": "anthropic", "base_url": format!("http://127.0.0.1:{ai_port}"), "model": "claude-haiku-4-5-20251001", "api_key": "sk-ant-test"}}),
    )
    .await;
    assert_eq!(saved["ok"], true, "{saved:?}");

    let created = invoke(
        &client, addr, "create_ai_agent",
        json!({"input": {"name": "Webhook Agent", "description": null, "icon": "🤖", "system_prompt": "You handle webhook triggers.", "action_names": [], "delegate_agent_ids": [], "skill_ids": []}}),
    )
    .await;
    assert_eq!(created["ok"], true, "{created:?}");
    let agent_id = created["data"]["id"].as_str().unwrap().to_string();

    let api_key = issue_api_key(&state, &workspace_id, &admin_id, &["agents.trigger"]);

    let response = reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/agents/{agent_id}/trigger"))
        .bearer_auth(&api_key)
        .json(&json!({"input": "hello from a webhook"}))
        .send()
        .await
        .unwrap();
    let status = response.status();
    let text = response.text().await.unwrap();
    assert!(status.is_success(), "status={status:?} body={text}");
    let body: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(body["ok"], true, "{body:?}");
    assert_eq!(body["data"]["status"], "succeeded", "{body:?}");
    assert_eq!(body["data"]["steps"][0]["output_text"], "Handled via webhook.");
    assert_eq!(body["data"]["triggered_by"], "webhook");
}

#[tokio::test]
async fn a_webhook_trigger_call_without_the_scope_is_forbidden() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    let created = invoke(
        &client, addr, "create_ai_agent",
        json!({"input": {"name": "No Scope Agent", "description": null, "icon": "🤖", "system_prompt": "Hi.", "action_names": [], "delegate_agent_ids": [], "skill_ids": []}}),
    )
    .await;
    let agent_id = created["data"]["id"].as_str().unwrap().to_string();

    // Only metadata.read - no agents.trigger.
    let api_key = issue_api_key(&state, &workspace_id, &admin_id, &["metadata.read"]);
    let response = reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/agents/{agent_id}/trigger"))
        .bearer_auth(&api_key)
        .json(&json!({"input": "hi"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
}
