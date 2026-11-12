//! AI Agent Platform v2, Phase 7: Process Lane commands over HTTP
//! (WP-0703 AC-5) - the Team Workspace server exposes the same lane
//! operations as the desktop Tauri commands, over the same service code.
//! Import/publish/list/get go through the sync `/api/invoke` dispatcher;
//! starting a run goes through the async admin route, which opens its own
//! connection (hence the file-backed DB, as in `agent_v1.rs`).
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use std::net::SocketAddr;

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::services::{custom_object_service, custom_record_service};
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
                "business_name": "Lane HTTP Co", "legal_name": null, "currency_code": "USD", "locale": "en-US",
                "timezone": "UTC", "default_tax_rate_bp": 0, "admin_username": "admin",
                "admin_display_name": "Admin", "admin_password": "supersecretpw", "load_sample_data": false
            }
        }),
    )
    .await
}

fn check_only_lane_json(version: i64) -> String {
    json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-check-only",
            "name": "Check only",
            "object_key": "purchase_order",
            "version": version,
            "status": "draft",
            "start_condition": {"field": "$record.status", "operator": "equals", "value": "Active"},
            "stages": [{
                "id": "verify",
                "name": "Verify",
                "steps": [{
                    "id": "status-check",
                    "name": "Status check",
                    "kind": "check",
                    "check": {
                        "scope": "anchor",
                        "message": "must be active",
                        "condition": {"field": "$record.status", "operator": "equals", "value": "Active"}
                    },
                    "branches": [
                        {"on": "pass", "end": {"outcome": "done"}},
                        {"on": "fail", "end": {"outcome": "held"}}
                    ]
                }]
            }]
        }
    })
    .to_string()
}

/// Covers: REQ-LANE-001 (WP-0703 AC-5), REQ-LANE-002 (WP-0703 AC-5)
#[tokio::test]
async fn lane_commands_work_over_http_the_same_as_on_desktop() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    // The demo lane imports unchanged over HTTP, exactly as in the core test.
    let example = std::fs::read_to_string(format!(
        "{}/../../program/spec/schemas/examples/lane-wh-receiving.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("demo lane example exists");
    let imported = invoke(
        &client,
        addr,
        "import_process_lane",
        json!({"definitionJson": example}),
    )
    .await;
    assert_eq!(imported["data"]["lane_key"], "lane-wh-receiving");
    assert_eq!(imported["data"]["versions"][0]["definition_json"], example);

    let published = invoke(
        &client,
        addr,
        "publish_process_lane",
        json!({"laneKey": "lane-wh-receiving"}),
    )
    .await;
    assert_eq!(published["data"]["status"], "published");
    let graph_id = published["data"]["graph_id"].as_str().unwrap().to_string();
    assert!(!graph_id.is_empty());

    let listed = invoke(&client, addr, "list_process_lanes", json!({})).await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);
    let got = invoke(
        &client,
        addr,
        "get_process_lane",
        json!({"laneKey": "lane-wh-receiving"}),
    )
    .await;
    assert_eq!(got["data"]["lane_key"], "lane-wh-receiving");

    // A run starts over the async admin route and finishes on its own
    // version, the same result the core test proves directly.
    let record_id = {
        let conn = state.conn.lock().unwrap();
        let object = custom_object_service::create_with_key(
            &conn,
            &workspace_id,
            "purchase_order",
            &CustomObjectDefinitionInput {
                singular_label: "Purchase Order".into(),
                plural_label: "Purchase Orders".into(),
                icon: "▣".into(),
                prefix: "PO".into(),
                digits: 6,
            },
            Some(&admin_id),
        )
        .unwrap();
        custom_record_service::create(
            &conn,
            &workspace_id,
            &CustomRecordInput {
                object_key: object.key.clone(),
                primary_name: "PO-2026-001".into(),
                status: "Active".into(),
                owner_user_id: None,
                notes: None,
            },
            Some(&admin_id),
        )
        .unwrap()
        .id
    };
    invoke(
        &client,
        addr,
        "import_process_lane",
        json!({"definitionJson": check_only_lane_json(1)}),
    )
    .await;
    let v1 = invoke(
        &client,
        addr,
        "publish_process_lane",
        json!({"laneKey": "lane-check-only"}),
    )
    .await;
    let graph_v1 = v1["data"]["graph_id"].as_str().unwrap().to_string();
    let run = client
        .post(format!(
            "http://{addr}/api/admin/process-lanes/lane-check-only/runs"
        ))
        .json(&json!({"anchor_record_id": record_id}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(run["ok"], true);
    assert_eq!(run["data"]["graph_id"], graph_v1);
    assert_eq!(run["data"]["status"], "completed");
    assert_eq!(run["data"]["anchor_object_key"], "purchase_order");
    assert_eq!(run["data"]["anchor_record_id"], record_id);
}
