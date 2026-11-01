//! WP-0113: the full-trace conformance suite, the Phase 1 gate test. Every
//! way a run can start produces a full trace, and every tool call in it a
//! complete audit entry (`support/trace_conformance.rs` says what both
//! mean).
//!
//! The cells run here, through the same service calls their entry points
//! make: a sequential, a consensus and a peer-review pipeline, an
//! execution-graph agent node, the "Run AI agent" workflow action, a
//! schedule trigger and a webhook trigger. A direct chat with an agent is a
//! conversation, not a run (WP-0111 decided it keeps no trace), so its cell
//! checks the tool audit and that no stray trace is written. The cells that
//! start over HTTP (the webhook route, an MCP tool call and the command-line
//! tool) are in `server/tests/trace_conformance.rs`.
//!
//! Runs go through the scripted provider (WP-0005), each agent calling the
//! `update_memory` tool once.
//!
//! Covers: REQ-OBS-003, REQ-NFR-003 (WP-0113 AC-1).

#[path = "support/trace_conformance.rs"]
mod conformance;
#[path = "support/event_schema.rs"]
mod event_schema;

use lanesra_core::models::agent_identity::AgentIdentityInput;
use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput};
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{
    AiAgentPipelineInput, AiAgentTriggerInput, PipelineStepInput,
};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::execution_graph::{ExecutionGraphInput, GraphEdgeInput, GraphNodeInput};
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ai_service::{RequestedToolCall, TokenUsage};
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    agent_identity_service, ai_agent_service, ai_orchestration_service, ai_provider_service,
    chat_service, company_service, execution_graph_service, graph_runtime_service,
    workflow_service, workspace_service,
};
use rusqlite::Connection;
use serde_json::json;

use conformance::{assert_full_trace, assert_tool_audit, audit_mark};

const KEY: [u8; 32] = [113u8; 32];

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
    Ws {
        conn,
        id: workspace.id,
        admin: admin.id,
    }
}

