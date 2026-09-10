//! AI & Agentic Layer, Phase 7a: the Unified AI Gateway -
//! `ai_gateway_service::dispatch`'s routing/failover/budget/air-gap
//! sequence, the new `google_gemini` provider adapter, named `ai_providers`
//! CRUD, and `ai_agent_service::set_model_routing`'s validation. Every
//! provider call goes through a real local HTTP stub (not a live
//! third-party endpoint), the same convention `ai_settings.rs`/
//! `ai_agent_orchestration.rs` already establish in this crate.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use lanesra_core::domain::ids::new_uuid;
use lanesra_core::models::ai::{AiAgentModelRouting, AiDailyTokenBudgetInput, AiProviderInput, AiSettingsInput};
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::chat::ChatMessage;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_gateway_service, ai_provider_service, ai_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Gateway Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = lanesra_core::services::workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [53u8; 32]
}

/// A minimal raw-socket HTTP server returning a fixed status/body for
/// every request - the same test double `ai_settings.rs`/
/// `ai_agent_orchestration.rs` already use.
fn spawn_http_stub(status: u16, body: String) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            let _ = reader.read_line(&mut line);
            loop {
                let mut l = String::new();
                match reader.read_line(&mut l) {
                    Ok(0) | Ok(_) if l == "\r\n" || l.is_empty() => break,
                    _ => continue,
                }
            }
            let reason = if status == 200 { "OK" } else { "Server Error" };
            let response = format!(
                "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn anthropic_text_body(text: &str) -> String {
    serde_json::json!({"content": [{"type": "text", "text": text}], "usage": {"input_tokens": 11, "output_tokens": 3}}).to_string()
}

fn gemini_text_body(text: &str) -> String {
    serde_json::json!({
        "candidates": [{"content": {"parts": [{"text": text}]}}],
        "usageMetadata": {"promptTokenCount": 7, "candidatesTokenCount": 2},
    })
    .to_string()
}

fn configure_workspace_anthropic(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap();
}

fn make_provider(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, name: &str, port: u16) -> String {
    ai_provider_service::create(
        conn, workspace_id, &master_key(),
        &AiProviderInput { name: name.into(), provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-provider".into()) },
        Some(admin),
    )
    .unwrap()
    .id
}

fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> lanesra_core::models::ai_agent::AiAgentDefinition {
    ai_agent_service::create(
        conn, ws,
        &AiAgentInput { name: name.into(), description: None, icon: "🤖".into(), system_prompt: format!("You are {name}."), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(admin),
    )
    .unwrap()
}

fn user_turn(text: &str) -> ChatMessage {
    ChatMessage { id: String::new(), conversation_id: String::new(), role: "user".into(), content: Some(text.into()), tool_calls: None, tool_call_id: None, created_at: String::new() }
}

// --- google_gemini provider adapter ----------------------------------

#[tokio::test]
async fn gemini_test_key_hits_the_models_endpoint() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_http_stub(200, r#"{"models":[{"name":"models/gemini-1.5-pro"}]}"#.to_string());
    let input = AiSettingsInput { provider: "google_gemini".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "gemini-1.5-pro".into(), api_key: Some("gm-key".into()) };
    ai_service::save_settings(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).unwrap();
    let result = ai_service::test_key(&conn, &workspace_id, &master_key(), Some(&admin_id)).await.unwrap();
    assert!(result.ok, "{result:?}");
}

#[tokio::test]
async fn gemini_complete_parses_the_real_response_shape() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_http_stub(200, gemini_text_body("hello from gemini"));
    let input = AiSettingsInput { provider: "google_gemini".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "gemini-1.5-pro".into(), api_key: Some("gm-key".into()) };
    ai_service::save_settings(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).unwrap();
    let reply = ai_service::complete(&conn, &workspace_id, &master_key(), "system prompt", "hi").await.unwrap();
    assert_eq!(reply, "hello from gemini");
}

// --- ai_provider_service CRUD ------------------------------------------

#[test]
fn provider_create_requires_admin_and_validates_the_provider_name() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let input = AiProviderInput { name: "Ollama".into(), provider: "anthropic".into(), base_url: None, model: "".into(), api_key: None };
    assert!(ai_provider_service::create(&conn, &workspace_id, &master_key(), &input, None).is_err(), "non-admin should be rejected");

    let bad = AiProviderInput { name: "Ollama".into(), provider: "made_up".into(), base_url: None, model: "".into(), api_key: None };
    assert!(ai_provider_service::create(&conn, &workspace_id, &master_key(), &bad, Some(&admin_id)).is_err());

    let good = ai_provider_service::create(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).unwrap();
    assert_eq!(good.name, "Ollama");
    assert!(!good.has_key);
}

