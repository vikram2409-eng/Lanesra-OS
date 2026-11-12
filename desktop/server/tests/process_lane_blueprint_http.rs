//! AI Agent Platform v2, Phase 7: the lane blueprint reads over HTTP
//! (WP-0712, REQ-LANE-020 AC-4) - the Team Workspace server exposes the
//! same blueprint operations as the desktop Tauri commands, over the same
//! service code (`process_lane_blueprint`), through the sync `/api/invoke`
//! dispatcher. The desktop-mode proof is
//! `core/tests/process_lane_blueprint.rs`; this file is the Team Workspace
//! mode of the same reads.

use std::net::SocketAddr;

use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};

async fn spawn_file_backed_server() -> (SocketAddr, SharedState, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("lanesra.sqlite3");
    let conn = lanesra_core::db::open_workspace_db(&db_path).unwrap();
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

/// Covers: REQ-LANE-020 (WP-0712 AC-4)
#[tokio::test]
async fn lane_blueprint_reads_work_over_http_the_same_as_on_desktop() {
    let (addr, _state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let _setup = first_run(&client, addr).await;

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

    let blueprint = invoke(
        &client,
        addr,
        "get_lane_blueprint",
        json!({"laneKey": "lane-wh-receiving"}),
    )
    .await;
    assert!(blueprint["ok"].as_bool().unwrap_or(false));
    let data = &blueprint["data"];
    assert_eq!(data["lane_key"], "lane-wh-receiving");
    assert_eq!(data["name"], "Receive Shipment");
    assert_eq!(data["object_key"], "purchase_order");
    assert_eq!(data["version"], 1);
    assert_eq!(data["status"], "draft");
    assert_eq!(data["definition"]["stages"].as_array().unwrap().len(), 5);
    assert_eq!(
        data["definition"]["run_data"]["fields"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    // The blueprint definition is the stored definition. Compared
    // structurally, not byte-wise: fields the file omits (a `required:
    // false` / `per_child: false` the schema defaults) serialize
    // explicitly from the struct, so a raw JSON comparison would differ
    // on absent-vs-false. The full re-parse equality is proven in the
    // core test; over HTTP the shape, order and branch targets are the
    // check.
    let stored: Value = serde_json::from_str(&example).unwrap();
    let stored_lane = &stored["lane"];
    assert_eq!(data["definition"]["id"], stored_lane["id"]);
    assert_eq!(data["definition"]["name"], stored_lane["name"]);
    assert_eq!(data["definition"]["object_key"], stored_lane["object_key"]);
    assert_eq!(data["definition"]["version"], stored_lane["version"]);
    assert_eq!(data["definition"]["status"], stored_lane["status"]);
    let stored_fields = stored_lane["run_data"]["fields"].as_array().unwrap();
    let shown_fields = data["definition"]["run_data"]["fields"].as_array().unwrap();
    assert_eq!(shown_fields.len(), stored_fields.len());
    for (shown, stored_field) in shown_fields.iter().zip(stored_fields.iter()) {
        assert_eq!(shown["key"], stored_field["key"]);
        assert_eq!(shown["type"], stored_field["type"]);
    }
    let shown_steps: Vec<&Value> = data["definition"]["stages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["steps"].as_array().unwrap().iter())
        .collect();
    assert_eq!(shown_steps.len(), 12);
    let check = shown_steps
        .iter()
        .find(|s| s["id"] == "discrepancy-check")
        .expect("discrepancy check step exists");
    assert_eq!(check["branches"][0]["on"], "pass");
    assert_eq!(check["branches"][1]["on"], "fail");

    let list = invoke(&client, addr, "list_lane_blueprints", json!({})).await;
    assert!(list["ok"].as_bool().unwrap_or(false));
    let rows = list["data"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["lane_key"], "lane-wh-receiving");
    assert_eq!(rows[0]["latest_version"], 1);
    assert_eq!(rows[0]["latest_status"], "draft");

    let missing = invoke(
        &client,
        addr,
        "get_lane_blueprint",
        json!({"laneKey": "lane-no-such-lane"}),
    )
    .await;
    assert!(!missing["ok"].as_bool().unwrap_or(true));
}
