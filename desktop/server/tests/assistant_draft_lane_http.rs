//! AI Agent Platform v2, Phase 7: describe a process - the Admin Assistant
//! drafts a lane (WP-0715 AC-4, AC-5) - the Team Workspace server exposes
//! the same draft operation as the desktop Tauri command, over the same
//! service code, through the async admin route
//! `POST /api/admin/process-lanes/drafts` (the sync `/api/invoke`
//! dispatcher cannot await a model call, so the draft lives with the
//! other async admin actions, like starting a run).
//!
//! The model reply is scripted (WP-0005) on this test's own thread - the
//! scripted provider keeps its queue in thread-local storage, so a reply
//! scripted here cannot reach a model call the server runs on one of its
//! own worker threads. The happy path over HTTP is therefore proven in
//! two halves that meet in the same store: the draft is made by calling
//! the same service the route calls, on this thread, and then listed over
//! HTTP - the draft the service makes is the draft the workspace API
//! serves. The route itself is proven over HTTP for what needs no model:
//! the session gate, the input validation, and failing closed with no
//! provider configured.
//!
//! Covers: REQ-LANE-022 (WP-0715 AC-4, AC-5)

use std::net::SocketAddr;

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::services::ai_service::TokenUsage;
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{ai_service, assistant_draft_lane_service, secret_service};
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
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
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

async fn first_run(client: &reqwest::Client, addr: SocketAddr) -> Value {
    invoke(
        client,
        addr,
        "first_run_setup",
        json!({
            "setup": {
                "business_name": "Draft Lane HTTP Co", "legal_name": null, "currency_code": "USD", "locale": "en-US",
                "timezone": "UTC", "default_tax_rate_bp": 0, "admin_username": "admin",
                "admin_display_name": "Admin", "admin_password": "supersecretpw", "load_sample_data": false
            }
        }),
    )
    .await
}

async fn draft(
    client: &reqwest::Client,
    addr: SocketAddr,
    description: &str,
    object_key: &str,
) -> Value {
    client
        .post(format!("http://{addr}/api/admin/process-lanes/drafts"))
        .json(&json!({"description": description, "object_key": object_key}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()
}

fn text_reply(text: &str) -> ScriptedResponse {
    ScriptedResponse::Text {
        text: text.into(),
        usage: TokenUsage {
            input_tokens: 50,
            output_tokens: 200,
        },
    }
}

/// The lane a model would draft for the receiving description: form steps
/// for the person's work, one check step where the description states
/// something to verify. Mirrors the core test's reply.
fn drafted_lane_json() -> String {
    json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-draft-receive-purchase-order",
            "name": "Receive a purchase order",
            "description": "Receive a purchase order at the Toronto warehouse.",
            "object_key": "purchase_order",
            "version": 1,
            "status": "draft",
            "run_data": {"fields": [
                {"key": "carrier", "type": "string", "required": true, "description": "Carrier who delivered the shipment."},
                {"key": "received_date", "type": "date", "required": true, "description": "Date the shipment arrived."}
            ]},
            "stages": [
                {"id": "verify", "name": "Verify", "steps": [
                    {"id": "confirm-arrival", "name": "Confirm shipment arrived", "kind": "form", "role": "role_wh_receiver", "required": true,
                     "description": "Confirm the truck and delivery.",
                     "fields": [
                        {"key": "carrier", "label": "Carrier", "type": "string", "required": true, "binds_to": "run_data.carrier"},
                        {"key": "received_date", "label": "Received date", "type": "date", "required": true, "binds_to": "run_data.received_date"}
                     ]}
                ]},
                {"id": "record-receipt", "name": "Record receipt", "steps": [
                    {"id": "quantities-match", "name": "Quantities match", "kind": "check", "required": true,
                     "description": "Every line fully received.",
                     "check": {"scope": "anchor", "message": "A line is short.", "condition": {"field": "$record.status", "operator": "equals", "value": "OPEN"}},
                     "branches": [
                        {"on": "pass", "end": {"outcome": "done"}},
                        {"on": "fail", "end": {"outcome": "held"}}
                     ]}
                ]}
            ]
        }
    })
    .to_string()
}