#[tokio::test]
async fn provider_test_key_makes_a_real_call_against_its_own_stub() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_http_stub(200, r#"{"data":[{"id":"gpt-4"}]}"#.to_string());
    let input = AiProviderInput { name: "Local".into(), provider: "openai_compatible".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "gpt-4".into(), api_key: Some("sk-local".into()) };
    let provider = ai_provider_service::create(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).unwrap();
    let result = ai_provider_service::test_key(&conn, &provider.id, &master_key(), Some(&admin_id)).await.unwrap();
    assert!(result.ok, "{result:?}");
}

// --- ai_agent_service::set_model_routing validation ---------------------

#[test]
fn set_model_routing_rejects_an_unknown_provider_id() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let agent = make_agent(&conn, &workspace_id, &admin_id, "Router");
    let routing = AiAgentModelRouting { primary_provider_id: Some(new_uuid()), ..Default::default() };
    assert!(ai_agent_service::set_model_routing(&conn, &agent.id, &workspace_id, Some(routing), Some(&admin_id)).is_err());
}

#[test]
fn set_model_routing_rejects_an_unrecognized_dlp_class() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let agent = make_agent(&conn, &workspace_id, &admin_id, "Router");
    let routing = AiAgentModelRouting { force_air_gapped_for: vec!["not_a_real_class".into()], ..Default::default() };
    assert!(ai_agent_service::set_model_routing(&conn, &agent.id, &workspace_id, Some(routing), Some(&admin_id)).is_err());
}

#[test]
fn set_model_routing_none_clears_a_previously_saved_policy() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let agent = make_agent(&conn, &workspace_id, &admin_id, "Router");
    let routing = AiAgentModelRouting { temperature: Some(0.5), ..Default::default() };
    let saved = ai_agent_service::set_model_routing(&conn, &agent.id, &workspace_id, Some(routing), Some(&admin_id)).unwrap();
    assert!(saved.model_routing.is_some());
    let cleared = ai_agent_service::set_model_routing(&conn, &agent.id, &workspace_id, None, Some(&admin_id)).unwrap();
    assert!(cleared.model_routing.is_none());
}

// --- ai_gateway_service::dispatch ---------------------------------------

#[tokio::test]
async fn no_routing_policy_dispatches_through_the_plain_workspace_default() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_http_stub(200, anthropic_text_body("plain default reply"));
    configure_workspace_anthropic(&conn, &workspace_id, &admin_id, port);
    let agent = make_agent(&conn, &workspace_id, &admin_id, "Plain");

    let outcome = ai_gateway_service::dispatch(&conn, &workspace_id, &master_key(), &agent, Some(&admin_id), "system", &[], &[user_turn("hi")]).await.unwrap();
    assert_eq!(outcome.served_by, "primary");
    match outcome.outcome {
        lanesra_core::services::ai_service::CompletionOutcome::Text(t) => assert_eq!(t, "plain default reply"),
        other => panic!("expected a text outcome, got {other:?}"),
    }
    assert!(ai_gateway_service::recent_failover_events(&conn, &workspace_id, 10).unwrap().is_empty(), "a clean primary success should log no failover event");
}

