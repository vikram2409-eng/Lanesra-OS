//! WP-0109: unattended runs act as an Agent Identity, and every tool call
//! records who it acted as.
//!
//! AC-1: every tool-call audit entry names the agent and the effective
//! principal, from every entry point (manual run, scheduled drain, webhook,
//! Workflow Automation enqueue, chat, delegation, graph agent node).
//! AC-2: upgrading a database with schedule and webhook triggers gives each
//! one an Agent Identity with the narrowest rights (not an Administrator, no
//! Access Roles), owned by its creator or else the first Administrator,
//! audits the assignment, and the trigger still runs.
//! Scope (RAID I-14): a webhook or queued run acts as its trigger's Agent
//! Identity; an admin-scoped agent runs unattended only as an identity that
//! holds Administrator, never by borrowing a person's rights.
//!
//! Covers: REQ-GOV-002, REQ-GOV-003 (WP-0109). Threats: T-03, T-12.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::db::migrate::run_migrations;
use lanesra_core::models::agent_identity::AgentIdentityInput;
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentRun, AiAgentTriggerInput};
use lanesra_core::models::execution_graph::{ExecutionGraphInput, GraphEdgeInput, GraphNodeInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    agent_identity_service, ai_agent_service, ai_orchestration_service, ai_service, chat_service,
    execution_graph_service, graph_runtime_service, workspace_service,
};
use rusqlite::Connection;
use serde_json::{json, Value};

const KEY: [u8; 32] = [98u8; 32];

struct Ws {
    conn: Connection,
    id: String,
    admin: String,
}

fn workspace() -> Ws {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Identity Co".into(),
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
    Ws {
        conn,
        id: workspace.id,
        admin: admin.id,
    }
}

/// Reads one HTTP request; returns its body.
fn read_request(stream: &std::net::TcpStream) -> String {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line.trim().is_empty() {
            break;
        }
        if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    let mut buf = vec![0u8; content_length];
    let _ = reader.read_exact(&mut buf);
    String::from_utf8_lossy(&buf).to_string()
}

fn respond(mut stream: std::net::TcpStream, body: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
}

/// A model provider that answers with `bodies` in turn and records every
/// request body it receives.
fn spawn_provider(bodies: Vec<Value>) -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let queue = Arc::new(Mutex::new(VecDeque::from(bodies)));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen_by_server = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            seen_by_server.lock().unwrap().push(read_request(&stream));
            let body = {
                let mut q = queue.lock().unwrap();
                if q.len() > 1 {
                    q.pop_front().unwrap()
                } else {
                    q.front().cloned().unwrap_or(json!({}))
                }
            };
            respond(stream, &body.to_string());
        }
    });
    (port, seen)
}

fn tool_use(name: &str, input: Value) -> Value {
    json!({"content": [{"type": "tool_use", "id": "call_1", "name": name, "input": input}]})
}

fn text(t: &str) -> Value {
    json!({"content": [{"type": "text", "text": t}]})
}

fn use_provider(conn: &Connection, workspace_id: &str, admin: &str, bodies: Vec<Value>) {
    let (port, _) = spawn_provider(bodies);
    ai_service::save_settings(
        conn,
        workspace_id,
        &KEY,
        &AiSettingsInput {
            provider: "anthropic".into(),
            base_url: Some(format!("http://127.0.0.1:{port}")),
            model: "claude-haiku-4-5-20251001".into(),
            api_key: Some("provider-key".into()),
        },
        Some(admin),
    )
    .unwrap();
}

/// The model calls `update_memory` once, then answers.
fn one_tool_call(ws: &Ws) {
    use_provider(
        &ws.conn,
        &ws.id,
        &ws.admin,
        vec![
            tool_use("update_memory", json!({"content": "noted"})),
            text("done"),
        ],
    );
}

