//! AI Agent Platform v2, Phase 7: WP-0714 publish checks, dry run and
//! lane cases over HTTP (REQ-LANE-021 AC-4) - the Team Workspace server
//! exposes the same operations as the desktop Tauri commands, over the
//! same service code. Check/dry-run/list/store go through the sync
//! `/api/invoke` dispatcher; running a case goes through the async admin
//! route, which opens its own connection (hence the file-backed DB, as in
//! `process_lane_http.rs`).

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
                "business_name": "Lane Publish HTTP Co", "legal_name": null, "currency_code": "USD", "locale": "en-US",
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

/// Covers: REQ-LANE-021 (WP-0714 AC-4)
#[tokio::test]
async fn publish_checks_dry_run_and_cases_work_over_http() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    // The demo Receiving lane imports over HTTP, and its publish report
    // refuses it - the same P-12 refusal the core test proves directly.
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

    let report = invoke(
        &client,
        addr,
        "check_process_lane_publish",
        json!({"laneKey": "lane-wh-receiving"}),
    )
    .await;
    assert_eq!(report["data"]["refused"], true);
    let checks = report["data"]["checks"].as_array().unwrap();
    assert!(
        checks
            .iter()
            .any(|c| c["id"] == "P-12" && c["status"] == "refuse"),
        "the Receiving lane refuses on P-12 (unadopted writes): {checks:?}"
    );

    // Publishing it over HTTP refuses the same way.
    let refused = invoke(
        &client,
        addr,
        "publish_process_lane",
        json!({"laneKey": "lane-wh-receiving"}),
    )
    .await;
    assert_eq!(refused["ok"], false);
    assert!(
        refused["error"].to_string().contains("P-12"),
        "the publish refusal names P-12: {}",
        refused["error"]
    );

    // A lane with no writes publishes over HTTP, and its dry run walks
    // the check against a seeded Active record.
    invoke(
        &client,
        addr,
        "import_process_lane",
        json!({"definitionJson": check_only_lane_json(1)}),
    )
    .await;
    let clean_report = invoke(
        &client,
        addr,
        "check_process_lane_publish",
        json!({"laneKey": "lane-check-only"}),
    )
    .await;
    assert_eq!(clean_report["data"]["refused"], false);
    let published = invoke(
        &client,
        addr,
        "publish_process_lane",
        json!({"laneKey": "lane-check-only"}),
    )
    .await;
    assert_eq!(published["data"]["status"], "published");

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
        "dry_run_process_lane",
        json!({
            "laneKey": "lane-check-only",
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

    // A case stored over HTTP lists back, and runs for real over the
    // async admin route - then rolls back, so the run leaves no trace.
    let case = json!({
        "case": "check-only-happy",
        "lane": "lane-check-only",
        "version": 1,
        "anchor": {"object": "purchase_order", "id": record_id},
        "fixtures": {"run_data": {}},
        "assert": [{"run.ends_with": "done"}, {"check.passed": "status-check"}]
    })
    .to_string();
    let stored = invoke(
        &client,
        addr,
        "store_lane_case",
        json!({"laneKey": "lane-check-only", "version": 1, "caseJson": case}),
    )
    .await;
    assert_eq!(stored["data"]["name"], "check-only-happy");
    let listed = invoke(
        &client,
        addr,
        "list_lane_cases",
        json!({"laneKey": "lane-check-only", "version": 1}),
    )
    .await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);

    let run = client
        .post(format!("http://{addr}/api/admin/process-lane-cases/run"))
        .json(&json!({"case_json": case}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(run["ok"], true);
    assert_eq!(run["data"]["passed"], true);
    assert_eq!(run["data"]["end_outcome"], "done");
}
