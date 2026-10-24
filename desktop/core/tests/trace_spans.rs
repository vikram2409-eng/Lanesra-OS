//! WP-0111: every run records a span tree (ADR-0008 §1): run -> step ->
//! model_call -> tool_call -> policy, with a delegation nested as a child
//! step. The tree is compared against a golden file; the cost of writing
//! it is held to the 5 ms per round budget (WP-0015); bodies are kept only
//! when the workspace's `trace_bodies` setting allows; and every span
//! passes the do-not-persist hook before it is written.
//!
//! Runs go through the scripted provider (WP-0005). Set
//! `LANESRA_UPDATE_GOLDEN=1` to rewrite the golden file after a deliberate
//! change, then review the diff.

use std::path::PathBuf;

use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput};
use lanesra_core::models::ai_agent::{AiAgentDefinition, AiAgentInput};
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::execution_graph::{ExecutionGraphInput, GraphEdgeInput, GraphNodeInput};
use lanesra_core::models::run_trace::{RunSpan, TraceBodiesInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ai_service::{RequestedToolCall, TokenUsage};
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::trace_service::{self, PersistDraft};
use lanesra_core::services::{
    ai_agent_service, ai_orchestration_service, ai_provider_service, execution_graph_service,
    graph_runtime_service, workspace_service,
};
use serde_json::{json, Value};

const KEY: [u8; 32] = [111u8; 32];

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Trace Co".into(),
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
    (conn, workspace.id, admin.id)
}

/// A provider row that dispatches to the scripted provider (see
/// `agent_loop_limits.rs`).
fn scripted_provider_row(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    let provider = ai_provider_service::create(
        conn,
        ws,
        &KEY,
        &AiProviderInput {
            name: "scripted".into(),
            provider: "anthropic".into(),
            base_url: None,
            model: "scripted-model".into(),
            api_key: Some("k".into()),
        },
        Some(admin),
    )
    .unwrap();
    conn.execute(
        "UPDATE ai_providers SET provider = 'scripted' WHERE id = ?1",
        [&provider.id],
    )
    .unwrap();
    provider.id
}