/// An agent with the `update_memory` tool, answered by the scripted provider.
fn agent(ws: &Ws, name: &str) -> String {
    let provider = ai_provider_service::create(
        &ws.conn,
        &ws.id,
        &KEY,
        &AiProviderInput {
            name: format!("scripted {name}"),
            provider: "anthropic".into(),
            base_url: None,
            model: "scripted-model".into(),
            api_key: Some("k".into()),
        },
        Some(&ws.admin),
    )
    .unwrap();
    ws.conn
        .execute(
            "UPDATE ai_providers SET provider = 'scripted' WHERE id = ?1",
            [&provider.id],
        )
        .unwrap();
    let created = ai_agent_service::create(
        &ws.conn,
        &ws.id,
        &AiAgentInput {
            name: name.into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: format!("You are {name}."),
            action_names: vec!["update_memory".into()],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(&ws.admin),
    )
    .unwrap();
    ai_agent_service::set_model_routing(
        &ws.conn,
        &created.id,
        &ws.id,
        Some(AiAgentModelRouting {
            primary_provider_id: Some(provider.id),
            ..Default::default()
        }),
        Some(&ws.admin),
    )
    .unwrap();
    created.id
}

fn usage() -> TokenUsage {
    TokenUsage {
        input_tokens: 40,
        output_tokens: 8,
    }
}

fn tool_call() -> ScriptedResponse {
    ScriptedResponse::ToolCalls {
        calls: vec![RequestedToolCall {
            id: "t1".into(),
            name: "update_memory".into(),
            arguments: json!({"content": "noted"}),
            malformed: None,
        }],
        usage: usage(),
    }
}

fn text(t: &str) -> ScriptedResponse {
    ScriptedResponse::Text {
        text: t.into(),
        usage: usage(),
    }
}

fn script(responses: Vec<ScriptedResponse>) {
    scripted_provider::reset();
    scripted_provider::script_responses(responses);
}

fn pipeline(ws: &Ws, topology: &str, agents: &[&str]) -> String {
    ai_orchestration_service::create_pipeline(
        &ws.conn,
        &ws.id,
        &AiAgentPipelineInput {
            name: format!("{topology} pipeline"),
            description: None,
            topology: topology.into(),
            steps: agents
                .iter()
                .map(|a| PipelineStepInput {
                    agent_id: a.to_string(),
                    input_template: "{{trigger_input}} {{previous_output}} {{candidate_outputs}}"
                        .into(),
                    requires_approval: false,
                })
                .collect(),
        },
        Some(&ws.admin),
    )
    .unwrap()
    .id
}

fn identity(ws: &Ws, name: &str) -> String {
    agent_identity_service::create(
        &ws.conn,
        &ws.id,
        &AgentIdentityInput {
            name: name.into(),
            description: None,
            owner_user_id: ws.admin.clone(),
            is_administrator: false,
            access_role_ids: vec![],
        },
        Some(&ws.admin),
    )
    .unwrap()
    .id
}

fn trigger(ws: &Ws, agent_id: &str, trigger_type: &str) {
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
    let id = identity(ws, &format!("{trigger_type} identity"));
    ai_orchestration_service::set_trigger_identity(
        &ws.conn,
        &ws.id,
        &t.id,
        Some(&id),
        Some(&ws.admin),
    )
    .unwrap();
}

/// The newest run of an agent target, and its agent steps.
fn latest_agent_run(ws: &Ws, agent_id: &str) -> (String, usize) {
    let run = ai_orchestration_service::list_runs(&ws.conn, "agent", agent_id, 1)
        .unwrap()
        .remove(0);
    assert_eq!(run.status, "succeeded", "{run:?}");
    (run.id, run.steps.len())
}

/// Checks a finished pipeline-kind run: full trace, then the tool audit.
fn check_pipeline_run(ws: &Ws, run_id: &str, agent_steps: usize, mark: i64, cell: &str) {
    let trace = assert_full_trace(&ws.conn, &ws.id, "pipeline", run_id, agent_steps, cell);
    assert!(
        !trace.tools_run.is_empty(),
        "{cell}: the agent's tool call was not traced"
    );
    // The tool round and the answer are both model calls, in steps.
    assert!(trace.model_calls > agent_steps, "{cell}: {trace:?}");
    assert!(trace.steps >= 1, "{cell}: {trace:?}");
    assert_tool_audit(&ws.conn, mark, &trace.tools_run, cell);
}

#[tokio::test]
async fn every_pipeline_topology_produces_a_full_trace_and_tool_audit() {
    let ws = workspace();
    let a = agent(&ws, "First");
    let b = agent(&ws, "Second");
    let c = agent(&ws, "Third");
    let cases: Vec<(&str, Vec<&str>, Vec<ScriptedResponse>)> = vec![
        (
            "sequential",
            vec![&a, &b],
            vec![tool_call(), text("notes"), text("the brief")],
        ),
        (
            "consensus",
            vec![&a, &b, &c],
            vec![tool_call(), text("one"), text("two"), text("both")],
        ),
        (
            "peer_review",
            vec![&a, &b],
            vec![tool_call(), text("a draft"), text("APPROVED - good")],
        ),
    ];
    for (topology, agents, responses) in cases {
        let id = pipeline(&ws, topology, &agents);
        script(responses);
        let mark = audit_mark(&ws.conn);
        let run = ai_orchestration_service::run_manual(
            &ws.conn,
            &ws.id,
            &KEY,
            "pipeline",
            &id,
            "Acme",
            Some(&ws.admin),
        )
        .await
        .unwrap();
        assert_eq!(run.status, "succeeded", "{topology}: {run:?}");
        check_pipeline_run(&ws, &run.id, run.steps.len(), mark, topology);
    }
}

#[tokio::test]
async fn a_graph_agent_node_produces_a_full_trace_and_tool_audit() {
    let ws = workspace();
    let a = agent(&ws, "Node agent");
    let node = |key: &str, node_type: &str, config: serde_json::Value| GraphNodeInput {
        node_key: key.into(),
        node_type: node_type.into(),
        config_json: config.to_string(),
        position_x: None,
        position_y: None,
        sort_order: 0,
    };
    let edge = |from: &str, to: &str| GraphEdgeInput {
        from_node_key: from.into(),
        to_node_key: to.into(),
        branch_label: None,
        sort_order: 0,
    };
    let graph = execution_graph_service::create(
        &ws.conn,
        &ws.id,
        &ExecutionGraphInput {
            name: "Team".into(),
            description: None,
            nodes: vec![
                node("trigger", "trigger", json!({})),
                node(
                    "work",
                    "agent",
                    json!({"agent_id": a, "input_template": "Work"}),
                ),
                node("end", "end", json!({})),
            ],
            edges: vec![edge("trigger", "work"), edge("work", "end")],
        },
        Some(&ws.admin),
    )
    .unwrap();
    let graph =
        execution_graph_service::publish(&ws.conn, &graph.id, &ws.id, Some(&ws.admin)).unwrap();
    script(vec![tool_call(), text("done")]);
    let mark = audit_mark(&ws.conn);
    let run = graph_runtime_service::start_run(
        &ws.conn,
        &ws.id,
        &KEY,
        &graph.id,
        "Go",
        Some(&ws.admin),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(run.status, "completed", "{run:?}");
    let trace = assert_full_trace(&ws.conn, &ws.id, "graph", &run.id, 1, "graph agent node");
    assert_tool_audit(&ws.conn, mark, &trace.tools_run, "graph agent node");
    assert_eq!(trace.tools_run, ["update_memory"]);
}

#[tokio::test]
async fn the_run_ai_agent_workflow_action_produces_a_full_trace_and_tool_audit() {
    let ws = workspace();
    let a = agent(&ws, "Workflow agent");
    workflow_service::create_rule(
        &ws.conn,
        &ws.id,
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
                params_json: json!({"target_type": "agent", "target_id": a, "input_template": "Welcome them"})
                    .to_string(),
            }],
        },
        Some(&ws.admin),
    )
    .unwrap();
    company_service::create(
        &ws.conn,
        &ws.id,
        &CompanyInput {
            name: "Acme".into(),
            status: "Active Customer".into(),
            ..Default::default()
        },
        Some(&ws.admin),
    )
    .unwrap();
    script(vec![tool_call(), text("welcomed")]);
    let mark = audit_mark(&ws.conn);
    assert_eq!(
        ai_orchestration_service::drain_pending_runs(&ws.conn, &ws.id, &KEY, 10)
            .await
            .unwrap(),
        1
    );
    let (run_id, steps) = latest_agent_run(&ws, &a);
    check_pipeline_run(&ws, &run_id, steps, mark, "workflow action");
}

