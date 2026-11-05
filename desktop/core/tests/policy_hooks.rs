//! WP-0128: policy hook points around the agent loop (REQ-LOOP-017).
//!
//! AC-1: a guardrail attached at each of the three hook points (step
//! boundary, model call, tool call) can block execution; the first blocking
//! hook short-circuits the remaining hooks, which are evaluated in order
//! (step, then model, then tool).
//! AC-2: every verdict - allow as well as block - is recorded on a `policy`
//! trace span marked with a `lanesra.policy.hook` attribute (ADR-0008) and
//! emitted as an auditable `policy.decided` run event.
//!
//! The engine functions tested here (`evaluate_loop_hook`,
//! `evaluate_loop_round`) are exactly the ones the run loop calls at each
//! point (`graph_runtime_service`, `ai_orchestration_service`,
//! `chat_service`); the final tests drive a real pipeline run through the
//! scripted provider so the wiring is exercised end to end.
//!
//! Covers: REQ-LOOP-017 (WP-0128 AC-1, AC-2)

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput};
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::ai_guardrail::{GuardrailEdit, GuardrailInput};
use lanesra_core::models::run_event::RunEvent;
use lanesra_core::models::run_trace::RunSpan;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::run_event_repo;
use lanesra_core::services::ai_service::{RequestedToolCall, TokenUsage};
use lanesra_core::services::policy_engine_service::{self, HookPoint, HookVerdict, LoopSubjects};
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    ai_agent_service, ai_orchestration_service, ai_provider_service, guardrail_service as gs,
    trace_service, workspace_service,
};
use serde_json::{json, Value};

const KEY: [u8; 32] = [128u8; 32];

struct Ws {
    conn: rusqlite::Connection,
    id: String,
    admin: String,
}

