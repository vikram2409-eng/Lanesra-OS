//! WP-0150 over HTTP, the way the Team Workspace browser UI calls it: an
//! Agent Team run started from `/api/admin/...` pauses at its Approval
//! node, the approvals inbox decides it at `/api/admin/approvals/:id/decide`
//! and the run continues to the end (RAID I-13). The synchronous
//! `/api/invoke/resolve_ai_approval` command refuses an approval a run
//! waits on, so it can no longer leave the run stuck.
//!
//! Covers: REQ-GOV-011 (WP-0150).

use std::net::SocketAddr;

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::execution_graph::{ExecutionGraphInput, GraphEdgeInput, GraphNodeInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{execution_graph_service, workspace_service};
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};

fn node(key: &str, node_type: &str) -> GraphNodeInput {
    GraphNodeInput {
        node_key: key.into(),
        node_type: node_type.into(),
        config_json: "{}".into(),
        position_x: None,
        position_y: None,
        sort_order: 0,
    }
}

fn edge(from: &str, to: &str, label: Option<&str>) -> GraphEdgeInput {
    GraphEdgeInput {
        from_node_key: from.into(),
        to_node_key: to.into(),
        branch_label: label.map(str::to_string),
        sort_order: 0,
    }
}

async fn post(client: &reqwest::Client, url: String, body: Value) -> Value {
    client
        .post(url)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_inbox_decides_a_graph_approval_over_http_and_the_run_continues() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("lanesra.sqlite3");
    let state = ServerState::new(
        open_workspace_db(&db_path).unwrap(),
        db_path,
        SecurityConfig::default(),
    );
    let frontend_dir = dir.path().join("frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state.clone(), frontend_dir);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let graph_id = {
        let conn = state.conn.lock().unwrap();
        let (workspace, admin) = workspace_service::first_run_setup(
            &conn,
            &WorkspaceSetup {
                business_name: "Inbox Co".into(),
                legal_name: None,
                currency_code: "USD".into(),
                locale: "en-US".into(),
                timezone: "UTC".into(),
                default_tax_rate_bp: 0,
                admin_username: "admin".into(),
                admin_display_name: "Admin".into(),
                admin_password: "supersecretpassword".into(),
                load_sample_data: false,
            },
        )
        .unwrap();
        let graph = execution_graph_service::create(
            &conn,
            &workspace.id,
            &ExecutionGraphInput {
                name: "Gate".into(),
                description: None,
                nodes: vec![
                    node("trigger", "trigger"),
                    node("gate", "approval"),
                    node("done", "end"),
                    node("stopped", "end"),
                ],
                edges: vec![
                    edge("trigger", "gate", None),
                    edge("gate", "done", Some("approved")),
                    edge("gate", "stopped", Some("rejected")),
                ],
            },
            Some(&admin.id),
        )
        .unwrap();
        execution_graph_service::publish(&conn, &graph.id, &workspace.id, Some(&admin.id))
            .unwrap()
            .id
    };

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let login = post(
        &client,
        format!("http://{addr}/api/invoke/login"),
        json!({"credentials": {"username": "admin", "password": "supersecretpassword"}}),
    )
    .await;
    assert_eq!(login["ok"], true, "{login}");

    // The browser's Start test run.
    let started = post(
        &client,
        format!("http://{addr}/api/admin/execution-graphs/{graph_id}/run"),
        json!({"input": "{}"}),
    )
    .await;
    assert_eq!(started["ok"], true, "{started}");
    assert_eq!(started["data"]["status"], "waiting_approval");
    let run_id = started["data"]["id"].as_str().unwrap().to_string();
    let approval_id = started["data"]["pending_approval_id"]
        .as_str()
        .unwrap()
        .to_string();

    // The old synchronous command refuses it instead of stranding the run.
    let refused = post(
        &client,
        format!("http://{addr}/api/invoke/resolve_ai_approval"),
        json!({"id": approval_id, "resolution": {"approve": true, "resolution_notes": null}}),
    )
    .await;
    assert_eq!(refused["ok"], false, "{refused}");

    // The inbox decides it and the run goes on to the end.
    let decided = post(
        &client,
        format!("http://{addr}/api/admin/approvals/{approval_id}/decide"),
        json!({"approve": true, "resolution_notes": "ok"}),
    )
    .await;
    assert_eq!(decided["ok"], true, "{decided}");
    assert_eq!(decided["data"]["status"], "approved");
    let run = post(
        &client,
        format!("http://{addr}/api/invoke/get_graph_run"),
        json!({"id": run_id}),
    )
    .await;
    assert_eq!(run["data"]["status"], "completed", "{run}");
}
