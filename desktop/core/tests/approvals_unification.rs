//! WP-0150: every pause for a person goes through `ai_approvals`, and the
//! approvals inbox (`approval_service::decide`) resumes whatever waits.
//!
//! AC-1: each pipeline topology's approval step pauses on an approval the
//! inbox decides; an execution-graph Approval node decided from the inbox
//! no longer leaves its run stuck (RAID I-13); a tool call the policy holds
//! pauses its pipeline step or graph node, and approving it runs exactly
//! that call once; a voice plan that needs approval can be decided.
//! AC-2: pauses already waiting when the release is installed survive the
//! upgrade (migration 0088) and are decided the same way.
//!
//! Runs go through the scripted provider (WP-0005).
//!
//! Covers: REQ-GOV-011 (WP-0150). Threat: T-05.

use lanesra_core::db::migrate::{run_migrations, run_migrations_up_to};
use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput};
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::ai_agent_policy::AiAgentPolicyInput;
use lanesra_core::models::ai_approval::{AiApproval, AiApprovalResolution};
use lanesra_core::models::ai_tool_registry::RiskLevel;
use lanesra_core::models::execution_graph::{ExecutionGraphInput, GraphEdgeInput, GraphNodeInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ai_service::{RequestedToolCall, TokenUsage};
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    ai_agent_service, ai_orchestration_service, ai_provider_service, approval_service,
    execution_graph_service, graph_runtime_service, policy_engine_service, workspace_service,
};
use rusqlite::Connection;
use serde_json::{json, Value};

const KEY: [u8; 32] = [150u8; 32];

struct Ws {
    conn: Connection,
    id: String,
    admin: String,
}