fn workspace() -> Ws {
    let conn = open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Hooks Co".into(),
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

/// Attaches a `forbid_substrings` rule guardrail at `hook_point` that fails
/// when the subject contains `forbid`.
fn forbid_at(ws: &Ws, name: &str, hook_point: &str, forbid: &str) -> String {
    let g = gs::create(
        &ws.conn,
        &ws.id,
        &GuardrailInput {
            name: name.into(),
            description: None,
            checker_type: "rule".into(),
            config_json: json!({"mode": "forbid_substrings", "substrings": [forbid]}).to_string(),
            hook_point: hook_point.into(),
        },
        Some(&ws.admin),
    )
    .unwrap();
    assert_eq!(g.hook_point, hook_point);
    g.id
}

fn policy_events(ws: &Ws, run_id: &str) -> Vec<RunEvent> {
    run_event_repo::list_for_run(&ws.conn, &ws.id, run_id, 0)
        .unwrap()
        .into_iter()
        .filter(|e| e.event_type == "policy.decided")
        .collect()
}

fn policy_hook_spans(ws: &Ws, run_kind: &str, run_id: &str) -> Vec<RunSpan> {
    trace_service::spans_for_run(&ws.conn, &ws.id, run_kind, run_id)
        .unwrap()
        .into_iter()
        // A loop hook shares the `policy` kind with the firewall but carries
        // the hook attribute.
        .filter(|s| s.kind == "policy" && s.attributes.get("lanesra.policy.hook").is_some())
        .collect()
}

// ---- engine: order + short-circuit (AC-1) --------------------------------

#[test]
fn a_block_at_the_step_boundary_short_circuits_model_and_tool() {
    // Covers: REQ-LOOP-017 (WP-0128 AC-1)
    let ws = workspace();
    forbid_at(&ws, "no-secret-step", "step", "SECRET");
    forbid_at(&ws, "no-secret-model", "model", "SECRET");
    forbid_at(&ws, "no-secret-tool", "tool", "SECRET");

    let trace = trace_service::begin_run(&ws.conn, &ws.id, "graph", "run-sc1", "g", json!({}));
    let verdict = policy_engine_service::evaluate_loop_round(
        &ws.conn,
        &ws.id,
        &LoopSubjects {
            step: Some("here is a SECRET"),
            model: Some("here is a SECRET"),
            tool: Some("here is a SECRET"),
        },
    )
    .unwrap();
    drop(trace);

    match verdict {
        HookVerdict::Block { hook, .. } => assert_eq!(hook, HookPoint::Step),
        other => panic!("expected a block at step, got {other:?}"),
    }
    // Only the step hook ran: the model and tool hooks were never reached.
    let events = policy_events(&ws, "run-sc1");
    assert_eq!(events.len(), 1, "{:?}", events);
    assert_eq!(events[0].data["hook"], json!("step"));
    assert_eq!(events[0].data["verdict"], json!("block"));
}

#[test]
fn hooks_run_in_order_and_a_model_block_short_circuits_the_tool_hook() {
    // Covers: REQ-LOOP-017 (WP-0128 AC-1)
    let ws = workspace();
    forbid_at(&ws, "no-secret-step", "step", "SECRET");
    forbid_at(&ws, "no-secret-model", "model", "SECRET");
    forbid_at(&ws, "no-secret-tool", "tool", "SECRET");

    let trace = trace_service::begin_run(&ws.conn, &ws.id, "graph", "run-sc2", "g", json!({}));
    let verdict = policy_engine_service::evaluate_loop_round(
        &ws.conn,
        &ws.id,
        &LoopSubjects {
            step: Some("clean step input"),
            model: Some("model said SECRET"),
            tool: Some("tool args with SECRET"),
        },
    )
    .unwrap();
    drop(trace);

    match verdict {
        HookVerdict::Block { hook, .. } => assert_eq!(hook, HookPoint::Model),
        other => panic!("expected a block at model, got {other:?}"),
    }
    // Step allowed, then model blocked; the tool hook was never reached.
    let events = policy_events(&ws, "run-sc2");
    let seen: Vec<(Value, Value)> = events
        .iter()
        .map(|e| (e.data["hook"].clone(), e.data["verdict"].clone()))
        .collect();
    assert_eq!(
        seen,
        vec![
            (json!("step"), json!("allow")),
            (json!("model"), json!("block")),
        ]
    );
}

// ---- engine: every verdict recorded (AC-2) -------------------------------

#[test]
fn an_allow_is_recorded_on_a_span_and_emitted_as_an_event() {
    // Covers: REQ-LOOP-017 (WP-0128 AC-2)
    let ws = workspace();
    forbid_at(&ws, "no-secret-model", "model", "SECRET");

    let trace = trace_service::begin_run(&ws.conn, &ws.id, "graph", "run-allow", "g", json!({}));
    let verdict =
        policy_engine_service::evaluate_loop_hook(&ws.conn, &ws.id, HookPoint::Model, "all clean")
            .unwrap();
    drop(trace);

    assert_eq!(verdict, HookVerdict::Allow);
    let spans = policy_hook_spans(&ws, "graph", "run-allow");
    assert_eq!(spans.len(), 1);
    assert_eq!(
        spans[0].attributes["lanesra.policy.verdict"],
        json!("allow")
    );
    assert_eq!(spans[0].status.as_deref(), Some("ok"));

    let events = policy_events(&ws, "run-allow");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data["hook"], json!("model"));
    assert_eq!(events[0].data["verdict"], json!("allow"));
}