#[tokio::test]
async fn a_failed_primary_tier_falls_back_and_records_a_failover_event() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let agent = make_agent(&conn, &workspace_id, &admin_id, "Failover");

    let bad_port = spawn_http_stub(500, r#"{"error":"boom"}"#.to_string());
    let good_port = spawn_http_stub(200, anthropic_text_body("fallback saved the day"));
    let primary_id = make_provider(&conn, &workspace_id, &admin_id, "Bad primary", bad_port);
    let fallback_id = make_provider(&conn, &workspace_id, &admin_id, "Good fallback", good_port);
    let routing = AiAgentModelRouting { primary_provider_id: Some(primary_id), fallback_provider_id: Some(fallback_id), ..Default::default() };
    ai_agent_service::set_model_routing(&conn, &agent.id, &workspace_id, Some(routing), Some(&admin_id)).unwrap();
    let agent = ai_agent_service::get(&conn, &agent.id).unwrap().unwrap();

    let outcome = ai_gateway_service::dispatch(&conn, &workspace_id, &master_key(), &agent, Some(&admin_id), "system", &[], &[user_turn("hi")]).await.unwrap();
    assert_eq!(outcome.served_by, "fallback");

    let events = ai_gateway_service::recent_failover_events(&conn, &workspace_id, 10).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].served_by, "fallback");
    assert_eq!(events[0].agent_id, agent.id);
}

#[tokio::test]
async fn every_tier_failing_surfaces_the_last_real_error() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let agent = make_agent(&conn, &workspace_id, &admin_id, "AllDown");
    let bad_port = spawn_http_stub(500, r#"{"error":"boom"}"#.to_string());
    let primary_id = make_provider(&conn, &workspace_id, &admin_id, "Down 1", bad_port);
    let routing = AiAgentModelRouting { primary_provider_id: Some(primary_id), ..Default::default() };
    ai_agent_service::set_model_routing(&conn, &agent.id, &workspace_id, Some(routing), Some(&admin_id)).unwrap();
    let agent = ai_agent_service::get(&conn, &agent.id).unwrap().unwrap();

    // No workspace ai_settings key configured either, so the fallback/
    // local_fallback tiers (both `None` -> workspace default) fail too.
    let result = ai_gateway_service::dispatch(&conn, &workspace_id, &master_key(), &agent, Some(&admin_id), "system", &[], &[user_turn("hi")]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn forced_air_gap_routes_straight_to_local_fallback_skipping_primary() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let agent = make_agent(&conn, &workspace_id, &admin_id, "AirGapped");

    // The primary points at an address nothing is listening on - if the
    // gateway ever tried it, this test would fail on a connection error
    // instead of a clean success, proving forced air-gapping genuinely
    // skips it rather than merely reordering after a failed attempt.
    let unreachable_port = 1u16;
    let local_port = spawn_http_stub(200, anthropic_text_body("air-gapped reply"));
    let primary_id = make_provider(&conn, &workspace_id, &admin_id, "Cloud primary", unreachable_port);
    let local_id = make_provider(&conn, &workspace_id, &admin_id, "Local model", local_port);
    let routing = AiAgentModelRouting { primary_provider_id: Some(primary_id), local_fallback_provider_id: Some(local_id), force_air_gapped_for: vec!["email".into()], ..Default::default() };
    ai_agent_service::set_model_routing(&conn, &agent.id, &workspace_id, Some(routing), Some(&admin_id)).unwrap();
    let agent = ai_agent_service::get(&conn, &agent.id).unwrap().unwrap();

    let history = vec![user_turn("Please reconcile the account for jane.doe@example.com")];
    let outcome = ai_gateway_service::dispatch(&conn, &workspace_id, &master_key(), &agent, Some(&admin_id), "system", &[], &history).await.unwrap();
    assert_eq!(outcome.served_by, "local_fallback");

    let events = ai_gateway_service::recent_failover_events(&conn, &workspace_id, 10).unwrap();
    assert_eq!(events.len(), 1);
    assert!(events[0].reason.contains("air-gapped"), "{}", events[0].reason);
}

#[tokio::test]
async fn a_payload_with_no_sensitive_data_does_not_trigger_air_gapping() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let agent = make_agent(&conn, &workspace_id, &admin_id, "NotSensitive");
    let primary_port = spawn_http_stub(200, anthropic_text_body("ordinary reply"));
    let primary_id = make_provider(&conn, &workspace_id, &admin_id, "Cloud primary", primary_port);
    let routing = AiAgentModelRouting { primary_provider_id: Some(primary_id), force_air_gapped_for: vec!["email".into()], ..Default::default() };
    ai_agent_service::set_model_routing(&conn, &agent.id, &workspace_id, Some(routing), Some(&admin_id)).unwrap();
    let agent = ai_agent_service::get(&conn, &agent.id).unwrap().unwrap();

    let outcome = ai_gateway_service::dispatch(&conn, &workspace_id, &master_key(), &agent, Some(&admin_id), "system", &[], &[user_turn("What's our Q3 pipeline look like?")]).await.unwrap();
    assert_eq!(outcome.served_by, "primary");
}