fn setup(conn: Connection) -> Ws {
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Approvals Co".into(),
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

fn workspace() -> Ws {
    setup(lanesra_core::db::open_in_memory_db().unwrap())
}

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
            action_names: vec!["list_objects".into()],
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

fn pipeline(ws: &Ws, topology: &str, steps: Vec<(&str, bool)>) -> String {
    ai_orchestration_service::create_pipeline(
        &ws.conn,
        &ws.id,
        &AiAgentPipelineInput {
            name: format!("{topology} pipeline"),
            description: None,
            topology: topology.into(),
            steps: steps
                .into_iter()
                .map(|(agent_id, approval)| PipelineStepInput {
                    agent_id: agent_id.into(),
                    input_template: "{{trigger_input}} {{previous_output}}".into(),
                    requires_approval: approval,
                })
                .collect(),
        },
        Some(&ws.admin),
    )
    .unwrap()
    .id
}

fn usage() -> TokenUsage {
    TokenUsage {
        input_tokens: 10,
        output_tokens: 5,
    }
}

fn text(t: &str) -> ScriptedResponse {
    ScriptedResponse::Text {
        text: t.into(),
        usage: usage(),
    }
}

fn list_objects() -> ScriptedResponse {
    ScriptedResponse::ToolCalls {
        calls: vec![RequestedToolCall {
            id: "c1".into(),
            name: "list_objects".into(),
            arguments: json!({}),
            malformed: None,
        }],
        usage: usage(),
    }
}

/// A workspace policy that holds every tool call for approval.
fn hold_every_tool(ws: &Ws) {
    policy_engine_service::upsert_policy(
        &ws.conn,
        &ws.id,
        None,
        &AiAgentPolicyInput {
            require_approval_at_or_above: Some(RiskLevel::Read),
            blocked_tool_names: vec![],
            exclude_restricted_memory: true,
            enforce_record_access: false,
        },
        Some(&ws.admin),
    )
    .unwrap();
}

async fn run_pipeline(
    ws: &Ws,
    pipeline_id: &str,
) -> lanesra_core::models::ai_agent_pipeline::AiAgentRun {
    ai_orchestration_service::run_manual(
        &ws.conn,
        &ws.id,
        &KEY,
        "pipeline",
        pipeline_id,
        "Brief on Acme",
        Some(&ws.admin),
    )
    .await
    .unwrap()
}

fn pending(ws: &Ws) -> Vec<AiApproval> {
    approval_service::list(&ws.conn, &ws.id, Some("pending"), Some(&ws.admin)).unwrap()
}

async fn decide(ws: &Ws, id: &str, approve: bool) -> AiApproval {
    approval_service::decide(
        &ws.conn,
        id,
        &ws.id,
        &KEY,
        &AiApprovalResolution {
            approve,
            resolution_notes: Some(if approve { "fine" } else { "no" }.into()),
        },
        Some(&ws.admin),
    )
    .await
    .unwrap()
}

fn get_run(ws: &Ws, id: &str) -> lanesra_core::models::ai_agent_pipeline::AiAgentRun {
    ai_orchestration_service::list_runs(&ws.conn, "pipeline", &run_target(ws, id), 50)
        .unwrap()
        .into_iter()
        .find(|r| r.id == id)
        .unwrap()
}

fn run_target(ws: &Ws, run_id: &str) -> String {
    ws.conn
        .query_row(
            "SELECT target_id FROM ai_agent_runs WHERE id = ?1",
            [run_id],
            |r| r.get(0),
        )
        .unwrap()
}

/// AC-1: in each topology an approval step pauses the run on a pending
/// approval in the inbox; deciding it there resumes the run to the end, or
/// ends it when rejected.
#[tokio::test]
async fn each_topology_pauses_on_an_approval_the_inbox_decides() {
    let ws = workspace();
    let a = agent(&ws, "First");
    let b = agent(&ws, "Second");
    let c = agent(&ws, "Third");
    let cases: Vec<(
        &str,
        Vec<(&str, bool)>,
        Vec<ScriptedResponse>,
        Vec<ScriptedResponse>,
    )> = vec![
        (
            "sequential",
            vec![(&a, true), (&b, false)],
            vec![text("notes")],
            vec![text("the brief")],
        ),
        (
            "consensus",
            vec![(&a, false), (&b, false), (&c, true)],
            vec![text("one view"), text("another view")],
            vec![text("the synthesis")],
        ),
        (
            "peer_review",
            vec![(&a, false), (&b, true)],
            vec![text("a draft"), text("APPROVED - good")],
            vec![],
        ),
    ];
    for (topology, steps, before, after) in cases {
        let id = pipeline(&ws, topology, steps.clone());
        scripted_provider::reset();
        scripted_provider::script_responses(before);
        let run = run_pipeline(&ws, &id).await;
        assert_eq!(run.status, "awaiting_approval", "{topology}: {run:?}");
        let approval_id = run.pending_approval_id.clone().expect(topology);
        let inbox = pending(&ws);
        let row = inbox.iter().find(|p| p.id == approval_id).expect(topology);
        assert_eq!(row.subject_type, "pipeline_run");
        assert_eq!(row.subject_id, run.id);

        scripted_provider::script_responses(after);
        let decided = decide(&ws, &approval_id, true).await;
        assert_eq!(decided.status, "approved");
        let resumed = get_run(&ws, &run.id);
        assert_eq!(resumed.status, "succeeded", "{topology}: {resumed:?}");
        assert_eq!(resumed.pending_approval_id, None);

        // Rejecting from the inbox ends the run.
        scripted_provider::script_responses(match topology {
            "sequential" => vec![text("notes")],
            "consensus" => vec![text("one view"), text("another view")],
            _ => vec![text("a draft"), text("APPROVED - good")],
        });
        let run = run_pipeline(&ws, &id).await;
        let approval_id = run.pending_approval_id.clone().unwrap();
        assert_eq!(decide(&ws, &approval_id, false).await.status, "rejected");
        assert_eq!(get_run(&ws, &run.id).status, "rejected", "{topology}");
    }
    assert!(pending(&ws).is_empty());
}

/// The run screen's own Approve still works, and decides the approval too.
#[tokio::test]
async fn approving_from_the_run_screen_decides_the_approval() {
    let ws = workspace();
    let a = agent(&ws, "First");
    let b = agent(&ws, "Second");
    let id = pipeline(&ws, "sequential", vec![(&a, true), (&b, false)]);
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text("notes"), text("the brief")]);
    let run = run_pipeline(&ws, &id).await;
    let approval_id = run.pending_approval_id.clone().unwrap();
    let done = ai_orchestration_service::approve_pending_step(
        &ws.conn,
        &ws.id,
        &KEY,
        &run.id,
        None,
        Some(&ws.admin),
    )
    .await
    .unwrap();
    assert_eq!(done.status, "succeeded");
    let approval = approval_service::get(&ws.conn, &approval_id, &ws.id, Some(&ws.admin)).unwrap();
    assert_eq!(approval.status, "approved");
    // The sync command path refuses an approval a run waits on.
    let id2 = pipeline(&ws, "sequential", vec![(&a, true), (&b, false)]);
    scripted_provider::script_responses(vec![text("notes")]);
    let run = run_pipeline(&ws, &id2).await;
    let err = approval_service::resolve_unless_waiting(
        &ws.conn,
        run.pending_approval_id.as_deref().unwrap(),
        &ws.id,
        &AiApprovalResolution {
            approve: true,
            resolution_notes: None,
        },
        Some(&ws.admin),
    )
    .unwrap_err();
    assert!(err.to_string().contains("approvals inbox"), "{err}");
}

