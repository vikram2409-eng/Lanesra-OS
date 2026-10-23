//! WP-0121: step limits and partial results. Every limit ends a run
//! gracefully: the round limit (default 8, at most 25) and the run token
//! limit ask the model once for a final answer with tools switched off and
//! keep it as a partial result (`completed_partial`); a model call that runs
//! past the time limit is cancelled and tried once on the next routing tier.
//! The agent's temperature and output limit reach the provider.
//!
//! Every run here goes through the scripted provider (WP-0005), so each
//! request the loop makes is captured and checked.

use lanesra_core::domain::AppError;
use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput, AiSettingsInput};
use lanesra_core::models::ai_agent::{AiAgentDefinition, AiAgentInput};
use lanesra_core::models::ai_memory::AgentMemoryContext;
use lanesra_core::models::execution_graph::{ExecutionGraphInput, GraphEdgeInput, GraphNodeInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ai_service::{RequestedToolCall, TokenUsage};
use lanesra_core::services::chat_service::{self, AgentRunOutcome};
use lanesra_core::services::scripted_provider::{self, CapturedRequest, ScriptedResponse};
use lanesra_core::services::{
    ai_agent_service, ai_provider_service, ai_service, execution_graph_service,
    graph_runtime_service, workspace_service,
};
use serde_json::json;

const KEY: [u8; 32] = [121u8; 32];

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Limits Co".into(),
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

/// A named provider row that dispatches to the scripted provider. The
/// settings API cannot select `scripted` (WP-0005 AC-3), so the row is
/// created as another provider and switched underneath.
fn scripted_provider_row(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> String {
    let provider = ai_provider_service::create(
        conn,
        ws,
        &KEY,
        &AiProviderInput {
            name: name.into(),
            provider: "anthropic".into(),
            base_url: None,
            model: format!("{name}-model"),
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
    routing: AiAgentModelRouting,
) -> AiAgentDefinition {
    let created = ai_agent_service::create(
        conn,
        ws,
        &AiAgentInput {
            name: "Researcher".into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "Find what the person asks for.".into(),
            action_names: vec!["list_objects".into()],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(admin),
    )
    .unwrap();
    ai_agent_service::set_model_routing(conn, &created.id, ws, Some(routing), Some(admin)).unwrap()
}

fn routed(conn: &rusqlite::Connection, ws: &str, admin: &str) -> AiAgentModelRouting {
    AiAgentModelRouting {
        primary_provider_id: Some(scripted_provider_row(conn, ws, admin, "primary")),
        ..Default::default()
    }
}

fn usage(tokens: i64) -> TokenUsage {
    TokenUsage {
        input_tokens: tokens / 2,
        output_tokens: tokens - tokens / 2,
    }
}

/// A round that asks for one tool call; `n` varies the arguments so the
/// repeated-call guard never stops the run first.
fn tool_round(n: usize, tokens: i64) -> ScriptedResponse {
    ScriptedResponse::ToolCalls {
        calls: vec![RequestedToolCall {
            id: format!("call-{n}"),
            name: "list_objects".into(),
            arguments: json!({"page": n}),
            malformed: None,
        }],
        usage: usage(tokens),
    }
}

fn text(t: &str) -> ScriptedResponse {
    ScriptedResponse::Text {
        text: t.into(),
        usage: usage(10),
    }
}

async fn run(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    agent: &AiAgentDefinition,
    script: Vec<ScriptedResponse>,
) -> (Result<AgentRunOutcome, AppError>, Vec<CapturedRequest>) {
    scripted_provider::reset();
    scripted_provider::script_responses(script);
    let result = chat_service::run_agent_once_with_text(
        conn,
        ws,
        &KEY,
        Some(admin),
        agent,
        "Find everything",
        &AgentMemoryContext {
            session_key: None,
            run_id: None,
        },
    )
    .await;
    (result, scripted_provider::take_captured())
}

/// The final-answer request: tools are off, every earlier one had them on.
fn assert_one_final_answer_request(captured: &[CapturedRequest], reason_text: &str) {
    let (last, earlier) = captured.split_last().expect("at least one request");
    assert!(last.options.tools_off, "the final answer has tools off");
    assert!(
        last.system_prompt
            .contains("This run has reached its limit")
            && last.system_prompt.contains(reason_text),
        "{}",
        last.system_prompt
    );
    assert!(
        earlier.iter().all(|c| !c.options.tools_off),
        "only the last request has tools off"
    );
}

#[tokio::test]
async fn ac1_the_round_limit_asks_once_for_a_final_answer_and_keeps_it() {
    // Covers: REQ-LOOP-008 (WP-0121 AC-1)
    let (conn, ws, admin) = setup_workspace();
    let a = agent(
        &conn,
        &ws,
        &admin,
        AiAgentModelRouting {
            max_rounds: Some(3),
            ..routed(&conn, &ws, &admin)
        },
    );
    let (result, captured) = run(
        &conn,
        &ws,
        &admin,
        &a,
        vec![
            tool_round(1, 10),
            tool_round(2, 10),
            tool_round(3, 10),
            text("Found two of the three objects; the third needs a person."),
        ],
    )
    .await;
    let outcome = result.unwrap();
    assert_eq!(outcome.stop_reason, Some(chat_service::STOP_LIMIT_ROUNDS));
    assert!(outcome.is_partial());
    assert_eq!(
        outcome.final_text,
        "Found two of the three objects; the third needs a person."
    );
    // The work so far is kept, not thrown away: three tool rounds, then
    // the answer.
    assert_eq!(
        outcome.produced.iter().filter(|m| m.role == "tool").count(),
        3
    );
    assert_eq!(outcome.produced.last().unwrap().role, "assistant");
    assert_eq!(captured.len(), 4, "three rounds, then one final answer");
    assert_one_final_answer_request(&captured, "all 3 of its tool-calling rounds");
}

#[tokio::test]
async fn ac1_without_a_setting_the_round_limit_is_eight() {
    // Covers: REQ-LOOP-008 (WP-0121 AC-1)
    let (conn, ws, admin) = setup_workspace();
    let a = agent(&conn, &ws, &admin, routed(&conn, &ws, &admin));
    let mut script: Vec<ScriptedResponse> = (1..=8).map(|n| tool_round(n, 10)).collect();
    script.push(text("partial"));
    let (result, captured) = run(&conn, &ws, &admin, &a, script).await;
    assert_eq!(
        result.unwrap().stop_reason,
        Some(chat_service::STOP_LIMIT_ROUNDS)
    );
    assert_eq!(captured.len(), 9);
    assert_one_final_answer_request(&captured, "all 8 of its tool-calling rounds");
}

#[tokio::test]
async fn ac1_the_token_limit_asks_once_for_a_final_answer_and_keeps_it() {
    // Covers: REQ-LOOP-009 (WP-0121 AC-1)
    let (conn, ws, admin) = setup_workspace();
    let a = agent(
        &conn,
        &ws,
        &admin,
        AiAgentModelRouting {
            run_token_limit: Some(1000),
            ..routed(&conn, &ws, &admin)
        },
    );
    // 600 tokens a round: under the limit after one round, over it after two.
    let (result, captured) = run(
        &conn,
        &ws,
        &admin,
        &a,
        vec![
            tool_round(1, 600),
            tool_round(2, 600),
            text("What I have so far."),
        ],
    )
    .await;
    let outcome = result.unwrap();
    assert_eq!(outcome.stop_reason, Some(chat_service::STOP_LIMIT_TOKENS));
    assert_eq!(outcome.final_text, "What I have so far.");
    assert_eq!(captured.len(), 3, "two rounds, then one final answer");
    assert_one_final_answer_request(&captured, "its 1000-token limit");
}

#[tokio::test]
async fn ac1_a_run_that_finishes_inside_its_limits_is_complete() {
    // Covers: REQ-LOOP-008 (WP-0121 AC-1)
    let (conn, ws, admin) = setup_workspace();
    let a = agent(
        &conn,
        &ws,
        &admin,
        AiAgentModelRouting {
            max_rounds: Some(3),
            run_token_limit: Some(5000),
            ..routed(&conn, &ws, &admin)
        },
    );
    let (result, captured) = run(
        &conn,
        &ws,
        &admin,
        &a,
        vec![tool_round(1, 100), text("All done.")],
    )
    .await;
    let outcome = result.unwrap();
    assert_eq!(outcome.stop_reason, None);
    assert!(!outcome.is_partial());
    assert_eq!(outcome.final_text, "All done.");
    assert!(captured.iter().all(|c| !c.options.tools_off));
}

#[tokio::test]
async fn ac1_a_final_answer_that_still_asks_for_a_tool_runs_nothing() {
    // Covers: REQ-LOOP-008 (WP-0121 AC-1, AC-2): tools are off, so a model
    // that asks for one anyway gets nothing run and the run still ends.
    let (conn, ws, admin) = setup_workspace();
    let a = agent(
        &conn,
        &ws,
        &admin,
        AiAgentModelRouting {
            max_rounds: Some(1),
            ..routed(&conn, &ws, &admin)
        },
    );
    let (result, captured) = run(
        &conn,
        &ws,
        &admin,
        &a,
        vec![tool_round(1, 10), tool_round(2, 10)],
    )
    .await;
    let outcome = result.unwrap();
    assert_eq!(outcome.stop_reason, Some(chat_service::STOP_LIMIT_ROUNDS));
    assert_eq!(outcome.final_text, "");
    assert_eq!(
        outcome.produced.iter().filter(|m| m.role == "tool").count(),
        1,
        "the tool asked for in the final answer did not run"
    );
    assert_eq!(captured.len(), 2);
}

#[tokio::test(start_paused = true)]
async fn ac1_a_call_past_the_time_limit_is_cancelled_and_tried_once_on_the_fallback() {
    // Covers: REQ-LOOP-010 (WP-0121 AC-1)
    let (conn, ws, admin) = setup_workspace();
    let a = agent(
        &conn,
        &ws,
        &admin,
        AiAgentModelRouting {
            fallback_provider_id: Some(scripted_provider_row(&conn, &ws, &admin, "fallback")),
            step_timeout_seconds: Some(30),
            ..routed(&conn, &ws, &admin)
        },
    );
    let (result, captured) = run(
        &conn,
        &ws,
        &admin,
        &a,
        vec![ScriptedResponse::Stall, text("Answered by the fallback.")],
    )
    .await;
    let outcome = result.unwrap();
    assert_eq!(outcome.final_text, "Answered by the fallback.");
    assert_eq!(outcome.stop_reason, None);
    assert_eq!(
        captured
            .iter()
            .map(|c| c.model.as_str())
            .collect::<Vec<_>>(),
        ["primary-model", "fallback-model"]
    );
    let events =
        lanesra_core::services::ai_gateway_service::recent_failover_events(&conn, &ws, 10).unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.served_by == "fallback" && e.reason.contains("30 seconds")),
        "{events:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn ac1_a_second_call_past_the_time_limit_ends_the_run_with_a_timeout() {
    // Covers: REQ-LOOP-010 (WP-0121 AC-1, AC-2): once on the fallback, not
    // on every tier.
    let (conn, ws, admin) = setup_workspace();
    let a = agent(
        &conn,
        &ws,
        &admin,
        AiAgentModelRouting {
            fallback_provider_id: Some(scripted_provider_row(&conn, &ws, &admin, "fallback")),
            local_fallback_provider_id: Some(scripted_provider_row(&conn, &ws, &admin, "local")),
            step_timeout_seconds: Some(30),
            ..routed(&conn, &ws, &admin)
        },
    );
    let (result, captured) = run(
        &conn,
        &ws,
        &admin,
        &a,
        vec![
            ScriptedResponse::Stall,
            ScriptedResponse::Stall,
            text("never reached"),
        ],
    )
    .await;
    let err = result.unwrap_err();
    assert!(
        matches!(
            &err,
            AppError::Classified {
                code: "provider.timeout",
                ..
            }
        ),
        "{err:?}"
    );
    assert_eq!(captured.len(), 2, "the local tier is not tried");
}

#[tokio::test(start_paused = true)]
async fn ac1_an_agent_without_routing_is_still_cut_off_at_the_default_time_limit() {
    // Covers: REQ-LOOP-010 (WP-0121 AC-1, AC-2)
    let (conn, ws, admin) = setup_workspace();
    let created = ai_agent_service::create(
        &conn,
        &ws,
        &AiAgentInput {
            name: "Plain".into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "Answer.".into(),
            action_names: vec![],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(&admin),
    )
    .unwrap();
    use_scripted_workspace_default(&conn, &ws, &admin);
    let started = tokio::time::Instant::now();
    let (result, _) = run(&conn, &ws, &admin, &created, vec![ScriptedResponse::Stall]).await;
    assert!(matches!(
        result,
        Err(AppError::Classified {
            code: "provider.timeout",
            ..
        })
    ));
    assert_eq!(
        started.elapsed().as_secs(),
        300,
        "the default is five minutes"
    );
}

#[tokio::test]
async fn the_agents_temperature_and_output_limit_reach_every_call() {
    // Covers: REQ-LOOP-001 (WP-0121): routing temperature and max tokens
    // take effect (gap audit, T-03).
    let (conn, ws, admin) = setup_workspace();
    let a = agent(
        &conn,
        &ws,
        &admin,
        AiAgentModelRouting {
            temperature: Some(0.2),
            max_tokens: Some(512),
            max_rounds: Some(1),
            ..routed(&conn, &ws, &admin)
        },
    );
    let (_, captured) = run(
        &conn,
        &ws,
        &admin,
        &a,
        vec![tool_round(1, 10), text("done")],
    )
    .await;
    assert_eq!(captured.len(), 2);
    for c in &captured {
        assert_eq!(c.options.temperature, Some(0.2));
        assert_eq!(c.options.max_tokens, Some(512));
    }
}

#[test]
fn limits_outside_their_bounds_are_refused() {
    // Covers: REQ-LOOP-008 (WP-0121): 1 to 25 rounds, at least 1,000 tokens,
    // 5 to 1,800 seconds a call.
    let (conn, ws, admin) = setup_workspace();
    let created = ai_agent_service::create(
        &conn,
        &ws,
        &AiAgentInput {
            name: "Bounds".into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "Answer.".into(),
            action_names: vec![],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(&admin),
    )
    .unwrap();
    let set = |routing: AiAgentModelRouting| {
        ai_agent_service::set_model_routing(&conn, &created.id, &ws, Some(routing), Some(&admin))
    };
    for bad in [
        AiAgentModelRouting {
            max_rounds: Some(26),
            ..Default::default()
        },
        AiAgentModelRouting {
            max_rounds: Some(0),
            ..Default::default()
        },
        AiAgentModelRouting {
            run_token_limit: Some(999),
            ..Default::default()
        },
        AiAgentModelRouting {
            step_timeout_seconds: Some(4),
            ..Default::default()
        },
        AiAgentModelRouting {
            max_tokens: Some(0),
            ..Default::default()
        },
    ] {
        assert!(set(bad.clone()).is_err(), "{bad:?} was accepted");
    }
    let saved = set(AiAgentModelRouting {
        max_rounds: Some(25),
        run_token_limit: Some(20_000),
        step_timeout_seconds: Some(120),
        ..Default::default()
    })
    .unwrap();
    let routing = saved.model_routing.unwrap();
    assert_eq!(routing.max_rounds, Some(25));
    assert_eq!(routing.run_token_limit, Some(20_000));
    assert_eq!(routing.step_timeout_seconds, Some(120));
}

fn use_scripted_workspace_default(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    ai_service::save_settings(
        conn,
        ws,
        &KEY,
        &AiSettingsInput {
            provider: "anthropic".into(),
            base_url: None,
            model: "default-model".into(),
            api_key: Some("k".into()),
        },
        Some(admin),
    )
    .unwrap();
    conn.execute(
        "UPDATE ai_settings SET provider = 'scripted' WHERE workspace_id = ?1",
        [ws],
    )
    .unwrap();
}

#[tokio::test]
async fn ac1_the_assistant_chat_keeps_a_final_answer_at_its_round_limit() {
    // Covers: REQ-LOOP-008 (WP-0121 AC-1): the records assistant used to
    // end with an error and lose the work at its round limit.
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    scripted_provider::reset();
    let mut script: Vec<ScriptedResponse> = (1..=8).map(|n| tool_round(n, 10)).collect();
    script.push(text("Here is what I found before stopping."));
    scripted_provider::script_responses(script);
    let appended =
        chat_service::send_message(&conn, &ws, &KEY, &admin, "records", "List everything")
            .await
            .unwrap();
    let last = appended.last().unwrap();
    assert_eq!(last.role, "assistant");
    assert_eq!(
        last.content.as_deref(),
        Some("Here is what I found before stopping.")
    );
    let captured = scripted_provider::take_captured();
    assert_eq!(captured.len(), 9);
    assert_one_final_answer_request(&captured, "all 8 of its tool-calling rounds");
    // No message the person did not write is added to their conversation.
    assert_eq!(appended.iter().filter(|m| m.role == "user").count(), 1);
}

fn node(key: &str, node_type: &str, config: serde_json::Value) -> GraphNodeInput {
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
async fn ac1_a_graph_run_whose_agent_hits_a_limit_ends_completed_partial() {
    // Covers: REQ-LOOP-008 (WP-0121 AC-1): ADR-0007 `completed_partial`,
    // with the limit as the stop reason, on the step and on the run.
    let (conn, ws, admin) = setup_workspace();
    let a = agent(
        &conn,
        &ws,
        &admin,
        AiAgentModelRouting {
            max_rounds: Some(1),
            ..routed(&conn, &ws, &admin)
        },
    );
    let graph = execution_graph_service::create(
        &conn,
        &ws,
        &ExecutionGraphInput {
            name: "Limited".into(),
            description: None,
            nodes: vec![
                node("trigger", "trigger", json!({})),
                node(
                    "research",
                    "agent",
                    json!({"agent_id": a.id, "input_template": "{{trigger_input}}"}),
                ),
                node("end", "end", json!({})),
            ],
            edges: vec![edge("trigger", "research"), edge("research", "end")],
        },
        Some(&admin),
    )
    .unwrap();
    let graph = execution_graph_service::publish(&conn, &graph.id, &ws, Some(&admin)).unwrap();
    scripted_provider::reset();
    scripted_provider::script_responses(vec![tool_round(1, 10), text("Partly done.")]);
    let run = graph_runtime_service::start_run(
        &conn,
        &ws,
        &KEY,
        &graph.id,
        "Find everything",
        Some(&admin),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(run.status, "completed_partial", "{run:?}");
    assert_eq!(run.stop_reason.as_deref(), Some("limit_rounds"));
    let step = run.nodes.iter().find(|n| n.node_key == "research").unwrap();
    assert_eq!(step.status, "completed_partial");
    assert!(step
        .output_json
        .as_deref()
        .unwrap_or_default()
        .contains("Partly done."));
    assert!(
        !run.context_json.contains("__partial_stop_reason"),
        "the marker is not left in the run's context"
    );
}