fn agent(ws: &Ws, name: &str, action_names: Vec<String>, delegates: Vec<String>) -> String {
    ai_agent_service::create(
        &ws.conn,
        &ws.id,
        &AiAgentInput {
            name: name.into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: format!("You are {name}."),
            action_names,
            delegate_agent_ids: delegates,
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(&ws.admin),
    )
    .unwrap()
    .id
}

fn identity(ws: &Ws, name: &str, is_administrator: bool) -> String {
    agent_identity_service::create(
        &ws.conn,
        &ws.id,
        &AgentIdentityInput {
            name: name.into(),
            description: None,
            owner_user_id: ws.admin.clone(),
            is_administrator,
            access_role_ids: vec![],
        },
        Some(&ws.admin),
    )
    .unwrap()
    .id
}

fn trigger(ws: &Ws, agent_id: &str, trigger_type: &str, identity_id: Option<&str>) -> String {
    let t = ai_orchestration_service::create_trigger(
        &ws.conn,
        &ws.id,
        &AiAgentTriggerInput {
            target_type: "agent".into(),
            target_id: agent_id.into(),
            trigger_type: trigger_type.into(),
            interval_minutes: (trigger_type == "schedule").then_some(60),
        },
        Some(&ws.admin),
    )
    .unwrap();
    ai_orchestration_service::set_trigger_identity(
        &ws.conn,
        &ws.id,
        &t.id,
        identity_id,
        Some(&ws.admin),
    )
    .unwrap()
    .id
}

async fn drain(conn: &Connection, workspace_id: &str, agent_id: &str) -> AiAgentRun {
    ai_orchestration_service::drain_pending_runs(conn, workspace_id, &KEY, 10)
        .await
        .unwrap();
    ai_orchestration_service::list_runs(conn, "agent", agent_id, 1)
        .unwrap()
        .remove(0)
}

/// The `details_json` of every tool-call audit entry for `agent_id`.
fn tool_call_audits(conn: &Connection, agent_id: &str) -> Vec<Value> {
    let mut stmt = conn
        .prepare(
            "SELECT details_json FROM audit_events
              WHERE event_type = 'ai_agent_tool_call' AND entity_id = ?1 ORDER BY rowid",
        )
        .unwrap();
    stmt.query_map([agent_id], |r| r.get::<_, Option<String>>(0))
        .unwrap()
        .map(|d| serde_json::from_str(&d.unwrap().expect("details_json")).unwrap())
        .collect()
}

/// The one tool-call entry for `agent_id`: names the agent and the tool,
/// and returns the effective principal it recorded.
fn principal_of_tool_call(conn: &Connection, agent_id: &str) -> Value {
    let audits = tool_call_audits(conn, agent_id);
    assert_eq!(audits.len(), 1, "{audits:?}");
    assert_eq!(audits[0]["agent_id"], agent_id);
    assert_eq!(audits[0]["tool"], "update_memory");
    audits[0]["effective_principal"].clone()
}

fn human(user_id: &str) -> Value {
    json!({"type": "human", "user_id": user_id, "agent_identity_id": null})
}

fn unattended(identity_id: &str, record_actor: Option<&str>) -> Value {
    json!({"type": "agent_identity", "agent_identity_id": identity_id, "record_actor": record_actor})
}

#[tokio::test]
async fn ac1_a_manual_run_records_the_person() {
    // Covers: REQ-GOV-002 (WP-0109 AC-1)
    let ws = workspace();
    one_tool_call(&ws);
    let a = agent(&ws, "Manual", vec![], vec![]);
    let run = ai_orchestration_service::run_manual(
        &ws.conn,
        &ws.id,
        &KEY,
        "agent",
        &a,
        "go",
        Some(&ws.admin),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "succeeded", "{run:?}");
    assert_eq!(principal_of_tool_call(&ws.conn, &a), human(&ws.admin));
}

#[tokio::test]
async fn ac1_a_scheduled_run_records_its_triggers_identity() {
    // Covers: REQ-GOV-003 (WP-0109 AC-1)
    let ws = workspace();
    one_tool_call(&ws);
    let a = agent(&ws, "Scheduled", vec![], vec![]);
    let id = identity(&ws, "Nightly", false);
    trigger(&ws, &a, "schedule", Some(&id));
    assert_eq!(
        ai_orchestration_service::enqueue_due_schedules(&ws.conn, &ws.id).unwrap(),
        1
    );
    let run = drain(&ws.conn, &ws.id, &a).await;
    assert_eq!(run.status, "succeeded", "{run:?}");
    assert_eq!(principal_of_tool_call(&ws.conn, &a), unattended(&id, None));
}

#[tokio::test]
async fn ac1_a_webhook_run_records_its_triggers_identity_and_record_actor() {
    // Covers: REQ-GOV-003 (WP-0109 AC-1)
    let ws = workspace();
    one_tool_call(&ws);
    let a = agent(&ws, "Hooked", vec![], vec![]);
    let id = identity(&ws, "Inbound", false);
    trigger(&ws, &a, "webhook", Some(&id));
    let run = ai_orchestration_service::run_webhook(
        &ws.conn,
        &ws.id,
        &KEY,
        "agent",
        &a,
        Some(&ws.admin),
        "go",
    )
    .await
    .unwrap();
    assert_eq!(run.status, "succeeded", "{run:?}");
    assert_eq!(
        principal_of_tool_call(&ws.conn, &a),
        unattended(&id, Some(&ws.admin))
    );
}

#[tokio::test]
async fn ac1_a_workflow_enqueued_run_records_the_named_identity() {
    // Covers: REQ-GOV-003 (WP-0109 AC-1)
    let ws = workspace();
    one_tool_call(&ws);
    let a = agent(&ws, "Workflow", vec![], vec![]);
    let id = identity(&ws, "Automation", false);
    ai_orchestration_service::enqueue(
        &ws.conn,
        &ws.id,
        "agent",
        &a,
        "go",
        Some("workflow"),
        None,
        None,
        Some(&id),
    )
    .unwrap();
    let run = drain(&ws.conn, &ws.id, &a).await;
    assert_eq!(run.status, "succeeded", "{run:?}");
    assert_eq!(principal_of_tool_call(&ws.conn, &a), unattended(&id, None));
}

#[tokio::test]
async fn ac1_a_chat_message_records_the_person() {
    // Covers: REQ-GOV-002 (WP-0109 AC-1)
    let ws = workspace();
    one_tool_call(&ws);
    let a = agent(&ws, "Chatty", vec![], vec![]);
    chat_service::send_agent_message(&ws.conn, &ws.id, &KEY, &ws.admin, &a, "hello")
        .await
        .unwrap();
    assert_eq!(principal_of_tool_call(&ws.conn, &a), human(&ws.admin));
}

#[tokio::test]
async fn ac1_a_delegated_agent_records_the_identity_of_the_run_that_delegated() {
    // Covers: REQ-GOV-003 (WP-0109 AC-1)
    let ws = workspace();
    use_provider(
        &ws.conn,
        &ws.id,
        &ws.admin,
        vec![
            tool_use(
                "delegate_to_agent",
                json!({"agent_name": "Helper", "input": "please"}),
            ),
            tool_use("update_memory", json!({"content": "noted"})),
            text("helper done"),
            text("lead done"),
        ],
    );
    let helper = agent(&ws, "Helper", vec![], vec![]);
    let lead = agent(&ws, "Lead", vec![], vec![helper.clone()]);
    let id = identity(&ws, "Delegating", false);
    trigger(&ws, &lead, "schedule", Some(&id));
    ai_orchestration_service::enqueue_due_schedules(&ws.conn, &ws.id).unwrap();
    let run = drain(&ws.conn, &ws.id, &lead).await;
    assert_eq!(run.status, "succeeded", "{run:?}");
    assert_eq!(
        principal_of_tool_call(&ws.conn, &helper),
        unattended(&id, None)
    );
    let lead_calls = tool_call_audits(&ws.conn, &lead);
    assert_eq!(lead_calls.len(), 1, "{lead_calls:?}");
    assert_eq!(lead_calls[0]["tool"], "delegate_to_agent");
    assert_eq!(lead_calls[0]["effective_principal"], unattended(&id, None));
}

fn node(key: &str, node_type: &str, config: Value) -> GraphNodeInput {
    GraphNodeInput {
        node_key: key.into(),
        node_type: node_type.into(),
        config_json: config.to_string(),
        position_x: None,
        position_y: None,
        sort_order: 0,
    }
}

fn edge(from: &str, to: &str) -> GraphEdgeInput {
    GraphEdgeInput {
        from_node_key: from.into(),
        to_node_key: to.into(),
        branch_label: None,
        sort_order: 0,
    }
}

#[tokio::test]
async fn ac1_a_graph_agent_node_records_the_person_who_started_the_graph() {
    // Covers: REQ-GOV-002 (WP-0109 AC-1)
    let ws = workspace();
    one_tool_call(&ws);
    let a = agent(&ws, "Node", vec![], vec![]);
    let admin = Some(ws.admin.as_str());
    let graph = execution_graph_service::create(
        &ws.conn,
        &ws.id,
        &ExecutionGraphInput {
            name: "Agent graph".into(),
            description: None,
            nodes: vec![
                node("trigger", "trigger", json!({})),
                node("agent", "agent", json!({"agent_id": a})),
                node("end", "end", json!({})),
            ],
            edges: vec![edge("trigger", "agent"), edge("agent", "end")],
        },
        admin,
    )
    .unwrap();
    let graph = execution_graph_service::publish(&ws.conn, &graph.id, &ws.id, admin).unwrap();
    let run = graph_runtime_service::start_run(
        &ws.conn, &ws.id, &KEY, &graph.id, "go", admin, None, None, None,
    )
    .await
    .unwrap();
    assert_eq!(run.status, "completed", "{run:?}");
    assert_eq!(principal_of_tool_call(&ws.conn, &a), human(&ws.admin));
}

#[tokio::test]
async fn an_admin_scoped_agent_runs_unattended_only_as_an_administrator_identity() {
    // Covers: REQ-GOV-003 (WP-0109, RAID I-14)
    let ws = workspace();
    use_provider(
        &ws.conn,
        &ws.id,
        &ws.admin,
        vec![tool_use("list_users", json!({})), text("done")],
    );
    let a = agent(&ws, "Admin agent", vec!["list_users".into()], vec![]);
    let admin_identity = identity(&ws, "Ops admin", true);
    let plain_identity = identity(&ws, "Ops plain", false);

    ai_orchestration_service::enqueue(
        &ws.conn,
        &ws.id,
        "agent",
        &a,
        "go",
        Some("workflow"),
        None,
        None,
        Some(&plain_identity),
    )
    .unwrap();
    let refused = drain(&ws.conn, &ws.id, &a).await;
    assert_eq!(refused.status, "failed", "{refused:?}");
    assert!(
        refused
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Administrator"),
        "{refused:?}"
    );
    assert!(tool_call_audits(&ws.conn, &a).is_empty());

    ai_orchestration_service::enqueue(
        &ws.conn,
        &ws.id,
        "agent",
        &a,
        "go",
        Some("workflow"),
        None,
        None,
        Some(&admin_identity),
    )
    .unwrap();
    let allowed = drain(&ws.conn, &ws.id, &a).await;
    assert_eq!(allowed.status, "succeeded", "{allowed:?}");
    let audits = tool_call_audits(&ws.conn, &a);
    assert_eq!(audits.len(), 1, "{audits:?}");
    assert_eq!(audits[0]["tool"], "list_users");
    assert_eq!(
        audits[0]["effective_principal"],
        unattended(&admin_identity, None)
    );
}

#[tokio::test]
async fn a_run_whose_identity_was_disabled_is_refused_and_recorded() {
    // Covers: REQ-GOV-003 (WP-0109, RAID I-14)
    let ws = workspace();
    one_tool_call(&ws);
    let a = agent(&ws, "Disabled", vec![], vec![]);
    let id = identity(&ws, "Retired", false);
    trigger(&ws, &a, "schedule", Some(&id));
    agent_identity_service::set_active(&ws.conn, &ws.id, &id, false, Some(&ws.admin)).unwrap();
    ai_orchestration_service::enqueue_due_schedules(&ws.conn, &ws.id).unwrap();
    let run = drain(&ws.conn, &ws.id, &a).await;
    assert_eq!(run.status, "failed", "{run:?}");
    assert!(tool_call_audits(&ws.conn, &a).is_empty());
}

#[tokio::test]
async fn setting_a_trigger_identity_is_admin_only_and_rejects_another_workspaces_identity() {
    // Covers: REQ-GOV-003 (WP-0109, RAID I-14)
    let ws = workspace();
    let other = workspace();
    let a = agent(&ws, "Guarded", vec![], vec![]);
    let t = trigger(&ws, &a, "schedule", None);
    let foreign = identity(&other, "Foreign", true);
    assert!(ai_orchestration_service::set_trigger_identity(
        &ws.conn,
        &ws.id,
        &t,
        Some(&foreign),
        Some(&ws.admin)
    )
    .is_err());
    assert!(
        ai_orchestration_service::set_trigger_identity(&ws.conn, &ws.id, &t, None, None).is_err()
    );
}

fn upgraded_trigger_fixture() -> Connection {
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/db/0081_triggers.db");
    let copy = std::env::temp_dir().join(format!(
        "lanesra-wp0109-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::copy(&src, &copy).unwrap();
    let conn = Connection::open(&copy).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    run_migrations(&conn).unwrap();
    conn
}

#[tokio::test]
async fn ac2_upgrade_gives_each_trigger_a_narrow_identity_and_it_still_runs() {
    // Covers: REQ-GOV-003 (WP-0109 AC-2)
    let conn = upgraded_trigger_fixture();
    let rows: Vec<(String, String, i64, i64, String)> = conn
        .prepare(
            "SELECT t.id, ai.owner_user_id, ai.is_administrator,
                    (SELECT COUNT(*) FROM agent_identity_access_roles r WHERE r.agent_identity_id = ai.id),
                    ai.status
               FROM ai_agent_triggers t JOIN agent_identities ai ON ai.id = t.agent_identity_id
              ORDER BY t.id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let owners: Vec<(&str, &str)> = rows
        .iter()
        .map(|(t, owner, ..)| (t.as_str(), owner.as_str()))
        .collect();
    assert_eq!(
        owners,
        vec![
            ("t_hook_admin", "u_admin"),
            ("t_sched_orphan", "u_admin"),
            ("t_sched_sales", "u_sales"),
        ],
        "every trigger has an identity owned by its creator, else the first Administrator"
    );
    for (t, _, is_admin, roles, status) in &rows {
        assert_eq!(*is_admin, 0, "{t} must not be an Administrator");
        assert_eq!(*roles, 0, "{t} must hold no Access Roles");
        assert_eq!(status, "active");
    }
    let audited: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE event_type = 'agent_identity_assigned_on_upgrade'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(audited, 3, "each assignment is audited");

    // The schedule triggers still run, now as their identities.
    use_provider(&conn, "ws_fixture", "u_admin", vec![text("still working")]);
    assert_eq!(
        ai_orchestration_service::enqueue_due_schedules(&conn, "ws_fixture").unwrap(),
        2
    );
    let drained = ai_orchestration_service::drain_pending_runs(&conn, "ws_fixture", &KEY, 10)
        .await
        .unwrap();
    assert_eq!(drained, 2);
    let runs = ai_orchestration_service::list_runs(&conn, "agent", "a_fixture", 10).unwrap();
    assert_eq!(runs.len(), 2);
    assert!(runs.iter().all(|r| r.status == "succeeded"), "{runs:?}");
}