/// AC-1, RAID I-13: an Approval node decided from the inbox continues its
/// graph run instead of leaving it stuck.
#[tokio::test]
async fn a_graph_approval_decided_from_the_inbox_continues_the_run() {
    let ws = workspace();
    let a = agent(&ws, "Writer");
    let graph = publish(
        &ws,
        vec![
            node("trigger", "trigger", json!({})),
            node("gate", "approval", json!({})),
            node(
                "write",
                "agent",
                json!({"agent_id": a, "input_template": "Write"}),
            ),
            node("end", "end", json!({})),
            node("stop", "end", json!({})),
        ],
        vec![
            edge("trigger", "gate", None),
            edge("gate", "write", Some("approved")),
            edge("gate", "stop", Some("rejected")),
            edge("write", "end", None),
        ],
    );
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text("written")]);
    let run = start_graph(&ws, &graph).await;
    assert_eq!(run.status, "waiting_approval");
    let approval_id = run.pending_approval_id.clone().unwrap();
    decide(&ws, &approval_id, true).await;
    let run = graph_runtime_service::get_run(&ws.conn, &run.id, &ws.id, Some(&ws.admin)).unwrap();
    assert_eq!(run.status, "completed", "{run:?}");
}

/// AC-1: a tool call the policy holds pauses the pipeline step; approving
/// it re-runs the step and that exact call runs once; the grant cannot be
/// used again.
#[tokio::test]
async fn a_held_tool_call_pauses_the_pipeline_and_runs_once_when_approved() {
    let ws = workspace();
    let a = agent(&ws, "Lister");
    let id = pipeline(&ws, "sequential", vec![(&a, false)]);
    hold_every_tool(&ws);
    scripted_provider::reset();
    scripted_provider::script_responses(vec![list_objects()]);
    let run = run_pipeline(&ws, &id).await;
    assert_eq!(run.status, "awaiting_approval", "{run:?}");
    let approval_id = run.pending_approval_id.clone().unwrap();
    let held = pending(&ws)
        .into_iter()
        .find(|p| p.id == approval_id)
        .unwrap();
    assert_eq!(held.subject_type, "tool_call");
    assert_eq!(held.proposal["tool_name"], "list_objects");

    // On approval the step runs again; the same call now goes through.
    scripted_provider::script_responses(vec![list_objects(), text("listed")]);
    decide(&ws, &approval_id, true).await;
    let resumed = get_run(&ws, &run.id);
    assert_eq!(resumed.status, "succeeded", "{resumed:?}");
    let grant = approval_service::get(&ws.conn, &approval_id, &ws.id, Some(&ws.admin)).unwrap();
    assert!(grant.consumed_at.is_some());
    // The tool really ran: its result reached the model.
    let captured = scripted_provider::take_captured();
    let last = captured.last().unwrap().history.last().unwrap();
    assert!(
        !last
            .content
            .clone()
            .unwrap_or_default()
            .contains("requires administrator approval"),
        "{last:?}"
    );

    // A new run makes the same call: the used grant does not cover it.
    scripted_provider::script_responses(vec![list_objects()]);
    let again = run_pipeline(&ws, &id).await;
    assert_eq!(again.status, "awaiting_approval");
    assert_ne!(
        again.pending_approval_id.as_deref(),
        Some(approval_id.as_str())
    );
}

/// AC-1: a held call in a graph agent node pauses the run on that node;
/// approving re-runs the node, rejecting fails the run.
#[tokio::test]
async fn a_held_tool_call_pauses_a_graph_agent_node() {
    let ws = workspace();
    let a = agent(&ws, "Lister");
    let graph = publish(
        &ws,
        vec![
            node("trigger", "trigger", json!({})),
            node(
                "list",
                "agent",
                json!({"agent_id": a, "input_template": "List"}),
            ),
            node("end", "end", json!({})),
        ],
        vec![edge("trigger", "list", None), edge("list", "end", None)],
    );
    hold_every_tool(&ws);
    scripted_provider::reset();
    scripted_provider::script_responses(vec![list_objects()]);
    let run = start_graph(&ws, &graph).await;
    assert_eq!(run.status, "waiting_approval", "{run:?}");
    scripted_provider::script_responses(vec![list_objects(), text("listed")]);
    decide(&ws, run.pending_approval_id.as_deref().unwrap(), true).await;
    let done = graph_runtime_service::get_run(&ws.conn, &run.id, &ws.id, Some(&ws.admin)).unwrap();
    assert_eq!(done.status, "completed", "{done:?}");

    scripted_provider::script_responses(vec![list_objects()]);
    let run = start_graph(&ws, &graph).await;
    decide(&ws, run.pending_approval_id.as_deref().unwrap(), false).await;
    let failed =
        graph_runtime_service::get_run(&ws.conn, &run.id, &ws.id, Some(&ws.admin)).unwrap();
    assert_eq!(failed.status, "failed");
    assert!(failed
        .error_message
        .unwrap_or_default()
        .contains("rejected"));
}

