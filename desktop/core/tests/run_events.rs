//! WP-0112: live run events (ADR-0008 §2, §3).
//!
//! AC-1: every event a run emits validates against its JSON Schema in
//! `program/spec/schemas/events/`, and the runs here between them emit
//! every type except the reserved `plan.proposed`.
//! AC-3: a run's events are numbered 1, 2, 3, ... with no gap, across a
//! pause for approval and the resume; the bus delivers them in that order.
//! AC-4: writing spans and publishing events stays under 5 ms per round.
//! AC-2 (who receives events) is proven over HTTP in
//! `server/tests/run_events.rs`; here, the read is refused to a non-admin.
//!
//! Runs go through the scripted provider (WP-0005).
//!
//! Covers: REQ-LOOP-005, REQ-OBS-003, REQ-API-005 (WP-0112).

#[path = "support/event_schema.rs"]
mod event_schema;

use std::collections::BTreeSet;
use std::sync::Arc;

use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput};
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::ai_agent_policy::AiAgentPolicyInput;
use lanesra_core::models::run_event::{RunEvent, RUN_EVENT_TYPES};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ai_service::{RequestedToolCall, TokenUsage};
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    ai_agent_service, ai_orchestration_service, ai_provider_service, policy_engine_service,
    run_event_service, trace_service, user_service, workspace_service,
};
use serde_json::{json, Value};

const KEY: [u8; 32] = [112u8; 32];

struct Ws {
    conn: rusqlite::Connection,
    id: String,
    admin: String,
}