#[tokio::test]
async fn a_schedule_trigger_produces_a_full_trace_and_tool_audit() {
    let ws = workspace();
    let a = agent(&ws, "Nightly");
    trigger(&ws, &a, "schedule");
    assert_eq!(
        ai_orchestration_service::enqueue_due_schedules(&ws.conn, &ws.id).unwrap(),
        1
    );
    script(vec![tool_call(), text("done")]);
    let mark = audit_mark(&ws.conn);
    ai_orchestration_service::drain_pending_runs(&ws.conn, &ws.id, &KEY, 10)
        .await
        .unwrap();
    let (run_id, steps) = latest_agent_run(&ws, &a);
    check_pipeline_run(&ws, &run_id, steps, mark, "schedule trigger");
    // An unattended run is attributed to its Agent Identity.
    let audits = assert_tool_audit(&ws.conn, mark, &["update_memory".to_string()], "schedule");
    assert_eq!(audits[0]["effective_principal"]["type"], "agent_identity");
}

#[tokio::test]
async fn a_webhook_trigger_produces_a_full_trace_and_tool_audit() {
    let ws = workspace();
    let a = agent(&ws, "Hooked");
    trigger(&ws, &a, "webhook");
    script(vec![tool_call(), text("done")]);
    let mark = audit_mark(&ws.conn);
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
    check_pipeline_run(&ws, &run.id, run.steps.len(), mark, "webhook trigger");
}

/// A chat with an agent is a conversation, not a run: no trace is written
/// (WP-0111), but its tool calls are audited like any run's.
#[tokio::test]
async fn a_chat_with_an_agent_audits_its_tool_calls() {
    let ws = workspace();
    let a = agent(&ws, "Chatty");
    script(vec![tool_call(), text("noted")]);
    let mark = audit_mark(&ws.conn);
    let spans_before: i64 = ws
        .conn
        .query_row("SELECT COUNT(*) FROM ai_run_spans", [], |r| r.get(0))
        .unwrap();
    chat_service::send_agent_message(&ws.conn, &ws.id, &KEY, &ws.admin, &a, "remember this")
        .await
        .unwrap();
    let audits = assert_tool_audit(&ws.conn, mark, &["update_memory".to_string()], "agent chat");
    assert_eq!(audits[0]["effective_principal"]["type"], "human");
    let spans_after: i64 = ws
        .conn
        .query_row("SELECT COUNT(*) FROM ai_run_spans", [], |r| r.get(0))
        .unwrap();
    assert_eq!(spans_before, spans_after, "a chat writes no trace");
}

/// The gate can fail: a trace with an unclosed span, a tool call without
/// its firewall verdict, or a tool call without its audit entry is refused.
#[tokio::test]
async fn the_checks_refuse_a_broken_trace_or_audit() {
    let ws = workspace();
    let a = agent(&ws, "Checked");
    let id = pipeline(&ws, "sequential", &[&a]);
    let run_once = || async {
        script(vec![tool_call(), text("done")]);
        let mark = audit_mark(&ws.conn);
        let run = ai_orchestration_service::run_manual(
            &ws.conn,
            &ws.id,
            &KEY,
            "pipeline",
            &id,
            "Acme",
            Some(&ws.admin),
        )
        .await
        .unwrap();
        (run.id, mark)
    };
    let refused = |f: &dyn Fn()| std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err();

    let (run_id, _) = run_once().await;
    ws.conn
        .execute(
            "UPDATE ai_run_spans SET ended_at = NULL, status = NULL WHERE run_id = ?1 AND kind = 'model_call'",
            [&run_id],
        )
        .unwrap();
    assert!(refused(&|| {
        assert_full_trace(&ws.conn, &ws.id, "pipeline", &run_id, 1, "unclosed");
    }));

    let (run_id, _) = run_once().await;
    ws.conn
        .execute(
            "DELETE FROM ai_run_spans WHERE run_id = ?1 AND kind = 'policy'",
            [&run_id],
        )
        .unwrap();
    assert!(refused(&|| {
        assert_full_trace(&ws.conn, &ws.id, "pipeline", &run_id, 1, "no verdict");
    }));

    let (run_id, mark) = run_once().await;
    let trace = assert_full_trace(&ws.conn, &ws.id, "pipeline", &run_id, 1, "intact");
    ws.conn
        .execute(
            "DELETE FROM audit_events WHERE event_type = 'ai_agent_tool_call' AND rowid > ?1",
            [mark],
        )
        .unwrap();
    assert!(refused(&|| {
        assert_tool_audit(&ws.conn, mark, &trace.tools_run, "no audit");
    }));
}