/// Covers: REQ-LANE-022 (WP-0715 AC-4, AC-5)
#[tokio::test]
async fn draft_route_is_gated_validated_and_fails_closed_without_a_provider() {
    let (addr, _state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    first_run(&client, addr).await;

    // No session cookie: the route refuses before it ever reaches the
    // service.
    let bare = reqwest::Client::new();
    let denied = bare
        .post(format!("http://{addr}/api/admin/process-lanes/drafts"))
        .json(&json!({"description": "Receive a purchase order.", "object_key": "purchase_order"}))
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(denied.json::<Value>().await.unwrap()["ok"], false);

    // An empty description is refused before any model call - the same
    // validation the core test proves, over HTTP.
    let refused = draft(&client, addr, "   ", "purchase_order").await;
    assert_eq!(refused["ok"], false);
    assert!(refused["error"]
        .as_str()
        .unwrap()
        .contains("describe the process"));

    // A real description with no provider configured fails closed: the
    // route reaches the model call and the service refuses without a key
    // rather than drafting from nowhere.
    let no_provider = draft(
        &client,
        addr,
        "Receive a purchase order at the Toronto warehouse.",
        "purchase_order",
    )
    .await;
    assert_eq!(no_provider["ok"], false);
    assert!(no_provider["error"]
        .as_str()
        .unwrap()
        .contains("Configure an AI provider key first"));
}

/// Covers: REQ-LANE-022 (WP-0715 AC-4, AC-5)
#[tokio::test]
async fn draft_made_by_the_service_is_the_draft_the_workspace_api_serves() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();
    let master_key =
        secret_service::resolve_master_key(&state.db_path.with_file_name("secret.key")).unwrap();

    // The settings row is created as another provider (the settings API
    // cannot select `scripted`) and switched underneath, as in the core
    // tests - then the model reply is scripted on this thread.
    {
        let conn = state.conn.lock().unwrap();
        ai_service::save_settings(
            &conn,
            &workspace_id,
            &master_key,
            &AiSettingsInput {
                provider: "anthropic".into(),
                base_url: None,
                model: "default-model".into(),
                api_key: Some("not-a-real-key".into()),
            },
            Some(&admin_id),
        )
        .unwrap();
        conn.execute(
            "UPDATE ai_settings SET provider = 'scripted' WHERE workspace_id = ?1",
            [&workspace_id],
        )
        .unwrap();
    }
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text_reply(&drafted_lane_json())]);

    // The draft is made by the same service the route and the desktop
    // command both call - on this thread, where the scripted reply lives.
    let lane = {
        // Open a fresh connection to the same file DB rather than holding the
        // shared `state.conn` mutex guard across the `.await`
        // (clippy::await_holding_lock). The scripted provider is thread-local
        // and `block_on` runs on this thread, so the draft still sees the
        // scripted reply.
        let conn = open_workspace_db(&state.db_path).unwrap();
        assistant_draft_lane_service::draft_lane(
            &conn,
            &workspace_id,
            &master_key,
            Some(&admin_id),
            "Receive a purchase order at the Toronto warehouse.",
            "purchase_order",
        )
        .await
        .unwrap()
    };
    assert_eq!(lane.lane_key, "lane-draft-receive-purchase-order");
    assert_eq!(lane.versions[0].status, "draft");

    // AC-4: the same draft is what the workspace API serves over HTTP -
    // a draft, version 1, no compiled graph.
    let listed = invoke(&client, addr, "list_process_lanes", json!({})).await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);
    assert_eq!(
        listed["data"][0]["lane_key"],
        "lane-draft-receive-purchase-order"
    );
    assert_eq!(listed["data"][0]["versions"][0]["version"], 1);
    assert_eq!(listed["data"][0]["versions"][0]["status"], "draft");
    assert!(listed["data"][0]["versions"][0]["graph_id"].is_null());
}