#[tokio::test]
async fn the_system_token_budget_blocks_dispatch_once_reached() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_http_stub(200, anthropic_text_body("should never be reached"));
    configure_workspace_anthropic(&conn, &workspace_id, &admin_id, port);
    ai_service::set_daily_token_budget(&conn, &workspace_id, &AiDailyTokenBudgetInput { daily_token_budget: Some(5) }, Some(&admin_id)).unwrap();
    let agent = make_agent(&conn, &workspace_id, &admin_id, "Budgeted");

    // Pre-seed today's usage past the ceiling directly through the same
    // repo `dispatch` itself reads from - simulating an earlier real run
    // without needing a second live dispatch just to build up spend.
    lanesra_core::repositories::ai_token_usage_repo::increment(&conn, &workspace_id, "", "", 10, 0).unwrap();

    let result = ai_gateway_service::dispatch(&conn, &workspace_id, &master_key(), &agent, Some(&admin_id), "system", &[], &[user_turn("hi")]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn an_agents_own_budget_is_enforced_independently_of_the_system_one() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_http_stub(200, anthropic_text_body("should never be reached"));
    let agent = make_agent(&conn, &workspace_id, &admin_id, "TightBudget");
    let provider_id = make_provider(&conn, &workspace_id, &admin_id, "Some provider", port);
    let routing = AiAgentModelRouting { primary_provider_id: Some(provider_id), daily_token_budget: Some(5), ..Default::default() };
    ai_agent_service::set_model_routing(&conn, &agent.id, &workspace_id, Some(routing), Some(&admin_id)).unwrap();
    let agent = ai_agent_service::get(&conn, &agent.id).unwrap().unwrap();

    // No system-level budget configured at all - proving this is a
    // genuinely separate ceiling, not a proxy for the System tier.
    lanesra_core::repositories::ai_token_usage_repo::increment(&conn, &workspace_id, &agent.id, "", 10, 0).unwrap();

    let result = ai_gateway_service::dispatch(&conn, &workspace_id, &master_key(), &agent, Some(&admin_id), "system", &[], &[user_turn("hi")]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn a_successful_dispatch_records_real_token_usage() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_http_stub(200, anthropic_text_body("counted"));
    configure_workspace_anthropic(&conn, &workspace_id, &admin_id, port);
    let agent = make_agent(&conn, &workspace_id, &admin_id, "Counted");

    ai_gateway_service::dispatch(&conn, &workspace_id, &master_key(), &agent, Some(&admin_id), "system", &[], &[user_turn("hi")]).await.unwrap();
    let usage = ai_agent_service::token_usage_today(&conn, &agent.id, &workspace_id).unwrap();
    assert_eq!(usage.today_input_tokens, 11);
    assert_eq!(usage.today_output_tokens, 3);
}
