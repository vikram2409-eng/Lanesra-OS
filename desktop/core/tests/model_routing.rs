//! WP-0123: per-step model routing and data boundaries. The Gateway must
//! never send a step's air-gapped (sensitive-bound) data to a provider that
//! is not marked local, and it fails closed - with `config.not_local` -
//! when no local provider is available, rather than quietly resolving the
//! workspace default (gap audit I-17). It also honours and records the
//! primary -> fallback -> local order (AC-2). Every provider call goes
//! through a real local HTTP stub, the same convention `ai_gateway.rs`
//! establishes.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput};
use lanesra_core::models::ai_agent::{AiAgentDefinition, AiAgentInput};
use lanesra_core::models::chat::ChatMessage;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_gateway_service, ai_provider_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Routing Test Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) =
        lanesra_core::services::workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [53u8; 32]
}

/// A minimal raw-socket HTTP server returning a fixed status/body for every
/// request - the same test double `ai_gateway.rs` uses.
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
                    Ok(0) => break,
                    Ok(_) if l == "\r\n" || l.is_empty() => break,
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

fn make_provider(
    conn: &rusqlite::Connection,
    workspace_id: &str,
    admin: &str,
    name: &str,
    port: u16,
) -> String {
    ai_provider_service::create(
        conn,
        workspace_id,
        &master_key(),
        &AiProviderInput {
            name: name.into(),
            provider: "anthropic".into(),
            base_url: Some(format!("http://127.0.0.1:{port}")),
            model: "claude-haiku-4-5-20251001".into(),
            api_key: Some("sk-ant-provider".into()),
        },
        Some(admin),
    )
    .unwrap()
    .id
}

/// A provider an administrator has marked local (`is_local = true`) - the
/// only kind forced air-gap routing will dispatch to.
fn make_local_provider(
    conn: &rusqlite::Connection,
    workspace_id: &str,
    admin: &str,
    name: &str,
    port: u16,
) -> String {
    let id = make_provider(conn, workspace_id, admin, name, port);
    ai_provider_service::set_local(conn, &id, true, Some(admin)).unwrap();
    id
}

fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> AiAgentDefinition {
    ai_agent_service::create(
        conn,
        ws,
        &AiAgentInput {
            name: name.into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: format!("You are {name}."),
            action_names: vec![],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(admin),
    )
    .unwrap()
}

fn set_routing(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    agent: &AiAgentDefinition,
    routing: AiAgentModelRouting,
) -> AiAgentDefinition {
    ai_agent_service::set_model_routing(conn, &agent.id, ws, Some(routing), Some(admin)).unwrap();
    ai_agent_service::get(conn, &agent.id).unwrap().unwrap()
}

fn user_turn(text: &str) -> ChatMessage {
    ChatMessage {
        id: String::new(),
        conversation_id: String::new(),
        role: "user".into(),
        content: Some(text.into()),
        tool_calls: None,
        tool_call_id: None,
        created_at: String::new(),
    }
}

/// A turn carrying an SSN, which `dlp_service::scan` flags as the "ssn"
/// class - the air-gap trigger these tests share.
fn sensitive_turn() -> Vec<ChatMessage> {
    vec![user_turn(
        "Reconcile the account for SSN 123-45-6789 right away",
    )]
}

// AC-1: a step bound to sensitive data is routed to the marked-local
// provider, never the cloud primary.
// Covers: REQ-LOOP-003, REQ-GOV-008 (WP-0123)
#[tokio::test]
async fn air_gapped_data_is_routed_to_a_local_provider_not_the_cloud_primary() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Reconciler");
    // Primary points at a port nothing listens on: if the gateway ever tried
    // it the test would fail on a connection error, proving the sensitive
    // payload really went to the local provider instead.
    let unreachable = 1u16;
    let local_port = spawn_http_stub(200, anthropic_text_body("handled locally"));
    let primary = make_provider(&conn, &ws, &admin, "Cloud primary", unreachable);
    let local = make_local_provider(&conn, &ws, &admin, "On-prem model", local_port);
    let agent = set_routing(
        &conn,
        &ws,
        &admin,
        &agent,
        AiAgentModelRouting {
            primary_provider_id: Some(primary),
            local_fallback_provider_id: Some(local),
            force_air_gapped_for: vec!["ssn".into()],
            ..Default::default()
        },
    );

    let outcome = ai_gateway_service::dispatch(
        &conn,
        &ws,
        &master_key(),
        &agent,
        Some(&admin),
        "system",
        &[],
        &sensitive_turn(),
        false,
    )
    .await
    .unwrap();
    assert_eq!(outcome.served_by, "local_fallback");
}

