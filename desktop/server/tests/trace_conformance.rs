//! WP-0113: the conformance cells that start over HTTP, against a real
//! spawned Team Workspace server (the rest of the matrix is in
//! `core/tests/trace_conformance.rs`):
//!
//! - the inbound webhook trigger route, which runs the agent itself;
//! - an MCP `tools/call` that creates a record, starting a run through a
//!   "Run AI agent" workflow;
//! - the command-line tool (`lanesra records create`, the real CLI), which
//!   starts a run the same way.
//!
//! Each run must have a full trace and a complete tool audit
//! (`core/tests/support/trace_conformance.rs`).
//!
//! Covers: REQ-OBS-003, REQ-NFR-003 (WP-0113 AC-1).

#[path = "../../core/tests/support/trace_conformance.rs"]
mod conformance;
#[path = "../../core/tests/support/event_schema.rs"]
mod event_schema;

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
use lanesra_core::db::open_workspace_db;
use lanesra_core::models::agent_identity::AgentIdentityInput;
use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput};
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::AiAgentTriggerInput;
use lanesra_core::models::integration::ApiClientInput;
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ai_service::{RequestedToolCall, TokenUsage};
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    agent_identity_service, ai_agent_service, ai_orchestration_service, ai_provider_service,
    api_client_service, workflow_service, workspace_service,
};
use lanesra_server::{build_router, SecurityConfig, ServerState};
use rusqlite::Connection;
use serde_json::{json, Value};

use conformance::{assert_full_trace, assert_tool_audit, audit_mark};

struct Server {
    addr: SocketAddr,
    db_path: PathBuf,
    _dir: tempfile::TempDir,
    workspace_id: String,
    admin_id: String,
    key: String,
    master_key: [u8; 32],
}

