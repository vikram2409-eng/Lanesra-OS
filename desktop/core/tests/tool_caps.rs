//! WP-0140: per-tool caps and rate limits (REQ-LOOP-014, REQ-TOOL-008).
//! A binding's calls-per-step cap refuses the extra call with a classified
//! error; its failures-before-disable cap disables it; its result-size cap
//! stores the result and passes a reference; a rate limit refuses a call
//! before it reaches the tool. Every enforcement shows in the run's trace.
//!
//! Runs go through the scripted provider (WP-0005). Side effects are
//! checked on real records, so "not run" means nothing was created.

use lanesra_core::domain::AppError;
use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput, AiSettingsInput};
use lanesra_core::models::ai_agent::{AiAgentDefinition, AiAgentInput};
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::ai_memory::AgentMemoryContext;
use lanesra_core::models::tool_limits::{AgentToolCapInput, ToolRateLimitInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ai_service::{RequestedToolCall, TokenUsage};
use lanesra_core::services::chat_service::{self, AgentRunOutcome};
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    ai_agent_service, ai_orchestration_service, ai_provider_service, ai_service, company_service,
    tool_limits_service, trace_service, workspace_service,
};
use serde_json::{json, Value};

const KEY: [u8; 32] = [140u8; 32];

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Caps Co".into(),
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

