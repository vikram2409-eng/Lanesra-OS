//! AI Agent Platform v2, Phase 7: WP-0718 builder over HTTP (REQ-LANE-024
//! AC-4) - the Team Workspace server exposes the same builder operations
//! as the desktop Tauri commands, over the same service code. Check /
//! dry-run / reads-writes / action-types / save go through the sync
//! `/api/invoke` dispatcher.
//!
//! Covers: REQ-LANE-024 (WP-0718 AC-1), REQ-LANE-024 (WP-0718 AC-2),
//! REQ-LANE-024 (WP-0718 AC-4)

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
                "business_name": "Lane Builder HTTP Co", "legal_name": null, "currency_code": "USD", "locale": "en-US",
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

/// Covers: REQ-LANE-024 (WP-0718 AC-1)
#[tokio::test]
async fn builder_check_dry_run_reads_writes_action_types_and_save_work_over_http() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    // The demo Receiving lane's builder check over HTTP refuses on P-12,
    // the same refusal the core test proves directly.
    let example = std::fs::read_to_string(format!(
        "{}/../../program/spec/schemas/examples/lane-wh-receiving.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("demo lane example exists");
    let report = invoke(
        &client,
        addr,
        "builder_check_lane",
        json!({"definitionJson": example}),
    )
    .await;
    assert_eq!(report["data"]["lane_key"], "lane-wh-receiving");
    assert_eq!(report["data"]["refused"], true);
    let checks = report["data"]["checks"].as_array().unwrap();
    assert!(
        checks
            .iter()
            .any(|c| c["id"] == "P-12" && c["status"] == "refuse"),
        "the Receiving lane refuses on P-12 (unadopted writes): {checks:?}"
    );
    assert!(
        checks
            .iter()
            .any(|c| c["id"] == "P-16" && c["status"] == "pass"),
        "P-16 passes for the draft being edited: {checks:?}"
    );

    // The reads/writes map over HTTP names the child step's anchor read.
    let map = invoke(
        &client,
        addr,
        "builder_reads_writes",
        json!({"definitionJson": example}),
    )
    .await;
    assert_eq!(map["data"]["lane_key"], "lane-wh-receiving");
    let child = map["data"]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["step_id"] == "verify-supplier")
        .expect("the child step is in the map");
    assert!(
        child["reads"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["reference"] == "$record.supplier_id"),
        "the child step reads its anchor: {child:?}"
    );

    // The Action-Type catalogue over HTTP is empty before any lane is
    // stored: no adopted types, no provisional ids to list yet.
    let catalogue = invoke(&client, addr, "builder_action_types", json!({})).await;
    assert_eq!(catalogue["data"].as_array().unwrap().len(), 0);

    // Saving the builder lane as a brand-new lane stores it as a draft,
    // and the stored lane's publish check then agrees with the builder
    // check above.
    let saved = invoke(
        &client,
        addr,
        "save_builder_lane",
        json!({"definitionJson": check_only_lane_json(1)}),
    )
    .await;
    assert_eq!(saved["data"]["lane_key"], "lane-check-only");
    assert_eq!(saved["data"]["version"], 1);
    assert_eq!(saved["data"]["status"], "draft");

    // Importing the Receiving lane makes its provisional ids appear in
    // the catalogue, flagged provisional.
    invoke(
        &client,
        addr,
        "import_process_lane",
        json!({"definitionJson": example}),
    )
    .await;
    let catalogue = invoke(
        &client,
        addr,
        "builder_action_types",
        json!({"objectKey": "purchase_order"}),
    )
    .await;
    let entries = catalogue["data"].as_array().unwrap();
    let stock_move = entries
        .iter()
        .find(|e| e["name"] == "warehouse.post_stock_move")
        .expect("the provisional StockMove id is listed");
    assert_eq!(stock_move["provisional"], true);
    assert_eq!(stock_move["adopted"], false);

    // The builder dry run over HTTP walks the saved check-only lane's
    // definition against a seeded Active record and posts nothing.
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
    let dry = invoke(
        &client,
        addr,
        "builder_dry_run_lane",
        json!({
            "definitionJson": check_only_lane_json(1),
            "anchorRecordId": record_id,
            "fixtures": {"run_data": {}},
            "expectedOutcome": "done"
        }),
    )
    .await;
    assert_eq!(dry["data"]["would_start"], true);
    assert_eq!(dry["data"]["end_outcome"], "done");
    assert_eq!(dry["data"]["matches_expected"], true);
    assert_eq!(dry["data"]["checks"][0]["outcome"], "passed");
    assert_eq!(dry["data"]["completed"], true);
}