fn workspace() -> Ws {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Events Co".into(),
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

/// An agent answered by the scripted provider.
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

fn pipeline(ws: &Ws, steps: Vec<(&str, bool)>) -> String {
    ai_orchestration_service::create_pipeline(
        &ws.conn,
        &ws.id,
        &AiAgentPipelineInput {
            name: "Brief".into(),
            description: None,
            topology: "sequential".into(),
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
        input_tokens: 100,
        output_tokens: 20,
    }
}

fn list_objects_call() -> ScriptedResponse {
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

fn text(t: &str) -> ScriptedResponse {
    ScriptedResponse::Text {
        text: t.into(),
        usage: usage(),
    }
}

async fn run(ws: &Ws, pipeline_id: &str, script: Vec<ScriptedResponse>) -> (String, String) {
    scripted_provider::reset();
    scripted_provider::script_responses(script);
    match ai_orchestration_service::run_manual(
        &ws.conn,
        &ws.id,
        &KEY,
        "pipeline",
        pipeline_id,
        "Write a brief on Acme",
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

fn events(ws: &Ws, run_id: &str) -> Vec<RunEvent> {
    run_event_service::list_run_events(&ws.conn, &ws.id, run_id, 0, Some(&ws.admin)).unwrap()
}

fn types(events: &[RunEvent]) -> Vec<String> {
    events.iter().map(|e| e.event_type.clone()).collect()
}

fn assert_gapless(events: &[RunEvent]) {
    let seqs: Vec<i64> = events.iter().map(|e| e.seq).collect();
    let expected: Vec<i64> = (1..=events.len() as i64).collect();
    assert_eq!(seqs, expected, "{:?}", types(events));
}

fn validate_all(events: &[RunEvent]) {
    for event in events {
        event_schema::assert_valid(&serde_json::to_value(event).unwrap());
    }
}

/// Every run here, taken together, emits every type but `plan.proposed`,
/// and each event matches its schema (AC-1); each run's numbering is
/// gapless, also across a pause for approval and the resume (AC-3).
#[tokio::test]
async fn every_event_matches_its_schema_and_each_run_numbers_its_events_without_gaps() {
    let ws = workspace();
    let researcher = agent(&ws, "Researcher");
    let writer = agent(&ws, "Writer");
    let mut seen: BTreeSet<String> = BTreeSet::new();

    // A two-step pipeline whose first step needs approval: it pauses, then
    // the Administrator approves and the second step runs.
    let gated = pipeline(&ws, vec![(&researcher, true), (&writer, false)]);
    let (run_id, status) = run(&ws, &gated, vec![list_objects_call(), text("notes")]).await;
    assert_eq!(status, "awaiting_approval");
    let paused = events(&ws, &run_id);
    assert_eq!(
        types(&paused),
        [
            "run.started",
            "step.started",
            // A tool call's span sits inside its round's model call, so the
            // tool reports before the round does.
            "tool.called",
            "round.completed",
            "round.completed",
            "step.completed",
            "approval.requested",
        ]
    );
    scripted_provider::script_responses(vec![text("the brief")]);
    let resumed = ai_orchestration_service::approve_pending_step(
        &ws.conn,
        &ws.id,
        &KEY,
        &run_id,
        None,
        Some(&ws.admin),
    )
    .await
    .unwrap();
    assert_eq!(resumed.status, "succeeded");
    let all = events(&ws, &run_id);
    assert_eq!(
        types(&all[paused.len()..]),
        [
            "step.started",
            "round.completed",
            "step.completed",
            "run.completed"
        ]
    );
    assert_gapless(&all);
    validate_all(&all);
    // Every event of the run carries it; a step's events carry the step.
    assert!(all
        .iter()
        .all(|e| e.run_id == run_id && e.run_kind == "pipeline"));
    let tool = all.iter().find(|e| e.event_type == "tool.called").unwrap();
    assert_eq!(tool.data["tool"], "list_objects");
    assert_eq!(tool.data["status"], "ok");
    assert!(tool.step_id.is_some());
    let round = all
        .iter()
        .find(|e| e.event_type == "round.completed")
        .unwrap();
    assert_eq!(round.data["input_tokens"], 100);
    assert_eq!(round.data["model"], "scripted-model");
    seen.extend(types(&all));

    // A policy that blocks the tool: the firewall refuses the call.
    policy_engine_service::upsert_policy(
        &ws.conn,
        &ws.id,
        None,
        &AiAgentPolicyInput {
            require_approval_at_or_above: None,
            blocked_tool_names: vec!["list_objects".into()],
            exclude_restricted_memory: true,
            enforce_record_access: false,
        },
        Some(&ws.admin),
    )
    .unwrap();
    let single = pipeline(&ws, vec![(&writer, false)]);
    let (run_id, status) = run(&ws, &single, vec![list_objects_call(), text("no data")]).await;
    assert_eq!(status, "succeeded");
    let denied = events(&ws, &run_id);
    let refusal = denied
        .iter()
        .find(|e| e.event_type == "tool.denied")
        .unwrap_or_else(|| panic!("{:?}", types(&denied)));
    assert_eq!(refusal.data["tool"], "list_objects");
    assert_eq!(refusal.data["verdict"], "deny");
    assert_gapless(&denied);
    validate_all(&denied);
    seen.extend(types(&denied));

    // A provider failure ends the run.
    let (run_id, status) = run(
        &ws,
        &single,
        vec![ScriptedResponse::Failure {
            kind: "malformed_json".into(),
        }],
    )
    .await;
    assert_ne!(status, "succeeded");
    let failed = events(&ws, &run_id);
    assert_eq!(
        failed.last().unwrap().event_type,
        "run.failed",
        "{:?}",
        types(&failed)
    );
    assert_gapless(&failed);
    validate_all(&failed);
    seen.extend(types(&failed));

    let expected: BTreeSet<String> = RUN_EVENT_TYPES
        .iter()
        .filter(|t| **t != "plan.proposed")
        .map(|t| t.to_string())
        .collect();
    assert_eq!(seen, expected);
}

/// The bus delivers the same events as the table holds, in `seq` order
/// (AC-3), and spans plus events stay within 5 ms per round (AC-4).
#[tokio::test]
async fn the_bus_delivers_stored_events_in_order_within_the_trace_budget() {
    let ws = workspace();
    let writer = agent(&ws, "Writer");
    let single = pipeline(&ws, vec![(&writer, false)]);
    let mut rx = run_event_service::subscribe();

    trace_service::reset_write_time_on_this_thread();
    // Two tool calls, then the answer: a third identical call would trip
    // the repeated-call guard.
    let rounds = 3;
    let mut script: Vec<ScriptedResponse> = (1..rounds).map(|_| list_objects_call()).collect();
    script.push(text("done"));
    let (run_id, status) = run(&ws, &single, script).await;
    assert_eq!(status, "succeeded", "{:?}", events(&ws, &run_id));
    let per_round = trace_service::write_time_on_this_thread() / rounds;
    assert!(
        per_round < std::time::Duration::from_millis(5),
        "spans and events took {per_round:?} per round"
    );

    let stored = events(&ws, &run_id);
    let mut delivered: Vec<Arc<RunEvent>> = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if event.run_id == run_id {
            delivered.push(event);
        }
    }
    let delivered: Vec<RunEvent> = delivered.iter().map(|e| (**e).clone()).collect();
    let strip = |events: &[RunEvent]| -> Vec<Value> {
        events
            .iter()
            .map(|e| serde_json::to_value(e).unwrap())
            .collect()
    };
    assert_eq!(strip(&delivered), strip(&stored));
    assert_gapless(&delivered);
    assert_eq!(
        stored
            .iter()
            .filter(|e| e.event_type == "round.completed")
            .count() as u32,
        rounds
    );
}

/// Only an Administrator reads a run's events; a run of another workspace,
/// or no run at all, is not found.
#[tokio::test]
async fn reading_events_needs_an_administrator() {
    let ws = workspace();
    let writer = agent(&ws, "Writer");
    let single = pipeline(&ws, vec![(&writer, false)]);
    let (run_id, _) = run(&ws, &single, vec![text("done")]).await;
    let member = user_service::create(
        &ws.conn,
        &ws.id,
        &lanesra_core::models::user::NewUser {
            username: "member".into(),
            display_name: "Member".into(),
            password: "anothersecretpw".into(),
            roles: vec!["Sales".to_string()],
        },
        Some(&ws.admin),
    )
    .unwrap();
    assert!(
        run_event_service::list_run_events(&ws.conn, &ws.id, &run_id, 0, Some(&member.id)).is_err()
    );
    assert!(run_event_service::list_run_events(&ws.conn, &ws.id, &run_id, 0, None).is_err());
    assert!(
        run_event_service::list_run_events(&ws.conn, "other", &run_id, 0, Some(&ws.admin)).is_err()
    );
    // Replay after a seq returns only what follows it.
    let all = events(&ws, &run_id);
    let tail =
        run_event_service::list_run_events(&ws.conn, &ws.id, &run_id, 2, Some(&ws.admin)).unwrap();
    assert_eq!(tail, all[2..].to_vec());
}

/// The schema checker itself refuses a broken event, so AC-1 can fail.
#[test]
fn the_schema_checker_refuses_a_broken_event() {
    let good = json!({
        "schema": "lanesra.run-event", "v": 1, "seq": 3, "type": "tool.denied",
        "at": "2026-10-30T00:00:00Z", "workspace_id": "ws", "run_kind": "pipeline",
        "run_id": "r", "step_id": null, "span_id": "s", "parent_span_id": null,
        "data": {"tool": "delete_record", "verdict": "deny", "risk": "destructive", "reason": null}
    });
    let check = |event: &Value| {
        let mut problems =
            event_schema::errors(&event_schema::load("envelope.v1.json"), event, "e");
        problems.extend(event_schema::errors(
            &event_schema::load("tool.denied.v1.json"),
            &event["data"],
            "e.data",
        ));
        problems
    };
    assert!(check(&good).is_empty(), "{:?}", check(&good));
    for broken in [
        {
            let mut e = good.clone();
            e["seq"] = json!(0);
            e
        },
        {
            let mut e = good.clone();
            e["data"]["verdict"] = json!("allow");
            e
        },
        {
            let mut e = good.clone();
            e["data"].as_object_mut().unwrap().remove("tool");
            e
        },
        {
            let mut e = good.clone();
            e["extra"] = json!(1);
            e
        },
    ] {
        assert!(!check(&broken).is_empty(), "accepted {broken}");
    }
}