fn scripted_agent(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    name: &str,
) -> AiAgentDefinition {
    let provider = ai_provider_service::create(
        conn,
        ws,
        &KEY,
        &AiProviderInput {
            name: format!("{name} provider"),
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
    let created = ai_agent_service::create(
        conn,
        ws,
        &AiAgentInput {
            name: name.into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "Do what is asked.".into(),
            action_names: vec![
                "list_objects".into(),
                "get_record".into(),
                "create_record".into(),
            ],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(admin),
    )
    .unwrap();
    ai_agent_service::set_model_routing(
        conn,
        &created.id,
        ws,
        Some(AiAgentModelRouting {
            primary_provider_id: Some(provider.id),
            ..Default::default()
        }),
        Some(admin),
    )
    .unwrap()
}

fn call(n: usize, name: &str, arguments: Value) -> ScriptedResponse {
    ScriptedResponse::ToolCalls {
        calls: vec![RequestedToolCall {
            id: format!("c{n}"),
            name: name.into(),
            arguments,
            malformed: None,
        }],
        usage: TokenUsage {
            input_tokens: 10,
            output_tokens: 5,
        },
    }
}

fn create_company(n: usize, name: &str) -> ScriptedResponse {
    call(
        n,
        "create_record",
        json!({"object_key": "Company", "data": {"name": name, "status": "Prospect"}}),
    )
}

fn text(t: &str) -> ScriptedResponse {
    ScriptedResponse::Text {
        text: t.into(),
        usage: TokenUsage {
            input_tokens: 10,
            output_tokens: 5,
        },
    }
}

fn cap(
    tool: &str,
    calls: Option<i64>,
    failures: Option<i64>,
    chars: Option<i64>,
) -> AgentToolCapInput {
    AgentToolCapInput {
        tool_name: tool.into(),
        max_calls_per_step: calls,
        failures_before_disable: failures,
        max_result_chars: chars,
    }
}

async fn run(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    agent: &AiAgentDefinition,
    script: Vec<ScriptedResponse>,
) -> AgentRunOutcome {
    scripted_provider::reset();
    scripted_provider::script_responses(script);
    chat_service::run_agent_once_with_text(
        conn,
        ws,
        &KEY,
        Some(admin),
        agent,
        "Go",
        &AgentMemoryContext {
            session_key: None,
            run_id: None,
        },
    )
    .await
    .unwrap()
}

fn tool_results(outcome: &AgentRunOutcome) -> Vec<String> {
    outcome
        .produced
        .iter()
        .filter(|m| m.role == "tool")
        .map(|m| m.content.clone().unwrap_or_default())
        .collect()
}

#[tokio::test]
async fn ac1_a_call_over_the_calls_per_step_cap_is_refused_and_not_run() {
    // Covers: REQ-LOOP-014 (WP-0140 AC-1)
    let (conn, ws, admin) = setup_workspace();
    let agent = scripted_agent(&conn, &ws, &admin, "Creator");
    tool_limits_service::set_agent_tool_caps(
        &conn,
        &ws,
        &agent.id,
        &[cap("create_record", Some(2), None, None)],
        Some(&admin),
    )
    .unwrap();
    let outcome = run(
        &conn,
        &ws,
        &admin,
        &agent,
        vec![
            create_company(1, "One"),
            create_company(2, "Two"),
            create_company(3, "Three"),
            text("Done."),
        ],
    )
    .await;
    let results = tool_results(&outcome);
    assert!(
        results[2].contains("policy.tool_cap") || results[2].contains("at most 2 times"),
        "{}",
        results[2]
    );
    assert!(results[2].contains("was not run"));
    assert_eq!(
        company_service::list(&conn, &ws).unwrap().len(),
        2,
        "the third call did not run"
    );

    // The cap is per step: a new run may call it twice again.
    let outcome = run(
        &conn,
        &ws,
        &admin,
        &agent,
        vec![create_company(4, "Four"), text("Done.")],
    )
    .await;
    assert!(!tool_results(&outcome)[0].starts_with("Error"));
    assert_eq!(company_service::list(&conn, &ws).unwrap().len(), 3);
}

#[tokio::test]
async fn ac1_failures_in_a_row_disable_the_binding_until_an_administrator_re_enables_it() {
    // Covers: REQ-LOOP-014 (WP-0140 AC-1)
    let (conn, ws, admin) = setup_workspace();
    let agent = scripted_agent(&conn, &ws, &admin, "Reader");
    tool_limits_service::set_agent_tool_caps(
        &conn,
        &ws,
        &agent.id,
        &[cap("get_record", None, Some(2), None)],
        Some(&admin),
    )
    .unwrap();
    let missing = |n| {
        call(
            n,
            "get_record",
            json!({"object_key": "Company", "id": format!("missing-{n}")}),
        )
    };
    let outcome = run(
        &conn,
        &ws,
        &admin,
        &agent,
        vec![missing(1), missing(2), missing(3), text("Gave up.")],
    )
    .await;
    let results = tool_results(&outcome);
    assert!(results[0].starts_with("Error") && results[1].starts_with("Error"));
    assert!(results[2].contains("disabled"), "{}", results[2]);
    assert!(results[2].contains("was not run"));
    let caps = tool_limits_service::list_agent_tool_caps(&conn, &ws, &agent.id).unwrap();
    assert!(caps[0].disabled_at.is_some());

    // Disabled across runs, until re-enabled.
    let outcome = run(
        &conn,
        &ws,
        &admin,
        &agent,
        vec![missing(4), text("Still off.")],
    )
    .await;
    assert!(tool_results(&outcome)[0].contains("disabled"));
    let caps =
        tool_limits_service::reenable_agent_tool(&conn, &ws, &agent.id, "get_record", Some(&admin))
            .unwrap();
    assert!(caps[0].disabled_at.is_none());
    assert_eq!(caps[0].consecutive_failures, 0);
    let outcome = run(&conn, &ws, &admin, &agent, vec![missing(5), text("Ran.")]).await;
    assert!(!tool_results(&outcome)[0].contains("disabled"));
}

#[tokio::test]
async fn ac1_a_result_over_the_size_cap_is_stored_and_referenced() {
    // Covers: REQ-LOOP-014 (WP-0140 AC-1)
    let (conn, ws, admin) = setup_workspace();
    let agent = scripted_agent(&conn, &ws, &admin, "Lister");
    tool_limits_service::set_agent_tool_caps(
        &conn,
        &ws,
        &agent.id,
        &[cap("list_objects", None, None, Some(100))],
        Some(&admin),
    )
    .unwrap();
    let outcome = run(
        &conn,
        &ws,
        &admin,
        &agent,
        vec![call(1, "list_objects", json!({})), text("Listed.")],
    )
    .await;
    let result = &tool_results(&outcome)[0];
    let id = result
        .split("tool_result:")
        .nth(1)
        .and_then(|rest| rest.split('.').next())
        .unwrap_or_else(|| panic!("no reference in {result}"));
    let stored = tool_limits_service::get_stored_result(&conn, &ws, id, Some(&admin)).unwrap();
    assert!(stored.body.chars().count() > 100);
    // The model saw the start and the reference, not the whole result.
    let preview = result.split_once(":\n").unwrap().1;
    assert_eq!(preview.chars().count(), 100);
    assert!(stored.body.starts_with(preview));
    let captured = scripted_provider::take_captured();
    let sent = captured[1].history.last().unwrap().content.clone().unwrap();
    assert!(sent.contains("tool_result:") && !sent.contains(&stored.body));
}

#[tokio::test]
async fn ac2_a_rate_limited_call_never_reaches_the_tool() {
    // Covers: REQ-TOOL-008 (WP-0140 AC-2)
    let (conn, ws, admin) = setup_workspace();
    let agent = scripted_agent(&conn, &ws, &admin, "Limited");
    tool_limits_service::save_rate_limit(
        &conn,
        &ws,
        &ToolRateLimitInput {
            scope: "tool".into(),
            scope_key: "create_record".into(),
            max_calls: 1,
            per_seconds: 3600,
        },
        Some(&admin),
    )
    .unwrap();
    let outcome = run(
        &conn,
        &ws,
        &admin,
        &agent,
        vec![
            create_company(1, "First"),
            create_company(2, "Second"),
            text("Done."),
        ],
    )
    .await;
    let results = tool_results(&outcome);
    assert!(results[1].contains("Rate limit reached"), "{}", results[1]);
    assert_eq!(
        company_service::list(&conn, &ws).unwrap().len(),
        1,
        "the limited call never ran"
    );
}

#[tokio::test]
async fn ac2_the_assistant_chat_is_rate_limited_too() {
    // Covers: REQ-TOOL-008 (WP-0140 AC-2): the limit is in the firewall,
    // so it holds outside agent runs.
    let (conn, ws, admin) = setup_workspace();
    ai_service::save_settings(
        &conn,
        &ws,
        &KEY,
        &AiSettingsInput {
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
    tool_limits_service::save_rate_limit(
        &conn,
        &ws,
        &ToolRateLimitInput {
            scope: "workspace".into(),
            scope_key: String::new(),
            max_calls: 1,
            per_seconds: 3600,
        },
        Some(&admin),
    )
    .unwrap();
    scripted_provider::reset();
    scripted_provider::script_responses(vec![
        create_company(1, "First"),
        create_company(2, "Second"),
        text("Done."),
    ]);
    let appended = chat_service::send_message(&conn, &ws, &KEY, &admin, "records", "make two")
        .await
        .unwrap();
    let tools: Vec<String> = appended
        .iter()
        .filter(|m| m.role == "tool")
        .map(|m| m.content.clone().unwrap_or_default())
        .collect();
    assert!(tools[1].contains("Rate limit reached"), "{}", tools[1]);
    assert_eq!(company_service::list(&conn, &ws).unwrap().len(), 1);
}

#[test]
fn rate_limits_match_their_scope_and_free_up_when_the_window_passes() {
    // Covers: REQ-TOOL-008 (WP-0140 AC-2)
    let (conn, ws, admin) = setup_workspace();
    let save = |scope: &str, key: &str, max: i64, per: i64| {
        tool_limits_service::save_rate_limit(
            &conn,
            &ws,
            &ToolRateLimitInput {
                scope: scope.into(),
                scope_key: key.into(),
                max_calls: max,
                per_seconds: per,
            },
            Some(&admin),
        )
        .unwrap()
    };
    save("connector", "conn-1", 2, 60);
    let t0 = 1_000_000_000_000i64;
    let at =
        |tool: &str, ms: i64| tool_limits_service::check_and_count_call_at(&conn, &ws, tool, ms);
    assert!(at("connector_action:conn-1:getItem", t0).is_ok());
    assert!(at("connector_write_action:conn-1:createItem", t0 + 1).is_ok());
    let refused = at("connector_action:conn-1:getItem", t0 + 2).unwrap_err();
    match refused {
        AppError::Classified {
            code,
            retry_after_ms,
            ..
        } => {
            assert_eq!(code, "policy.rate_limited");
            assert_eq!(retry_after_ms, Some(59_998));
        }
        other => panic!("{other:?}"),
    }
    assert!(
        at("connector_action:conn-2:getItem", t0 + 3).is_ok(),
        "another connector is not limited"
    );
    assert!(at("list_objects", t0 + 4).is_ok());
    assert!(
        at("connector_action:conn-1:getItem", t0 + 60_001).is_ok(),
        "the window has passed"
    );
    // Saving the same scope again changes it rather than adding a second.
    let limits = save("connector", "conn-1", 5, 60);
    assert_eq!(limits.len(), 1);
    assert_eq!(limits[0].max_calls, 5);
}

#[tokio::test]
async fn every_enforcement_is_recorded_in_the_run_trace() {
    // Covers: REQ-LOOP-014, REQ-TOOL-008 (WP-0140 AC-1, ADR-0008)
    let (conn, ws, admin) = setup_workspace();
    let agent = scripted_agent(&conn, &ws, &admin, "Traced");
    tool_limits_service::set_agent_tool_caps(
        &conn,
        &ws,
        &agent.id,
        &[
            cap("create_record", Some(1), None, None),
            cap("list_objects", None, None, Some(100)),
        ],
        Some(&admin),
    )
    .unwrap();
    tool_limits_service::save_rate_limit(
        &conn,
        &ws,
        &ToolRateLimitInput {
            scope: "tool".into(),
            scope_key: "get_record".into(),
            max_calls: 1,
            per_seconds: 3600,
        },
        Some(&admin),
    )
    .unwrap();
    let pipeline = ai_orchestration_service::create_pipeline(
        &conn,
        &ws,
        &AiAgentPipelineInput {
            name: "Traced".into(),
            description: None,
            topology: "sequential".into(),
            steps: vec![PipelineStepInput {
                agent_id: agent.id.clone(),
                input_template: "{{trigger_input}}".into(),
                requires_approval: false,
            }],
        },
        Some(&admin),
    )
    .unwrap();
    scripted_provider::reset();
    scripted_provider::script_responses(vec![
        create_company(1, "One"),
        create_company(2, "Two"),
        call(3, "list_objects", json!({})),
        call(4, "get_record", json!({"object_key": "Company", "id": "x"})),
        call(5, "get_record", json!({"object_key": "Company", "id": "y"})),
        text("Done."),
    ]);
    let run = ai_orchestration_service::run_manual(
        &conn,
        &ws,
        &KEY,
        "pipeline",
        &pipeline.id,
        "Go",
        Some(&admin),
    )
    .await
    .unwrap();
    let spans = trace_service::spans_for_run(&conn, &ws, "pipeline", &run.id).unwrap();
    let tools: Vec<(&str, Option<&Value>, Option<&Value>)> = spans
        .iter()
        .filter(|s| s.kind == "tool_call")
        .map(|s| {
            (
                s.name.as_str(),
                s.attributes.get("lanesra.tool.cap"),
                s.attributes.get("lanesra.tool.outcome"),
            )
        })
        .collect();
    assert_eq!(tools[1].1, Some(&json!("calls_per_step")), "{tools:?}");
    assert_eq!(tools[1].2, Some(&json!("capped")));
    assert_eq!(tools[2].1, Some(&json!("result_size")));
    let rate_limited = spans
        .iter()
        .filter(|s| s.kind == "policy")
        .filter(|s| s.attributes.get("lanesra.policy.verdict") == Some(&json!("rate_limited")))
        .count();
    assert_eq!(rate_limited, 1, "the second get_record was rate limited");
}

#[test]
fn caps_and_limits_are_checked_administrator_only_and_audited() {
    // Covers: REQ-LOOP-014 (WP-0140)
    let (conn, ws, admin) = setup_workspace();
    let agent = scripted_agent(&conn, &ws, &admin, "Checked");
    let set = |caps: &[AgentToolCapInput], actor: Option<&str>| {
        tool_limits_service::set_agent_tool_caps(&conn, &ws, &agent.id, caps, actor)
    };
    assert!(set(
        &[cap("delete_everything", Some(1), None, None)],
        Some(&admin)
    )
    .is_err());
    assert!(set(&[cap("list_objects", Some(0), None, None)], Some(&admin)).is_err());
    assert!(set(&[cap("list_objects", None, None, Some(50))], Some(&admin)).is_err());
    assert!(set(&[cap("list_objects", Some(3), None, None)], None).is_err());
    assert_eq!(
        set(
            &[cap("list_objects", Some(3), Some(2), Some(500))],
            Some(&admin)
        )
        .unwrap()
        .len(),
        1
    );
    // Leaving a tool out removes its caps.
    assert!(set(&[], Some(&admin)).unwrap().is_empty());
    let bad = |scope: &str, max: i64, per: i64| {
        tool_limits_service::save_rate_limit(
            &conn,
            &ws,
            &ToolRateLimitInput {
                scope: scope.into(),
                scope_key: "x".into(),
                max_calls: max,
                per_seconds: per,
            },
            Some(&admin),
        )
        .is_err()
    };
    assert!(bad("model", 1, 60));
    assert!(bad("tool", 0, 60));
    assert!(bad("tool", 1, 0));
    assert!(bad("tool", 1, 86_401));
    let audited: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE event_type = 'ai_agent_tool_caps_changed'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(audited, 2);
}