#[test]
fn a_block_names_the_guardrail_on_the_span_and_the_event() {
    // Covers: REQ-LOOP-017 (WP-0128 AC-2)
    let ws = workspace();
    let id = forbid_at(&ws, "No secret", "tool", "SECRET");

    let trace = trace_service::begin_run(&ws.conn, &ws.id, "graph", "run-block", "g", json!({}));
    let verdict = policy_engine_service::evaluate_loop_hook(
        &ws.conn,
        &ws.id,
        HookPoint::Tool,
        "{\"q\":\"SECRET\"}",
    )
    .unwrap();
    drop(trace);

    match &verdict {
        HookVerdict::Block {
            hook,
            guardrail_id,
            guardrail_name,
            reason,
        } => {
            assert_eq!(*hook, HookPoint::Tool);
            assert_eq!(guardrail_id, &id);
            assert_eq!(guardrail_name, "No secret");
            assert!(reason.contains("SECRET"), "reason was {reason:?}");
        }
        other => panic!("expected a block, got {other:?}"),
    }
    // The block becomes a PolicyBlocked error for the run loop.
    assert!(matches!(
        verdict.into_error(),
        Some(lanesra_core::domain::AppError::PolicyBlocked(_))
    ));

    let spans = policy_hook_spans(&ws, "graph", "run-block");
    assert_eq!(spans.len(), 1);
    assert_eq!(
        spans[0].attributes["lanesra.policy.verdict"],
        json!("block")
    );
    assert_eq!(
        spans[0].attributes["lanesra.policy.guardrail"],
        json!("No secret")
    );
    assert_eq!(spans[0].status.as_deref(), Some("error"));

    let events = policy_events(&ws, "run-block");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data["hook"], json!("tool"));
    assert_eq!(events[0].data["verdict"], json!("block"));
    assert_eq!(events[0].data["guardrail"], json!("No secret"));
}

// ---- engine: a point with no attached guardrail is a no-op ---------------

#[test]
fn a_point_with_no_guardrail_attached_is_a_silent_allow() {
    // Covers: REQ-LOOP-017 (WP-0128 AC-2)
    let ws = workspace();
    // A guardrail that exists but is wired `off`, and one at a different
    // point, must not fire at the step boundary.
    forbid_at(&ws, "off-one", "off", "anything");
    forbid_at(&ws, "model-one", "model", "anything");

    let trace = trace_service::begin_run(&ws.conn, &ws.id, "graph", "run-noop", "g", json!({}));
    let verdict =
        policy_engine_service::evaluate_loop_hook(&ws.conn, &ws.id, HookPoint::Step, "anything")
            .unwrap();
    drop(trace);

    assert_eq!(verdict, HookVerdict::Allow);
    assert!(policy_hook_spans(&ws, "graph", "run-noop").is_empty());
    assert!(policy_events(&ws, "run-noop").is_empty());
}

#[test]
fn a_disabled_guardrail_does_not_run_at_its_hook_point() {
    // Covers: REQ-LOOP-017 (WP-0128 AC-1)
    let ws = workspace();
    let id = forbid_at(&ws, "no-secret-step", "step", "SECRET");
    gs::edit(
        &ws.conn,
        &ws.id,
        &id,
        &GuardrailEdit {
            name: None,
            description: None,
            enabled: Some(false),
            config_json: None,
            hook_point: None,
        },
        Some(&ws.admin),
    )
    .unwrap();

    let trace = trace_service::begin_run(&ws.conn, &ws.id, "graph", "run-off", "g", json!({}));
    let verdict =
        policy_engine_service::evaluate_loop_hook(&ws.conn, &ws.id, HookPoint::Step, "has SECRET")
            .unwrap();
    drop(trace);

    // Disabled, so it neither blocks nor records anything.
    assert_eq!(verdict, HookVerdict::Allow);
    assert!(policy_events(&ws, "run-off").is_empty());
}

// ---- integration: the wiring blocks a real pipeline run ------------------

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