/// AC-2: a pipeline run and a voice plan waiting for approval before the
/// upgrade get pending approvals from migration 0088, and the inbox decides
/// them: the run resumes, the plan goes back to its speaker to confirm.
#[tokio::test]
async fn pauses_waiting_before_the_upgrade_survive_it() {
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    run_migrations_up_to(&conn, 87).unwrap();
    let ws = setup(conn);
    let a = agent(&ws, "First");
    let b = agent(&ws, "Second");
    let id = pipeline(&ws, "sequential", vec![(&a, true), (&b, false)]);
    // As the previous release left them: a paused run with no approval row,
    // and a voice plan waiting for approval that nothing could decide.
    ws.conn
        .execute(
            "INSERT INTO ai_agent_runs (id, workspace_id, target_type, target_id, status, triggered_by, trigger_input, started_at, paused_at_step_order, resume_previous_output)
             VALUES ('legacy-run', ?1, 'pipeline', ?2, 'awaiting_approval', 'manual', 'Brief on Acme', '2026-10-30T00:00:00Z', 1, 'notes')",
            [&ws.id, &id],
        )
        .unwrap();
    ws.conn
        .execute_batch(&format!(
            "INSERT INTO voice_sessions (id, workspace_id, user_id, state, unlocked_at, expires_at, created_at, updated_at)
               VALUES ('vs1', '{ws}', '{admin}', 'awaiting_approval', '2026-10-30T00:00:00Z', '2099-01-01T00:00:00Z', '2026-10-30T00:00:00Z', '2026-10-30T00:00:00Z');
             INSERT INTO voice_commands (id, session_id, workspace_id, user_id, transcript, status, correlation_id, created_at)
               VALUES ('vc1', 'vs1', '{ws}', '{admin}', 'delete every lead', 'planned', 'corr', '2026-10-30T00:00:00Z');
             INSERT INTO voice_action_plans (id, command_id, plan_json, risk, confirmation_required, status, created_at, updated_at)
               VALUES ('vp1', 'vc1', '{{}}', 'high', 1, 'awaiting_approval', '2026-10-30T00:00:00Z', '2026-10-30T00:00:00Z');",
            ws = ws.id,
            admin = ws.admin
        ))
        .unwrap();

    run_migrations(&ws.conn).unwrap();

    let inbox = pending(&ws);
    let run_approval = inbox
        .iter()
        .find(|p| p.subject_type == "pipeline_run" && p.subject_id == "legacy-run")
        .expect("the paused run has an approval");
    assert_eq!(run_approval.proposal["proposed_output"], "notes");
    let plan_approval = inbox
        .iter()
        .find(|p| p.subject_type == "voice_plan" && p.subject_id == "vp1")
        .expect("the voice plan has an approval");
    let linked: String = ws
        .conn
        .query_row(
            "SELECT pending_approval_id FROM ai_agent_runs WHERE id = 'legacy-run'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(linked, run_approval.id);

    scripted_provider::reset();
    scripted_provider::script_responses(vec![text("the brief")]);
    decide(&ws, &run_approval.id.clone(), true).await;
    assert_eq!(get_run(&ws, "legacy-run").status, "succeeded");

    decide(&ws, &plan_approval.id.clone(), true).await;
    let (plan_status, session_state): (String, String) = ws
        .conn
        .query_row(
            "SELECT p.status, s.state FROM voice_action_plans p JOIN voice_commands c ON c.id = p.command_id JOIN voice_sessions s ON s.id = c.session_id WHERE p.id = 'vp1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(plan_status, "awaiting_confirmation");
    assert_eq!(session_state, "awaiting_confirmation");
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

fn edge(from: &str, to: &str, label: Option<&str>) -> GraphEdgeInput {
    GraphEdgeInput {
        from_node_key: from.into(),
        to_node_key: to.into(),
        branch_label: label.map(str::to_string),
        sort_order: 0,
    }
}

fn publish(
    ws: &Ws,
    nodes: Vec<GraphNodeInput>,
    edges: Vec<GraphEdgeInput>,
) -> lanesra_core::models::execution_graph::ExecutionGraph {
    let graph = execution_graph_service::create(
        &ws.conn,
        &ws.id,
        &ExecutionGraphInput {
            name: "Team".into(),
            description: None,
            nodes,
            edges,
        },
        Some(&ws.admin),
    )
    .unwrap();
    execution_graph_service::publish(&ws.conn, &graph.id, &ws.id, Some(&ws.admin)).unwrap()
}

async fn start_graph(
    ws: &Ws,
    graph: &lanesra_core::models::execution_graph::ExecutionGraph,
) -> lanesra_core::models::graph_run::GraphRun {
    graph_runtime_service::start_run(
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
    .unwrap()
}
