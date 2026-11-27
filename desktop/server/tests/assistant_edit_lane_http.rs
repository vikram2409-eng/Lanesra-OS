//! AI Agent Platform v2, Phase 7: revise a lane by description - the Admin
//! Assistant edits a lane (WP-0723 AC-4, AC-5) - the Team Workspace server
//! exposes the same edit operation as the desktop Tauri command, over the
//! same service code, through the async admin route
//! `POST /api/admin/process-lanes/:lane_key/edits` (the sync `/api/invoke`
//! dispatcher cannot await a model call, so the edit lives with the other
//! async admin actions, like starting a run and the WP-0715 draft).
//!
//! The model reply is scripted (WP-0005) on this test's own thread - the
//! scripted provider keeps its queue in thread-local storage, so a reply
//! scripted here cannot reach a model call the server runs on one of its
//! own worker threads. The happy path over HTTP is therefore proven in
//! two halves that meet in the same store: the edit is made by calling
//! the same service the route calls, on this thread, and then listed over
//! HTTP - the revision the service makes is the revision the workspace
//! API serves. The route itself is proven over HTTP for what needs no
//! model: the session gate, the input validation, and failing closed
//! with no provider configured.
//!
//! Covers: REQ-LANE-022 (WP-0723 AC-4, AC-5)

use std::net::SocketAddr;

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::services::ai_service::TokenUsage;
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    ai_service, assistant_edit_lane_service, process_lane_service, secret_service,
};
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
                "business_name": "Edit Lane HTTP Co", "legal_name": null, "currency_code": "USD", "locale": "en-US",
                "timezone": "UTC", "default_tax_rate_bp": 0, "admin_username": "admin",
                "admin_display_name": "Admin", "admin_password": "supersecretpw", "load_sample_data": false
            }
        }),
    )
    .await
}

async fn edit(
    client: &reqwest::Client,
    addr: SocketAddr,
    lane_key: &str,
    description: &str,
) -> Value {
    client
        .post(format!(
            "http://{addr}/api/admin/process-lanes/{lane_key}/edits"
        ))
        .json(&json!({"description": description}))
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

/// The seed lane, as the core tests import it: one form step, run-data
/// fields only, so it publishes cleanly.
fn seed_lane_json() -> String {
    json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-edit-receive-purchase-order",
            "name": "Receive a purchase order",
            "description": "Receive a purchase order at the Toronto warehouse.",
            "object_key": "purchase_order",
            "version": 1,
            "status": "draft",
            "run_data": {"fields": [
                {"key": "carrier", "type": "string", "required": true, "description": "Carrier who delivered the shipment."}
            ]},
            "stages": [
                {"id": "verify", "name": "Verify", "steps": [
                    {"id": "confirm-arrival", "name": "Confirm shipment arrived", "kind": "form", "role": "role_wh_receiver", "required": true,
                     "description": "Confirm the truck and delivery: carrier.",
                     "fields": [
                        {"key": "carrier", "label": "Carrier", "type": "string", "required": true, "binds_to": "run_data.carrier"}
                     ]}
                ]}
            ]
        }
    })
    .to_string()
}

/// The seed lane with one added form step, as the model would revise it.
/// Mirrors the core test's reply.
fn revised_lane_json() -> String {
    let mut reply: serde_json::Value = serde_json::from_str(&seed_lane_json()).unwrap();
    reply["lane"]["run_data"]["fields"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key": "delivery_note_number", "type": "string", "required": true, "description": "Delivery-note number from the carrier's paperwork."}));
    reply["lane"]["stages"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "record-delivery-note", "name": "Record delivery-note number", "kind": "form", "role": "role_wh_receiver", "required": true,
            "description": "Record the delivery-note number from the carrier's paperwork.",
            "fields": [
                {"key": "delivery_note_number", "label": "Delivery-note number", "type": "string", "required": true, "binds_to": "run_data.delivery_note_number"}
            ]}));
    reply.to_string()
}

const LANE_KEY: &str = "lane-edit-receive-purchase-order";
const CHANGE: &str = "Add a step where the receiver records the delivery-note number.";

