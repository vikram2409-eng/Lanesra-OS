//! WP-0120: agent persona, compiled system prompt, and reasoning modes.
//!
//! AC-1: the compiled prompt is deterministic for the same inputs, and
//! carries the persona, instructions, memory, guardrails and ontology
//! context (REQ-CORE-001/002).
//! AC-2: Plan-first emits a plan span before any tool use and a
//! `plan.proposed` event; Reflect makes exactly one extra model call
//! (REQ-LOOP-015).
//! AC-3: an agent with no persona compiles to today's prompt unchanged.
//!
//! Covers: REQ-CORE-001, REQ-CORE-002, REQ-LOOP-015 (WP-0120)

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput};
use lanesra_core::models::ai_agent::{AiAgentInput, PersonaInput};
use lanesra_core::models::ai_memory::AgentMemoryContext;
use lanesra_core::models::run_trace::RunSpan;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::run_event_repo;
use lanesra_core::services::ai_service::TokenUsage;
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    ai_agent_service, ai_provider_service, chat_service, trace_service, workspace_service,
};
use serde_json::json;

const KEY: [u8; 32] = [120u8; 32];

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
            business_name: "Persona Co".into(),
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

fn agent(ws: &Ws, name: &str, system_prompt: &str) -> String {
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
            system_prompt: system_prompt.into(),
            action_names: vec![],
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

fn get_agent(ws: &Ws, id: &str) -> lanesra_core::models::ai_agent::AiAgentDefinition {
    ai_agent_service::list(&ws.conn, &ws.id, false)
        .unwrap()
        .into_iter()
        .find(|a| a.id == id)
        .unwrap()
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

async fn run(ws: &Ws, agent_id: &str, run_id: &str, script: Vec<ScriptedResponse>) -> String {
    scripted_provider::reset();
    scripted_provider::script_responses(script);
    let agent = get_agent(ws, agent_id);
    let trace =
        trace_service::begin_run(&ws.conn, &ws.id, "pipeline", run_id, &agent.name, json!({}));
    let outcome = chat_service::run_agent_once_with_text(
        &ws.conn,
        &ws.id,
        &KEY,
        Some(&ws.admin),
        &agent,
        "Do the task",
        &AgentMemoryContext {
            session_key: None,
            run_id: None,
        },
    )
    .await
    .unwrap();
    drop(trace);
    outcome.final_text
}

fn spans(ws: &Ws, run_id: &str) -> Vec<RunSpan> {
    trace_service::spans_for_run(&ws.conn, &ws.id, "pipeline", run_id).unwrap()
}

fn event_types(ws: &Ws, run_id: &str) -> Vec<String> {
    run_event_repo::list_for_run(&ws.conn, &ws.id, run_id, 0)
        .unwrap()
        .into_iter()
        .map(|e| e.event_type)
        .collect()
}

// ---- AC-1 / AC-3: the compiled prompt --------------------------------------

#[test]
fn the_compiled_prompt_is_deterministic_and_carries_the_persona() {
    // Covers: REQ-CORE-002 (WP-0120 AC-1)
    let ws = workspace();
    let id = agent(&ws, "Writer", "Follow the house style.");
    ai_agent_service::set_persona(
        &ws.conn,
        &id,
        &PersonaInput {
            role: "Billing specialist".into(),
            goal: "resolve invoice disputes".into(),
            background: "follows the finance playbook".into(),
            reasoning_mode: "direct".into(),
        },
        Some(&ws.admin),
    )
    .unwrap();
    ai_agent_service::set_guardrails(
        &ws.conn,
        &id,
        "Never promise a refund over $1,000.",
        Some(&ws.admin),
    )
    .unwrap();

    let a = ai_agent_service::compiled_prompt(&ws.conn, &id, Some(&ws.admin)).unwrap();
    let b = ai_agent_service::compiled_prompt(&ws.conn, &id, Some(&ws.admin)).unwrap();
    assert_eq!(
        a, b,
        "the compiled prompt must be deterministic for the same inputs"
    );

    // Persona comes first, then the instructions, then the guardrails.
    assert!(
        a.starts_with("Your persona for this work:"),
        "persona first:\n{a}"
    );
    assert!(a.contains("Role: Billing specialist"));
    assert!(a.contains("Goal: resolve invoice disputes"));
    assert!(a.contains("Background: follows the finance playbook"));
    assert!(a.contains("Follow the house style."));
    assert!(a.contains("Never promise a refund over $1,000."));
    let persona_at = a.find("Your persona").unwrap();
    let instr_at = a.find("Follow the house style.").unwrap();
    assert!(persona_at < instr_at, "persona precedes the instructions");
}

#[test]
fn an_agent_without_a_persona_compiles_to_todays_prompt() {
    // Covers: REQ-CORE-002 (WP-0120 AC-3)
    let ws = workspace();
    let id = agent(&ws, "Plain", "Just the instructions.");
    let compiled = ai_agent_service::compiled_prompt(&ws.conn, &id, Some(&ws.admin)).unwrap();
    assert_eq!(compiled, "Just the instructions.");
    assert!(!compiled.contains("Your persona"));
}

#[test]
fn an_unknown_reasoning_mode_is_rejected() {
    // Covers: REQ-LOOP-015 (WP-0120)
    let ws = workspace();
    let id = agent(&ws, "Writer", "x");
    let err = ai_agent_service::set_persona(
        &ws.conn,
        &id,
        &PersonaInput {
            role: "".into(),
            goal: "".into(),
            background: "".into(),
            reasoning_mode: "nonsense".into(),
        },
        Some(&ws.admin),
    );
    assert!(err.is_err());
}

// ---- AC-2: reasoning modes -------------------------------------------------

#[tokio::test]
async fn plan_first_emits_a_plan_span_and_event_before_tool_use() {
    // Covers: REQ-LOOP-015 (WP-0120 AC-2)
    let ws = workspace();
    let id = agent(&ws, "Planner", "Be careful.");
    ai_agent_service::set_persona(
        &ws.conn,
        &id,
        &PersonaInput {
            role: "".into(),
            goal: "".into(),
            background: "".into(),
            reasoning_mode: "plan_first".into(),
        },
        Some(&ws.admin),
    )
    .unwrap();
    // One planning call, then the answering round.
    let final_text = run(
        &ws,
        &id,
        "run-plan",
        vec![text("1. look it up\n2. answer"), text("the answer")],
    )
    .await;
    assert_eq!(final_text, "the answer");

    let all = spans(&ws, "run-plan");
    let plan: Vec<&RunSpan> = all
        .iter()
        .filter(|s| {
            s.kind == "step" && s.attributes.get("lanesra.step.kind") == Some(&json!("plan"))
        })
        .collect();
    assert_eq!(plan.len(), 1, "exactly one plan span");
    let plan_seq = plan[0].seq;
    // The plan span comes before the model_call that produced the answer.
    let answer_model = all.iter().find(|s| s.kind == "model_call").unwrap();
    assert!(
        plan_seq < answer_model.seq,
        "the plan span precedes the answering model call"
    );
    assert!(event_types(&ws, "run-plan").contains(&"plan.proposed".to_string()));
}

#[tokio::test]
async fn reflect_makes_exactly_one_extra_model_call() {
    // Covers: REQ-LOOP-015 (WP-0120 AC-2)
    let ws = workspace();
    let id = agent(&ws, "Reflector", "Be accurate.");
    ai_agent_service::set_persona(
        &ws.conn,
        &id,
        &PersonaInput {
            role: "".into(),
            goal: "".into(),
            background: "".into(),
            reasoning_mode: "reflect".into(),
        },
        Some(&ws.admin),
    )
    .unwrap();
    // The first answer is the draft; the reflect call returns the improved one.
    let final_text = run(
        &ws,
        &id,
        "run-reflect",
        vec![text("draft answer"), text("improved answer")],
    )
    .await;
    assert_eq!(
        final_text, "improved answer",
        "the reflect pass's output is the final answer"
    );

    let all = spans(&ws, "run-reflect");
    let model_calls = all.iter().filter(|s| s.kind == "model_call").count();
    assert_eq!(
        model_calls, 2,
        "one answering call plus exactly one reflect call"
    );
    let reflect = all
        .iter()
        .filter(|s| s.attributes.get("lanesra.reasoning") == Some(&json!("reflect")))
        .count();
    assert_eq!(
        reflect, 1,
        "exactly one model call is marked as the reflect pass"
    );
}

#[tokio::test]
async fn direct_mode_makes_no_extra_call() {
    // Covers: REQ-LOOP-015 (WP-0120 AC-3)
    let ws = workspace();
    let id = agent(&ws, "Direct", "Answer.");
    let final_text = run(&ws, &id, "run-direct", vec![text("just this")]).await;
    assert_eq!(final_text, "just this");
    let all = spans(&ws, "run-direct");
    assert_eq!(all.iter().filter(|s| s.kind == "model_call").count(), 1);
    assert!(!event_types(&ws, "run-direct").contains(&"plan.proposed".to_string()));
}