fn one_step_pipeline(ws: &Ws, agent_id: &str) -> String {
    ai_orchestration_service::create_pipeline(
        &ws.conn,
        &ws.id,
        &AiAgentPipelineInput {
            name: "Brief".into(),
            description: None,
            topology: "sequential".into(),
            steps: vec![PipelineStepInput {
                agent_id: agent_id.into(),
                input_template: "{{trigger_input}}".into(),
                requires_approval: false,
            }],
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

async fn run(
    ws: &Ws,
    pipeline_id: &str,
    trigger: &str,
    script: Vec<ScriptedResponse>,
) -> (String, String) {
    scripted_provider::reset();
    scripted_provider::script_responses(script);
    match ai_orchestration_service::run_manual(
        &ws.conn,
        &ws.id,
        &KEY,
        "pipeline",
        pipeline_id,
        trigger,
        Some(&ws.admin),
    )
    .await
    {
        Ok(run) => (run.id, run.status),
        Err(e) => {
            let id: String = ws
                .conn
                .query_row(
                    "SELECT id FROM ai_agent_runs ORDER BY started_at DESC, rowid DESC LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            (id, format!("error: {e}"))
        }
    }
}

#[tokio::test]
async fn the_step_boundary_hook_blocks_a_real_pipeline_run() {
    // Covers: REQ-LOOP-017 (WP-0128 AC-1, AC-2)
    let ws = workspace();
    let writer = agent(&ws, "Writer");
    let pipeline = one_step_pipeline(&ws, &writer);
    forbid_at(&ws, "no-acme", "step", "Acme");

    // The model would answer, but the step hook blocks before it runs.
    let (run_id, status) = run(
        &ws,
        &pipeline,
        "Write about Acme",
        vec![text_response("hi")],
    )
    .await;
    assert_ne!(status, "succeeded", "run should have been blocked");

    let events = policy_events(&ws, &run_id);
    assert!(
        events
            .iter()
            .any(|e| e.data["hook"] == json!("step") && e.data["verdict"] == json!("block")),
        "{:?}",
        events
    );
}

#[tokio::test]
async fn the_model_call_hook_blocks_a_real_pipeline_run() {
    // Covers: REQ-LOOP-017 (WP-0128 AC-1, AC-2)
    let ws = workspace();
    let writer = agent(&ws, "Writer");
    let pipeline = one_step_pipeline(&ws, &writer);
    forbid_at(&ws, "no-forbidden", "model", "FORBIDDEN");

    let (run_id, status) = run(
        &ws,
        &pipeline,
        "Write a brief",
        vec![text_response("this contains FORBIDDEN content")],
    )
    .await;
    assert_ne!(status, "succeeded", "run should have been blocked");

    let events = policy_events(&ws, &run_id);
    assert!(
        events
            .iter()
            .any(|e| e.data["hook"] == json!("model") && e.data["verdict"] == json!("block")),
        "{:?}",
        events
    );
}

#[tokio::test]
async fn the_tool_call_hook_blocks_a_single_call_in_a_real_run() {
    // Covers: REQ-LOOP-017 (WP-0128 AC-1, AC-2)
    let ws = workspace();
    let writer = agent(&ws, "Writer");
    let pipeline = one_step_pipeline(&ws, &writer);
    forbid_at(&ws, "no-forbidden-args", "tool", "FORBIDDEN");

    // The agent tries a tool call whose arguments carry the forbidden token;
    // the tool hook stands in for the result, then the agent answers.
    let script = vec![
        ScriptedResponse::ToolCalls {
            calls: vec![RequestedToolCall {
                id: "c1".into(),
                name: "list_objects".into(),
                arguments: json!({"q": "FORBIDDEN"}),
                malformed: None,
            }],
            usage: usage(),
        },
        text_response("done"),
    ];
    let (run_id, _status) = run(&ws, &pipeline, "Go", script).await;

    let events = policy_events(&ws, &run_id);
    assert!(
        events
            .iter()
            .any(|e| e.data["hook"] == json!("tool") && e.data["verdict"] == json!("block")),
        "{:?}",
        events
    );
}

fn text_response(t: &str) -> ScriptedResponse {
    ScriptedResponse::Text {
        text: t.into(),
        usage: usage(),
    }
}