fn agent(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    name: &str,
    delegates: Vec<String>,
    routing: AiAgentModelRouting,
) -> AiAgentDefinition {
    let created = ai_agent_service::create(
        conn,
        ws,
        &AiAgentInput {
            name: name.into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: format!("You are {name}."),
            action_names: vec!["list_objects".into()],
            delegate_agent_ids: delegates,
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(admin),
    )
    .unwrap();
    ai_agent_service::set_model_routing(conn, &created.id, ws, Some(routing), Some(admin)).unwrap()
}

fn routed(provider: &str) -> AiAgentModelRouting {
    AiAgentModelRouting {
        primary_provider_id: Some(provider.to_string()),
        ..Default::default()
    }
}

fn usage(input: i64, output: i64) -> TokenUsage {
    TokenUsage {
        input_tokens: input,
        output_tokens: output,
    }
}

fn tool_call(id: &str, name: &str, arguments: Value) -> ScriptedResponse {
    ScriptedResponse::ToolCalls {
        calls: vec![RequestedToolCall {
            id: id.into(),
            name: name.into(),
            arguments,
            malformed: None,
        }],
        usage: usage(100, 20),
    }
}

fn text(t: &str) -> ScriptedResponse {
    ScriptedResponse::Text {
        text: t.into(),
        usage: usage(80, 30),
    }
}

fn pipeline(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    agent_id: &str,
    approval: bool,
) -> String {
    ai_orchestration_service::create_pipeline(
        conn,
        ws,
        &AiAgentPipelineInput {
            name: "Brief".into(),
            description: None,
            topology: "sequential".into(),
            steps: vec![PipelineStepInput {
                agent_id: agent_id.into(),
                input_template: "{{trigger_input}}".into(),
                requires_approval: approval,
            }],
        },
        Some(admin),
    )
    .unwrap()
    .id
}

async fn run_pipeline(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    pipeline_id: &str,
    script: Vec<ScriptedResponse>,
) -> String {
    scripted_provider::reset();
    scripted_provider::script_responses(script);
    let run = ai_orchestration_service::run_manual(
        conn,
        ws,
        &KEY,
        "pipeline",
        pipeline_id,
        "Write a brief on Acme",
        Some(admin),
    )
    .await
    .unwrap();
    run.id
}

fn spans(conn: &rusqlite::Connection, ws: &str, kind: &str, run_id: &str) -> Vec<RunSpan> {
    trace_service::spans_for_run(conn, ws, kind, run_id).unwrap()
}

/// The attributes a golden tree compares: stable, meaningful ones only (no
/// ids, times or durations).
const GOLDEN_ATTRIBUTES: &[&str] = &[
    "lanesra.run.kind",
    "lanesra.run.target_type",
    "lanesra.run.topology",
    "lanesra.trace_bodies",
    "lanesra.step.kind",
    "lanesra.step.order",
    "lanesra.agent.name",
    "lanesra.round",
    "lanesra.tools_off",
    "lanesra.served_by",
    "gen_ai.system",
    "gen_ai.request.model",
    "gen_ai.usage.input_tokens",
    "gen_ai.usage.output_tokens",
    "lanesra.model.reply",
    "lanesra.tool.name",
    "lanesra.tool.source",
    "lanesra.tool.outcome",
    "lanesra.policy.verdict",
    "lanesra.policy.risk",
];

fn tree(all: &[RunSpan], parent: Option<&str>) -> Vec<Value> {
    all.iter()
        .filter(|s| s.parent_span_id.as_deref() == parent)
        .map(|s| {
            let attrs: serde_json::Map<String, Value> = GOLDEN_ATTRIBUTES
                .iter()
                .filter_map(|k| s.attributes.get(*k).map(|v| (k.to_string(), v.clone())))
                .collect();
            let name = if s.kind == "run" {
                "<run>".to_string()
            } else {
                s.name.clone()
            };
            json!({
                "kind": s.kind,
                "name": name,
                "status": s.status,
                "attributes": attrs,
                "children": tree(all, Some(&s.span_id)),
            })
        })
        .collect()
}

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn assert_golden(name: &str, actual: &Value) {
    let path = golden_path(name);
    let pretty = serde_json::to_string_pretty(actual).unwrap() + "\n";
    if std::env::var("LANESRA_UPDATE_GOLDEN").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &pretty).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e} (run with LANESRA_UPDATE_GOLDEN=1)", path.display()));
    assert_eq!(
        pretty,
        expected,
        "the span tree differs from {}",
        path.display()
    );
}

/// Every span belongs to the run's one trace, has OpenTelemetry-sized ids,
/// a parent inside the run (except the root), and is closed.
fn assert_well_formed(all: &[RunSpan]) {
    let roots: Vec<&RunSpan> = all.iter().filter(|s| s.kind == "run").collect();
    assert_eq!(roots.len(), 1, "one root span");
    let trace_id = &roots[0].trace_id;
    assert_eq!(trace_id.len(), 32);
    assert!(trace_id.chars().all(|c| c.is_ascii_hexdigit()));
    let ids: Vec<&str> = all.iter().map(|s| s.span_id.as_str()).collect();
    for (i, s) in all.iter().enumerate() {
        assert_eq!(&s.trace_id, trace_id, "{} is in another trace", s.name);
        assert_eq!(s.span_id.len(), 16);
        if s.kind != "run" {
            let parent = s.parent_span_id.as_deref().expect("a parent");
            assert!(
                ids.contains(&parent),
                "{}'s parent is outside the run",
                s.name
            );
        }
        if i > 0 {
            assert!(s.seq > all[i - 1].seq, "seq increases");
        }
        assert!(
            s.ended_at.is_some() && s.status.is_some(),
            "{} was left open",
            s.name
        );
    }
}