/// Covers: REQ-LANE-022 (WP-0723 AC-4, AC-5)
#[tokio::test]
async fn edit_route_is_gated_validated_and_fails_closed_without_a_provider() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    // No session cookie: the route refuses before it ever reaches the
    // service.
    let bare = reqwest::Client::new();
    let denied = bare
        .post(format!(
            "http://{addr}/api/admin/process-lanes/{LANE_KEY}/edits"
        ))
        .json(&json!({"description": CHANGE}))
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(denied.json::<Value>().await.unwrap()["ok"], false);

    // An empty description is refused before any model call - the same
    // validation the core test proves, over HTTP.
    let refused = edit(&client, addr, LANE_KEY, "   ").await;
    assert_eq!(refused["ok"], false);
    assert!(refused["error"]
        .as_str()
        .unwrap()
        .contains("describe the change"));

    // An unknown lane key is refused with the named reason, over HTTP.
    let unknown = edit(&client, addr, "lane-does-not-exist", CHANGE).await;
    assert_eq!(unknown["ok"], false);
    assert!(unknown["error"]
        .as_str()
        .unwrap()
        .contains("no lane with key 'lane-does-not-exist'"));

    // A real description with no provider configured fails closed: the
    // route reaches the model call and the service refuses without a key
    // rather than editing from nowhere. The lane must exist first, so
    // seed it through the service on this thread.
    {
        let conn = open_workspace_db(&state.db_path).unwrap();
        process_lane_service::import_lane_definition(
            &conn,
            &workspace_id,
            &seed_lane_json(),
            Some(&admin_id),
        )
        .unwrap();
    }
    let no_provider = edit(&client, addr, LANE_KEY, CHANGE).await;
    assert_eq!(no_provider["ok"], false);
    assert!(no_provider["error"]
        .as_str()
        .unwrap()
        .contains("Configure an AI provider key first"));
}

/// Covers: REQ-LANE-022 (WP-0723 AC-4, AC-5)
#[tokio::test]
async fn edit_made_by_the_service_is_the_revision_the_workspace_api_serves() {
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
    scripted_provider::script_responses(vec![text_reply(&revised_lane_json())]);

    // The seed lane is imported and published by the service on this
    // thread, then edited by the same service the route and the desktop
    // command both call - on this thread, where the scripted reply lives.
    let version = {
        // Open a fresh connection to the same file DB rather than holding the
        // shared `state.conn` mutex guard across the `.await`
        // (clippy::await_holding_lock). The scripted provider is thread-local
        // and `block_on` runs on this thread, so the edit still sees the
        // scripted reply.
        let conn = open_workspace_db(&state.db_path).unwrap();
        process_lane_service::import_lane_definition(
            &conn,
            &workspace_id,
            &seed_lane_json(),
            Some(&admin_id),
        )
        .unwrap();
        process_lane_service::publish_lane(&conn, &workspace_id, LANE_KEY, Some(&admin_id))
            .unwrap();
        assistant_edit_lane_service::edit_lane(
            &conn,
            &workspace_id,
            &master_key,
            Some(&admin_id),
            LANE_KEY,
            CHANGE,
        )
        .await
        .unwrap()
    };
    assert_eq!(version.version, 2);
    assert_eq!(version.status, "draft");

    // AC-4: the same revision is what the workspace API serves over HTTP -
    // v1 published, v2 draft, and the draft carries the added step.
    let listed = invoke(&client, addr, "list_process_lanes", json!({})).await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);
    assert_eq!(listed["data"][0]["lane_key"], LANE_KEY);
    assert_eq!(listed["data"][0]["versions"].as_array().unwrap().len(), 2);
    assert_eq!(listed["data"][0]["versions"][0]["version"], 1);
    assert_eq!(listed["data"][0]["versions"][0]["status"], "published");
    assert_eq!(listed["data"][0]["versions"][1]["version"], 2);
    assert_eq!(listed["data"][0]["versions"][1]["status"], "draft");
    let definition: Value = serde_json::from_str(
        listed["data"][0]["versions"][1]["definition_json"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        definition["lane"]["stages"][0]["steps"][1]["id"],
        "record-delivery-note"
    );
}