// AC-1: with no local provider configured for the air-gap tier, a sensitive
// step fails closed rather than resolving the workspace default (I-17) - and
// the cloud primary is never dispatched to.
// Covers: REQ-GOV-008 (WP-0123)
#[tokio::test]
async fn air_gap_fails_closed_when_no_local_provider_is_configured() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Reconciler");
    let reachable = spawn_http_stub(200, anthropic_text_body("cloud reply"));
    let primary = make_provider(&conn, &ws, &admin, "Cloud primary", reachable);
    // force_air_gapped_for is set, but local_fallback_provider_id is not.
    let agent = set_routing(
        &conn,
        &ws,
        &admin,
        &agent,
        AiAgentModelRouting {
            primary_provider_id: Some(primary),
            force_air_gapped_for: vec!["ssn".into()],
            ..Default::default()
        },
    );

    let result = ai_gateway_service::dispatch(
        &conn,
        &ws,
        &master_key(),
        &agent,
        Some(&admin),
        "system",
        &[],
        &sensitive_turn(),
        false,
    )
    .await;
    let Err(err) = result else {
        panic!("a sensitive step with no local provider must fail closed");
    };
    assert!(
        err.to_string().contains("local provider"),
        "expected a config.not_local failure, got: {err}"
    );
}

// AC-1: a local_fallback that is resolvable but NOT marked local (e.g. it
// was pointed at a cloud endpoint by mistake) still fails closed - the
// sensitive payload is never dispatched to it.
// Covers: REQ-GOV-008 (WP-0123)
#[tokio::test]
async fn air_gap_fails_closed_when_the_fallback_is_not_marked_local() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Reconciler");
    let port = spawn_http_stub(200, anthropic_text_body("should never be reached"));
    // Created as an ordinary provider: is_local defaults to false.
    let not_local = make_provider(&conn, &ws, &admin, "Mislabelled fallback", port);
    let agent = set_routing(
        &conn,
        &ws,
        &admin,
        &agent,
        AiAgentModelRouting {
            local_fallback_provider_id: Some(not_local),
            force_air_gapped_for: vec!["ssn".into()],
            ..Default::default()
        },
    );

    let result = ai_gateway_service::dispatch(
        &conn,
        &ws,
        &master_key(),
        &agent,
        Some(&admin),
        "system",
        &[],
        &sensitive_turn(),
        false,
    )
    .await;
    let Err(err) = result else {
        panic!("a non-local fallback must fail closed under air-gap");
    };
    assert!(
        err.to_string().contains("local provider"),
        "expected a config.not_local failure, got: {err}"
    );
}

// AC-2: the fallback chain order is honoured and recorded - a failed primary
// advances to the fallback tier, which answers, and the failover is logged.
// Covers: REQ-LOOP-003 (WP-0123)
#[tokio::test]
async fn the_fallback_tier_answers_when_the_primary_fails_and_is_recorded() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Resilient");
    let down = spawn_http_stub(500, r#"{"error":"boom"}"#.to_string());
    let up = spawn_http_stub(200, anthropic_text_body("served by the fallback"));
    let primary = make_provider(&conn, &ws, &admin, "Primary (down)", down);
    let fallback = make_provider(&conn, &ws, &admin, "Fallback (up)", up);
    let agent = set_routing(
        &conn,
        &ws,
        &admin,
        &agent,
        AiAgentModelRouting {
            primary_provider_id: Some(primary),
            fallback_provider_id: Some(fallback),
            ..Default::default()
        },
    );

    let outcome = ai_gateway_service::dispatch(
        &conn,
        &ws,
        &master_key(),
        &agent,
        Some(&admin),
        "system",
        &[],
        &[user_turn("What's the status?")],
        false,
    )
    .await
    .unwrap();
    assert_eq!(outcome.served_by, "fallback");

    let events = ai_gateway_service::recent_failover_events(&conn, &ws, 10).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].served_by, "fallback");
}