#[tokio::test]
async fn ac1_a_pipeline_run_with_delegation_records_the_expected_tree() {
    // Covers: REQ-OBS-001, REQ-OBS-002 (WP-0111 AC-1)
    let (conn, ws, admin) = setup_workspace();
    let provider = scripted_provider_row(&conn, &ws, &admin);
    let researcher = agent(&conn, &ws, &admin, "Researcher", vec![], routed(&provider));
    let lead = agent(
        &conn,
        &ws,
        &admin,
        "Lead",
        vec![researcher.id.clone()],
        routed(&provider),
    );
    let pipeline_id = pipeline(&conn, &ws, &admin, &lead.id, false);
    let run_id = run_pipeline(
        &conn,
        &ws,
        &admin,
        &pipeline_id,
        vec![
            // Lead delegates; Researcher lists objects, then answers; Lead answers.
            tool_call(
                "c1",
                "delegate_to_agent",
                json!({"agent_name": "Researcher", "input": "Find facts on Acme"}),
            ),
            tool_call("c2", "list_objects", json!({})),
            text("Acme makes widgets."),
            text("Brief: Acme makes widgets."),
        ],
    )
    .await;
    let all = spans(&conn, &ws, "pipeline", &run_id);
    assert_well_formed(&all);
    assert_golden("trace_pipeline_delegation.json", &json!(tree(&all, None)));
}