/// A server with one workspace, one scripted agent with `update_memory`,
/// and an API key with the scopes the three entry points need.
async fn server() -> (Server, String) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("lanesra.sqlite3");
    let state = ServerState::new(
        open_workspace_db(&db_path).unwrap(),
        db_path.clone(),
        SecurityConfig::default(),
    );
    let frontend_dir = dir.path().join("frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state.clone(), frontend_dir);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    });

    // The server encrypts provider keys with its own master key.
    let master_key = lanesra_core::services::secret_service::resolve_master_key(
        &db_path.with_file_name("secret.key"),
    )
    .unwrap();
    let conn = state.conn.lock().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Gate Co".into(),
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
    let provider = ai_provider_service::create(
        &conn,
        &workspace.id,
        &master_key,
        &AiProviderInput {
            name: "scripted".into(),
            provider: "anthropic".into(),
            base_url: None,
            model: "scripted-model".into(),
            api_key: Some("k".into()),
        },
        Some(&admin.id),
    )
    .unwrap();
    conn.execute(
        "UPDATE ai_providers SET provider = 'scripted' WHERE id = ?1",
        [&provider.id],
    )
    .unwrap();
    let agent = ai_agent_service::create(
        &conn,
        &workspace.id,
        &AiAgentInput {
            name: "Gatekeeper".into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "You keep notes.".into(),
            action_names: vec!["update_memory".into()],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(&admin.id),
    )
    .unwrap();
    ai_agent_service::set_model_routing(
        &conn,
        &agent.id,
        &workspace.id,
        Some(AiAgentModelRouting {
            primary_provider_id: Some(provider.id),
            ..Default::default()
        }),
        Some(&admin.id),
    )
    .unwrap();
    let key = api_client_service::create(
        &conn,
        &workspace.id,
        &ApiClientInput {
            name: "Gate client".into(),
            scopes: vec![
                "agents.trigger".into(),
                "objects.write".into(),
                "objects.read".into(),
                "metadata.read".into(),
            ],
            allowed_cidr: None,
            owner_user_id: Some(admin.id.clone()),
        },
        Some(&admin.id),
    )
    .unwrap()
    .api_key;
    drop(conn);
    (
        Server {
            addr,
            db_path,
            _dir: dir,
            workspace_id: workspace.id,
            admin_id: admin.id,
            key,
            master_key,
        },
        agent.id,
    )
}

fn usage() -> TokenUsage {
    TokenUsage {
        input_tokens: 40,
        output_tokens: 8,
    }
}

fn one_tool_call() -> Vec<ScriptedResponse> {
    vec![
        ScriptedResponse::ToolCalls {
            calls: vec![RequestedToolCall {
                id: "t1".into(),
                name: "update_memory".into(),
                arguments: json!({"content": "noted"}),
                malformed: None,
            }],
            usage: usage(),
        },
        ScriptedResponse::Text {
            text: "done".into(),
            usage: usage(),
        },
    ]
}

fn latest_run(conn: &Connection, agent_id: &str) -> (String, usize) {
    let run = ai_orchestration_service::list_runs(conn, "agent", agent_id, 1)
        .unwrap()
        .remove(0);
    assert_eq!(run.status, "succeeded", "{run:?}");
    (run.id, run.steps.len())
}

fn check(conn: &Connection, s: &Server, agent_id: &str, mark: i64, cell: &str) {
    let (run_id, steps) = latest_run(conn, agent_id);
    let trace = assert_full_trace(conn, &s.workspace_id, "pipeline", &run_id, steps, cell);
    assert_eq!(trace.tools_run, ["update_memory"], "{cell}");
    assert_tool_audit(conn, mark, &trace.tools_run, cell);
}

/// A "Run AI agent" workflow on new Companies.
fn run_agent_on_new_companies(conn: &Connection, s: &Server, agent_id: &str) {
    workflow_service::create_rule(
        conn,
        &s.workspace_id,
        &WorkflowDefinitionInput {
            app_id: None,
            entity_type: "Company".into(),
            name: "Company created -> run agent".into(),
            description: None,
            trigger_type: "record_created".into(),
            trigger_status: None,
            trigger_field_key: None,
            trigger_field_source: "custom".into(),
            trigger_offset_days: 0,
            match_type: "all".into(),
            priority: 0,
            conditions: vec![],
            actions: vec![WorkflowActionInput {
                action_type: "run_ai_agent".into(),
                params_json: json!({"target_type": "agent", "target_id": agent_id, "input_template": "Welcome them"})
                    .to_string(),
            }],
        },
        Some(&s.admin_id),
    )
    .unwrap();
}

/// Drains the queued run on this thread, where the script is.
async fn drain(conn: &Connection, s: &Server) {
    scripted_provider::reset();
    scripted_provider::script_responses(one_tool_call());
    assert_eq!(
        ai_orchestration_service::drain_pending_runs(conn, &s.workspace_id, &s.master_key, 10)
            .await
            .unwrap(),
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_webhook_route_produces_a_full_trace_and_tool_audit() {
    let (s, agent_id) = server().await;
    let conn = open_workspace_db(&s.db_path).unwrap();
    let trigger = ai_orchestration_service::create_trigger(
        &conn,
        &s.workspace_id,
        &AiAgentTriggerInput {
            target_type: "agent".into(),
            target_id: agent_id.clone(),
            trigger_type: "webhook".into(),
            interval_minutes: None,
        },
        Some(&s.admin_id),
    )
    .unwrap();
    let identity = agent_identity_service::create(
        &conn,
        &s.workspace_id,
        &AgentIdentityInput {
            name: "Inbound".into(),
            description: None,
            owner_user_id: s.admin_id.clone(),
            is_administrator: false,
            access_role_ids: vec![],
        },
        Some(&s.admin_id),
    )
    .unwrap();
    ai_orchestration_service::set_trigger_identity(
        &conn,
        &s.workspace_id,
        &trigger.id,
        Some(&identity.id),
        Some(&s.admin_id),
    )
    .unwrap();
    let mark = audit_mark(&conn);
    // The route runs the agent on a server thread.
    scripted_provider::script_shared_responses(one_tool_call());
    let response: Value = reqwest::Client::new()
        .post(format!(
            "http://{}/api/v1/agents/{agent_id}/trigger",
            s.addr
        ))
        .bearer_auth(&s.key)
        .json(&json!({"input": "go"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response["ok"], true, "{response}");
    check(&conn, &s, &agent_id, mark, "webhook route");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_mcp_started_run_produces_a_full_trace_and_tool_audit() {
    let (s, agent_id) = server().await;
    let conn = open_workspace_db(&s.db_path).unwrap();
    run_agent_on_new_companies(&conn, &s, &agent_id);
    let created: Value = reqwest::Client::new()
        .post(format!("http://{}/mcp", s.addr))
        .bearer_auth(&s.key)
        .json(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": {"name": "create_record", "arguments": {"object_key": "Company", "data": {"name": "Acme MCP", "status": "Prospect"}}}}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(created["error"].is_null(), "{created}");
    assert_ne!(created["result"]["isError"], true, "{created}");
    let mark = audit_mark(&conn);
    drain(&conn, &s).await;
    check(&conn, &s, &agent_id, mark, "MCP-started run");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cli_started_run_produces_a_full_trace_and_tool_audit() {
    let (s, agent_id) = server().await;
    let conn = open_workspace_db(&s.db_path).unwrap();
    run_agent_on_new_companies(&conn, &s, &agent_id);
    let base_url = format!("http://{}", s.addr);
    let key = s.key.clone();
    let exit = tokio::task::spawn_blocking(move || {
        lanesra_cli::run(lanesra_cli::cli::Cli::parse_from([
            "lanesra",
            "--base-url",
            &base_url,
            "--api-key",
            &key,
            "records",
            "create",
            "Company",
            "--data",
            r#"{"name": "Acme CLI", "status": "Prospect"}"#,
        ]))
    })
    .await
    .unwrap();
    assert_eq!(exit, 0, "the CLI command failed");
    let mark = audit_mark(&conn);
    drain(&conn, &s).await;
    check(&conn, &s, &agent_id, mark, "CLI-started run");
}
