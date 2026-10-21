//! WP-0107: who an agent run acts as, proven on real runs
//! (`ai_orchestration_service::run_triggered`).
//!
//! AC-1: an unattended run of an admin-scoped agent fails unless its Agent
//! Identity holds Administrator; it never inherits an Administrator's rights,
//! not even from the owner of the API client that triggered it.
//! AC-2: an interactive run never exceeds the starting user's access, even
//! when the agent's identity has more.
//!
//! Covers: REQ-GOV-002, REQ-GOV-003 (WP-0107 AC-1, AC-2). Threats: T-03, T-12.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::AiAgentRun;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::principals::{AgentIdentityGrant, EffectiveIdentity};
use lanesra_core::services::{
    ai_agent_service, ai_orchestration_service, ai_service, user_service, workspace_service,
};

const MASTER_KEY: [u8; 32] = [41u8; 32];

fn setup() -> (rusqlite::Connection, String, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Principals Co".into(),
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
    let rep = user_service::create(
        &conn,
        &workspace.id,
        &NewUser {
            username: "rep".into(),
            display_name: "Sales Rep".into(),
            password: "supersecretpassword".into(),
            roles: vec!["Sales".into()],
        },
        Some(&admin.id),
    )
    .unwrap()
    .id;
    (conn, workspace.id, admin.id, rep)
}

/// A model provider that answers every request with the same text.
fn spawn_text_stub(text: &str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let body = serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut content_length = 0usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line.trim().is_empty() {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    content_length = v.trim().parse().unwrap_or(0);
                }
            }
            let mut buf = vec![0u8; content_length];
            let _ = reader.read_exact(&mut buf);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn configure_provider(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    let port = spawn_text_stub("done");
    ai_service::save_settings(
        conn,
        ws,
        &MASTER_KEY,
        &AiSettingsInput {
            provider: "anthropic".into(),
            base_url: Some(format!("http://127.0.0.1:{port}")),
            model: "claude-haiku-4-5-20251001".into(),
            api_key: Some("sk-ant-test".into()),
        },
        Some(admin),
    )
    .unwrap();
}

/// An agent whose tools need Administrator (`list_business_rules` is an
/// admin-catalog tool).
fn admin_scoped_agent(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    ai_agent_service::create(
        conn,
        ws,
        &AiAgentInput {
            name: "Rules Auditor".into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "You audit business rules.".into(),
            action_names: vec!["list_business_rules".into()],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(admin),
    )
    .unwrap()
    .id
}

fn identity(ws: &str, is_administrator: bool) -> AgentIdentityGrant {
    AgentIdentityGrant {
        id: "identity-1".into(),
        workspace_id: ws.into(),
        is_administrator,
    }
}

async fn run(
    conn: &rusqlite::Connection,
    ws: &str,
    agent_id: &str,
    who: &EffectiveIdentity,
) -> AiAgentRun {
    ai_orchestration_service::run_triggered(
        conn,
        ws,
        &MASTER_KEY,
        "agent",
        agent_id,
        who,
        "check the rules",
        Some("webhook"),
        None,
        None,
    )
    .await
    .unwrap()
}

fn assert_refused(run: &AiAgentRun, needle: &str) {
    assert_eq!(run.status, "failed", "{run:?}");
    let error = run.error.as_deref().unwrap_or_default();
    assert!(error.contains(needle), "{error}");
}

#[tokio::test]
async fn ac1_webhook_run_never_borrows_the_api_client_owners_admin_rights() {
    // Covers: REQ-GOV-003 (WP-0107 AC-1)
    let (conn, ws, admin, _) = setup();
    configure_provider(&conn, &ws, &admin);
    let agent = admin_scoped_agent(&conn, &ws, &admin);
    // The API client's owner is an Administrator; the run still has no
    // Agent Identity, so it must not run.
    let who = EffectiveIdentity::unattended(Some(&admin));
    assert_refused(&run(&conn, &ws, &agent, &who).await, "never borrows");
}

#[tokio::test]
async fn ac1_scheduled_run_without_an_agent_identity_is_refused() {
    // Covers: REQ-GOV-003 (WP-0107 AC-1)
    let (conn, ws, admin, _) = setup();
    configure_provider(&conn, &ws, &admin);
    let agent = admin_scoped_agent(&conn, &ws, &admin);
    let who = EffectiveIdentity::unattended(None);
    assert_refused(
        &run(&conn, &ws, &agent, &who).await,
        "needs an Agent Identity",
    );
}

#[tokio::test]
async fn ac1_unattended_run_whose_identity_is_not_administrator_is_refused() {
    // Covers: REQ-GOV-003 (WP-0107 AC-1)
    let (conn, ws, admin, _) = setup();
    configure_provider(&conn, &ws, &admin);
    let agent = admin_scoped_agent(&conn, &ws, &admin);
    let who = EffectiveIdentity::unattended(Some(&admin)).with_agent_identity(identity(&ws, false));
    assert_refused(
        &run(&conn, &ws, &agent, &who).await,
        "needs an Agent Identity",
    );
}

#[tokio::test]
async fn ac1_an_identity_from_another_workspace_is_refused() {
    // Covers: REQ-GOV-003 (WP-0107 AC-1; threat T-12)
    let (conn, ws, admin, _) = setup();
    configure_provider(&conn, &ws, &admin);
    let agent = admin_scoped_agent(&conn, &ws, &admin);
    let who = EffectiveIdentity::unattended(None).with_agent_identity(identity("other-ws", true));
    assert_refused(&run(&conn, &ws, &agent, &who).await, "another workspace");
}

#[tokio::test]
async fn ac2_interactive_run_by_a_non_administrator_is_refused() {
    // Covers: REQ-GOV-002 (WP-0107 AC-2)
    let (conn, ws, admin, rep) = setup();
    configure_provider(&conn, &ws, &admin);
    let agent = admin_scoped_agent(&conn, &ws, &admin);
    // The agent's identity holds Administrator, the person does not.
    let who = EffectiveIdentity::interactive(&rep).with_agent_identity(identity(&ws, true));
    assert_refused(&run(&conn, &ws, &agent, &who).await, "Administrator");
}

#[tokio::test]
async fn ac2_interactive_run_is_also_limited_by_its_agent_identity() {
    // Covers: REQ-GOV-002 (WP-0107 AC-2)
    let (conn, ws, admin, _) = setup();
    configure_provider(&conn, &ws, &admin);
    let agent = admin_scoped_agent(&conn, &ws, &admin);
    let who = EffectiveIdentity::interactive(&admin).with_agent_identity(identity(&ws, false));
    assert_refused(&run(&conn, &ws, &agent, &who).await, "does not hold");
}

#[tokio::test]
async fn ac2_interactive_run_by_an_administrator_runs() {
    // Covers: REQ-GOV-002 (WP-0107 AC-2)
    let (conn, ws, admin, _) = setup();
    configure_provider(&conn, &ws, &admin);
    let agent = admin_scoped_agent(&conn, &ws, &admin);
    let who = EffectiveIdentity::interactive(&admin);
    let run = run(&conn, &ws, &agent, &who).await;
    assert_eq!(run.status, "succeeded", "{run:?}");
}

#[test]
fn refusals_are_classified_policy_errors() {
    // Covers: REQ-GOV-003 (WP-0107 AC-1)
    let (conn, ws, admin, _) = setup();
    let err = EffectiveIdentity::unattended(Some(&admin))
        .require_admin(&conn, &ws)
        .unwrap_err();
    assert_eq!(err.class(), lanesra_core::domain::ErrorClass::Policy);
    assert_eq!(err.code(), Some("policy.access_denied"));
}