#[tokio::test]
async fn ac1_a_graph_run_records_a_step_per_node_and_a_waiting_run_continues_its_trace() {
    // Covers: REQ-OBS-001 (WP-0111 AC-1): a delay node leaves the run
    // waiting with its root open; resuming continues the same trace.
    let (conn, ws, admin) = setup_workspace();
    let provider = scripted_provider_row(&conn, &ws, &admin);
    let writer = agent(&conn, &ws, &admin, "Writer", vec![], routed(&provider));
    let node = |key: &str, kind: &str, config: Value| GraphNodeInput {
        node_key: key.into(),
        node_type: kind.into(),
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
        &conn,
        &ws,
        &ExecutionGraphInput {
            name: "Wait then write".into(),
            description: None,
            nodes: vec![
                node("trigger", "trigger", json!({})),
                node("wait", "delay", json!({"delay_seconds": 0})),
                node(
                    "write",
                    "agent",
                    json!({"agent_id": writer.id, "input_template": "{{trigger_input}}"}),
                ),
                node("end", "end", json!({})),
            ],
            edges: vec![
                edge("trigger", "wait"),
                edge("wait", "write"),
                edge("write", "end"),
            ],
        },
        Some(&admin),
    )
    .unwrap();
    let graph = execution_graph_service::publish(&conn, &graph.id, &ws, Some(&admin)).unwrap();
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text("Done.")]);
    let run = graph_runtime_service::start_run(
        &conn,
        &ws,
        &KEY,
        &graph.id,
        "Go",
        Some(&admin),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_scheduled");
    let waiting = spans(&conn, &ws, "graph", &run.id);
    let root = waiting.iter().find(|s| s.kind == "run").unwrap();
    assert!(
        root.ended_at.is_none(),
        "a waiting run keeps its trace open"
    );

    // A second run of the same graph, cancelled while it waits, closes its
    // trace as cancelled.
    let other = graph_runtime_service::start_run(
        &conn,
        &ws,
        &KEY,
        &graph.id,
        "Go",
        Some(&admin),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    graph_runtime_service::cancel_run(&conn, &ws, &other.id, Some(&admin))
        .await
        .unwrap();
    let cancelled = spans(&conn, &ws, "graph", &other.id);
    let root = cancelled.iter().find(|s| s.kind == "run").unwrap();
    assert_eq!(root.status.as_deref(), Some("cancelled"));
    assert_eq!(cancelled.iter().filter(|s| s.kind == "run").count(), 1);

    graph_runtime_service::resume_due_delays(&conn, &ws, &KEY)
        .await
        .unwrap();
    let all = spans(&conn, &ws, "graph", &run.id);
    assert_well_formed(&all);
    let root = all.iter().find(|s| s.kind == "run").unwrap();
    assert_eq!(root.status.as_deref(), Some("ok"));
    let steps: Vec<&str> = all
        .iter()
        .filter(|s| s.kind == "step")
        .map(|s| s.name.as_str())
        .collect();
    assert_eq!(steps, ["trigger", "wait", "write"]);
    let write = all.iter().find(|s| s.name == "write").unwrap();
    assert!(
        all.iter()
            .any(|s| s.kind == "model_call" && s.parent_span_id.as_deref() == Some(&write.span_id)),
        "the agent node's model call nests under its step"
    );
}

#[tokio::test]
async fn ac2_writing_the_trace_costs_less_than_5_ms_per_round() {
    // Covers: REQ-OBS-002 (WP-0111 AC-2): the budget agreed for WP-0015.
    let (conn, ws, admin) = setup_workspace();
    let provider = scripted_provider_row(&conn, &ws, &admin);
    let worker = agent(&conn, &ws, &admin, "Worker", vec![], routed(&provider));
    let pipeline_id = pipeline(&conn, &ws, &admin, &worker.id, false);
    let mut script: Vec<ScriptedResponse> = (0..7)
        .map(|n| tool_call(&format!("c{n}"), "list_objects", json!({"page": n})))
        .collect();
    script.push(text("All listed."));
    trace_service::reset_write_time_on_this_thread();
    let run_id = run_pipeline(&conn, &ws, &admin, &pipeline_id, script).await;
    let spent = trace_service::write_time_on_this_thread();
    let all = spans(&conn, &ws, "pipeline", &run_id);
    let rounds = all.iter().filter(|s| s.kind == "model_call").count();
    assert_eq!(rounds, 8);
    let per_round = spent / rounds as u32;
    println!(
        "trace writes: {spent:?} for {rounds} rounds and {} spans, {per_round:?} per round",
        all.len()
    );
    assert!(
        per_round.as_micros() < 5_000,
        "{per_round:?} per round is over the 5 ms budget"
    );
}

fn persist_hook(draft: &mut PersistDraft) {
    // Stands in for WP-0127's do-not-persist state fields: an attribute
    // and any body carrying the marker never reach storage.
    if draft
        .attributes
        .get("lanesra.agent.name")
        .is_some_and(|v| v == "HookAgent")
    {
        draft.attributes.remove("lanesra.agent.name");
    }
    draft
        .bodies
        .retain(|(_, body)| !body.contains("DO-NOT-PERSIST"));
}

#[tokio::test]
async fn ac3_every_span_passes_the_do_not_persist_hook_before_it_is_written() {
    // Covers: REQ-OBS-002 (WP-0111 AC-3)
    trace_service::register_persist_hook(persist_hook);
    let (conn, ws, admin) = setup_workspace();
    trace_service::set_trace_bodies(
        &conn,
        &ws,
        &TraceBodiesInput {
            trace_bodies: "full".into(),
        },
        Some(&admin),
    )
    .unwrap();
    let provider = scripted_provider_row(&conn, &ws, &admin);
    let hooked = agent(&conn, &ws, &admin, "HookAgent", vec![], routed(&provider));
    let pipeline_id = pipeline(&conn, &ws, &admin, &hooked.id, false);
    let run_id = run_pipeline(
        &conn,
        &ws,
        &admin,
        &pipeline_id,
        vec![text("Answer with DO-NOT-PERSIST state inside.")],
    )
    .await;
    let all = spans(&conn, &ws, "pipeline", &run_id);
    let step = all.iter().find(|s| s.kind == "step").unwrap();
    assert!(step.attributes.get("lanesra.agent.name").is_none());
    let model = all.iter().find(|s| s.kind == "model_call").unwrap();
    let bodies = trace_service::span_bodies(&conn, &ws, &model.span_id, Some(&admin)).unwrap();
    assert!(
        bodies.iter().all(|b| !b.body.contains("DO-NOT-PERSIST")),
        "{bodies:?}"
    );
    assert!(
        bodies.iter().any(|b| b.part == "input"),
        "other bodies are still kept"
    );
}

#[tokio::test]
async fn bodies_are_kept_only_as_the_trace_bodies_setting_says() {
    // Covers: REQ-OBS-002 (WP-0111, ADR-0008 §4, owner decision ADR-0013 #2)
    let (conn, ws, admin) = setup_workspace();
    let provider = scripted_provider_row(&conn, &ws, &admin);
    let a = agent(&conn, &ws, &admin, "Bodies", vec![], routed(&provider));
    let pipeline_id = pipeline(&conn, &ws, &admin, &a.id, false);
    let answer = "SSN 123-45-6789, token lanesra_agent_0123456789abcdef0123456789abcdef";
    let model_bodies = |conn: &rusqlite::Connection, run_id: &str| {
        let all = spans(conn, &ws, "pipeline", run_id);
        let model = all.iter().find(|s| s.kind == "model_call").unwrap();
        let root = all.iter().find(|s| s.kind == "run").unwrap();
        (
            root.attributes["lanesra.trace_bodies"].clone(),
            trace_service::span_bodies(conn, &ws, &model.span_id, Some(&admin)).unwrap(),
        )
    };

    // Off by default: nothing is written, and the trace says so.
    let run_id = run_pipeline(&conn, &ws, &admin, &pipeline_id, vec![text(answer)]).await;
    let (mode, bodies) = model_bodies(&conn, &run_id);
    assert_eq!(mode, "off");
    assert!(bodies.is_empty());
    let stored: i64 = conn
        .query_row("SELECT COUNT(*) FROM ai_run_span_bodies", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stored, 0);

    let set = |mode: &str| {
        trace_service::set_trace_bodies(
            &conn,
            &ws,
            &TraceBodiesInput {
                trace_bodies: mode.into(),
            },
            Some(&admin),
        )
        .unwrap()
    };

    // Redacted: DLP classes and agent credentials are removed.
    assert_eq!(set("redacted").trace_bodies, "redacted");
    let run_id = run_pipeline(&conn, &ws, &admin, &pipeline_id, vec![text(answer)]).await;
    let (mode, bodies) = model_bodies(&conn, &run_id);
    assert_eq!(mode, "redacted");
    let output = &bodies.iter().find(|b| b.part == "output").unwrap().body;
    assert!(
        !output.contains("123-45-6789") && !output.contains("0123456789abcdef"),
        "{output}"
    );

    // Full: kept as-is, still without credentials.
    set("full");
    let run_id = run_pipeline(&conn, &ws, &admin, &pipeline_id, vec![text(answer)]).await;
    let (mode, bodies) = model_bodies(&conn, &run_id);
    assert_eq!(mode, "full");
    let output = &bodies.iter().find(|b| b.part == "output").unwrap().body;
    assert!(output.contains("123-45-6789"), "{output}");
    assert!(!output.contains("0123456789abcdef"), "{output}");
}

#[test]
fn only_an_administrator_changes_trace_bodies_and_each_change_is_audited() {
    // Covers: REQ-OBS-002 (WP-0111, ADR-0008 §4)
    let (conn, ws, admin) = setup_workspace();
    let member = lanesra_core::services::user_service::create(
        &conn,
        &ws,
        &lanesra_core::models::user::NewUser {
            username: "member".into(),
            display_name: "Member".into(),
            password: "anothersecretpw".into(),
            roles: vec!["Sales".to_string()],
        },
        Some(&admin),
    )
    .unwrap();
    let input = |mode: &str| TraceBodiesInput {
        trace_bodies: mode.into(),
    };
    assert!(
        trace_service::set_trace_bodies(&conn, &ws, &input("full"), Some(&member.id)).is_err(),
        "a member cannot change it"
    );
    assert!(
        trace_service::set_trace_bodies(&conn, &ws, &input("everything"), Some(&admin)).is_err()
    );
    trace_service::set_trace_bodies(&conn, &ws, &input("redacted"), Some(&admin)).unwrap();
    let audited: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE event_type = 'ai_trace_bodies_changed'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(audited, 1);
}

#[tokio::test]
async fn a_chat_outside_any_run_writes_no_spans() {
    // Covers: REQ-OBS-001 (WP-0111): tracing is per run; the Assistant chat
    // is not a run.
    let (conn, ws, admin) = setup_workspace();
    lanesra_core::services::ai_service::save_settings(
        &conn,
        &ws,
        &KEY,
        &lanesra_core::models::ai::AiSettingsInput {
            provider: "anthropic".into(),
            base_url: None,
            model: "m".into(),
            api_key: Some("k".into()),
        },
        Some(&admin),
    )
    .unwrap();
    conn.execute(
        "UPDATE ai_settings SET provider = 'scripted' WHERE workspace_id = ?1",
        [&ws],
    )
    .unwrap();
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text("Hello.")]);
    lanesra_core::services::chat_service::send_message(&conn, &ws, &KEY, &admin, "records", "hi")
        .await
        .unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ai_run_spans", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}
