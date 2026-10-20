//! WP-0108: Agent Identities and their credentials.
//!
//! AC-1: an agent credential is never present in a prompt, trace, log or API
//! response (scan test: every text column in the database, the audit log, the
//! serialised API models and the requests sent to the model provider).
//! AC-2: a revoked or expired credential is refused immediately.
//!
//! Covers: REQ-GOV-002, REQ-GOV-029 (WP-0108 AC-1, AC-2). Threats: T-03, T-09.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::domain::{AppError, ErrorClass};
use lanesra_core::models::agent_identity::AgentIdentityInput;
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::principals::EffectiveIdentity;
use lanesra_core::services::{
    agent_identity_service, ai_agent_service, ai_orchestration_service, ai_service, user_service,
    workspace_service,
};

const MASTER_KEY: [u8; 32] = [53u8; 32];

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Identity Co".into(),
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

fn identity_input(owner: &str) -> AgentIdentityInput {
    AgentIdentityInput {
        name: "Nightly Reporter".into(),
        description: Some("Runs the nightly report".into()),
        owner_user_id: owner.into(),
        is_administrator: false,
        access_role_ids: vec![],
    }
}

fn assert_refused(result: Result<impl std::fmt::Debug, AppError>, needle: &str) {
    let err = result.expect_err("credential must be refused");
    assert_eq!(err.class(), ErrorClass::Policy, "{err:?}");
    assert!(err.to_string().contains(needle), "{err}");
}

/// Every TEXT value in every table, so a scan cannot miss a column.
fn all_stored_text(conn: &rusqlite::Connection) -> Vec<String> {
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let mut values = Vec::new();
    for table in tables {
        let mut stmt = conn.prepare(&format!("SELECT * FROM \"{table}\"")).unwrap();
        let columns = stmt.column_count();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            for i in 0..columns {
                if let Ok(Some(text)) = row.get::<_, Option<String>>(i) {
                    values.push(text);
                }
            }
        }
    }
    values
}

/// A model provider that records every request body and answers with text.
fn spawn_recording_provider() -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen_by_server = seen.clone();
    let body =
        serde_json::json!({"content": [{"type": "text", "text": "report done"}]}).to_string();
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
            seen_by_server
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&buf).to_string());
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, seen)
}

#[tokio::test]
async fn ac1_a_credential_is_never_stored_logged_returned_or_sent_to_a_model() {
    // Covers: REQ-GOV-029 (WP-0108 AC-1)
    let (conn, ws, admin) = setup();
    let (port, provider_requests) = spawn_recording_provider();
    ai_service::save_settings(
        &conn,
        &ws,
        &MASTER_KEY,
        &AiSettingsInput {
            provider: "anthropic".into(),
            base_url: Some(format!("http://127.0.0.1:{port}")),
            model: "claude-haiku-4-5-20251001".into(),
            api_key: Some("sk-ant-test".into()),
        },
        Some(&admin),
    )
    .unwrap();

    let identity =
        agent_identity_service::create(&conn, &ws, &identity_input(&admin), Some(&admin)).unwrap();
    let issued = agent_identity_service::issue_credential(
        &conn,
        &ws,
        &identity.id,
        "nightly",
        None,
        Some(&admin),
    )
    .unwrap();
    let rotated = agent_identity_service::rotate_credential(
        &conn,
        &ws,
        &issued.credential.id,
        None,
        Some(&admin),
    )
    .unwrap();
    let tokens = [issued.token.clone(), rotated.token.clone()];

    // The token authenticates, and a run acts as its identity.
    let grant = agent_identity_service::authenticate(&conn, &ws, &rotated.token).unwrap();
    let agent = ai_agent_service::create(
        &conn,
        &ws,
        &AiAgentInput {
            name: "Reporter".into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "You write the nightly report.".into(),
            action_names: vec![],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(&admin),
    )
    .unwrap();
    let run = ai_orchestration_service::run_triggered(
        &conn,
        &ws,
        &MASTER_KEY,
        "agent",
        &agent.id,
        &EffectiveIdentity::unattended(None).with_agent_identity(grant),
        "write the report",
        Some("schedule"),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(run.status, "succeeded", "{run:?}");

    // API responses: the identity and credential listings.
    let responses = [
        serde_json::to_string(&agent_identity_service::list(&conn, &ws, Some(&admin)).unwrap())
            .unwrap(),
        serde_json::to_string(
            &agent_identity_service::list_credentials(&conn, &ws, &identity.id, Some(&admin))
                .unwrap(),
        )
        .unwrap(),
        serde_json::to_string(&run).unwrap(),
    ];
    let stored = all_stored_text(&conn);
    let prompts = provider_requests.lock().unwrap().clone();
    assert!(!prompts.is_empty(), "the run must have called the provider");

    for token in &tokens {
        let hash = lanesra_core::services::secret_service::hash_api_secret(token);
        assert!(
            responses
                .iter()
                .all(|r| !r.contains(token) && !r.contains(&hash)),
            "API response leaks a credential"
        );
        assert!(
            prompts.iter().all(|p| !p.contains(token)),
            "a prompt leaks a credential"
        );
        // Stored text includes the audit log and run traces; only the hash may be kept.
        assert!(
            stored.iter().all(|v| !v.contains(token)),
            "the database stores a plaintext credential"
        );
    }
    // The audit log records the credential events by label, not by value.
    assert!(stored
        .iter()
        .any(|v| v.contains("Issued credential 'nightly'")));
}

#[test]
fn ac2_a_revoked_credential_is_refused_at_once() {
    // Covers: REQ-GOV-029 (WP-0108 AC-2)
    let (conn, ws, admin) = setup();
    let identity =
        agent_identity_service::create(&conn, &ws, &identity_input(&admin), Some(&admin)).unwrap();
    let issued = agent_identity_service::issue_credential(
        &conn,
        &ws,
        &identity.id,
        "ci",
        None,
        Some(&admin),
    )
    .unwrap();
    agent_identity_service::authenticate(&conn, &ws, &issued.token).unwrap();

    agent_identity_service::revoke_credential(&conn, &ws, &issued.credential.id, Some(&admin))
        .unwrap();
    assert_refused(
        agent_identity_service::authenticate(&conn, &ws, &issued.token),
        "revoked",
    );
}

#[test]
fn ac2_a_rotated_credential_is_refused_and_its_replacement_works() {
    // Covers: REQ-GOV-029 (WP-0108 AC-2)
    let (conn, ws, admin) = setup();
    let identity =
        agent_identity_service::create(&conn, &ws, &identity_input(&admin), Some(&admin)).unwrap();
    let old = agent_identity_service::issue_credential(
        &conn,
        &ws,
        &identity.id,
        "ci",
        None,
        Some(&admin),
    )
    .unwrap();
    let new = agent_identity_service::rotate_credential(
        &conn,
        &ws,
        &old.credential.id,
        None,
        Some(&admin),
    )
    .unwrap();
    assert_refused(
        agent_identity_service::authenticate(&conn, &ws, &old.token),
        "revoked",
    );
    assert_eq!(
        agent_identity_service::authenticate(&conn, &ws, &new.token)
            .unwrap()
            .id,
        identity.id
    );
}

#[test]
fn ac2_an_expired_credential_is_refused() {
    // Covers: REQ-GOV-029 (WP-0108 AC-2)
    let (conn, ws, admin) = setup();
    let identity =
        agent_identity_service::create(&conn, &ws, &identity_input(&admin), Some(&admin)).unwrap();
    let issued = agent_identity_service::issue_credential(
        &conn,
        &ws,
        &identity.id,
        "ci",
        Some(1),
        Some(&admin),
    )
    .unwrap();
    // Move the expiry into the past: the next use must fail without waiting.
    conn.execute(
        "UPDATE agent_identity_credentials SET expires_at = ?1 WHERE id = ?2",
        ("2020-01-01T00:00:00+00:00", &issued.credential.id),
    )
    .unwrap();
    assert_refused(
        agent_identity_service::authenticate(&conn, &ws, &issued.token),
        "expired",
    );
}

#[test]
fn ac2_a_disabled_identity_refuses_its_credentials() {
    // Covers: REQ-GOV-029 (WP-0108 AC-2)
    let (conn, ws, admin) = setup();
    let identity =
        agent_identity_service::create(&conn, &ws, &identity_input(&admin), Some(&admin)).unwrap();
    let issued = agent_identity_service::issue_credential(
        &conn,
        &ws,
        &identity.id,
        "ci",
        None,
        Some(&admin),
    )
    .unwrap();
    agent_identity_service::set_active(&conn, &ws, &identity.id, false, Some(&admin)).unwrap();
    assert_refused(
        agent_identity_service::authenticate(&conn, &ws, &issued.token),
        "disabled",
    );
}

#[test]
fn a_credential_is_refused_in_another_workspace_and_an_unknown_token_is_refused() {
    // Covers: REQ-GOV-029 (WP-0108; threat T-12)
    let (conn, ws, admin) = setup();
    let identity =
        agent_identity_service::create(&conn, &ws, &identity_input(&admin), Some(&admin)).unwrap();
    let issued = agent_identity_service::issue_credential(
        &conn,
        &ws,
        &identity.id,
        "ci",
        None,
        Some(&admin),
    )
    .unwrap();
    assert_refused(
        agent_identity_service::authenticate(&conn, "another-workspace", &issued.token),
        "not valid",
    );
    assert_refused(
        agent_identity_service::authenticate(&conn, &ws, "lanesra_agent_00ff"),
        "not valid",
    );
}

#[test]
fn managing_identities_and_credentials_is_administrator_only() {
    // Covers: REQ-GOV-002 (WP-0108)
    let (conn, ws, admin) = setup();
    let rep = user_service::create(
        &conn,
        &ws,
        &NewUser {
            username: "rep".into(),
            display_name: "Sales Rep".into(),
            password: "supersecretpassword".into(),
            roles: vec!["Sales".into()],
        },
        Some(&admin),
    )
    .unwrap()
    .id;
    assert!(agent_identity_service::create(&conn, &ws, &identity_input(&rep), Some(&rep)).is_err());
    let identity =
        agent_identity_service::create(&conn, &ws, &identity_input(&rep), Some(&admin)).unwrap();
    assert!(agent_identity_service::issue_credential(
        &conn,
        &ws,
        &identity.id,
        "x",
        None,
        Some(&rep)
    )
    .is_err());
    assert!(agent_identity_service::list(&conn, &ws, Some(&rep)).is_err());
    assert!(agent_identity_service::delete(&conn, &ws, &identity.id, Some(&rep)).is_err());
}

#[test]
fn a_credential_is_short_lived() {
    // Covers: REQ-GOV-029 (WP-0108)
    let (conn, ws, admin) = setup();
    let identity =
        agent_identity_service::create(&conn, &ws, &identity_input(&admin), Some(&admin)).unwrap();
    let too_long = agent_identity_service::MAX_LIFETIME_HOURS + 1;
    assert!(agent_identity_service::issue_credential(
        &conn,
        &ws,
        &identity.id,
        "x",
        Some(too_long),
        Some(&admin)
    )
    .is_err());
    assert!(agent_identity_service::issue_credential(
        &conn,
        &ws,
        &identity.id,
        "x",
        Some(0),
        Some(&admin)
    )
    .is_err());
}

#[test]
fn an_identitys_grant_carries_its_administrator_flag_and_workspace() {
    // Covers: REQ-GOV-002 (WP-0108)
    let (conn, ws, admin) = setup();
    let mut input = identity_input(&admin);
    input.is_administrator = true;
    let identity = agent_identity_service::create(&conn, &ws, &input, Some(&admin)).unwrap();
    let grant = agent_identity_service::grant_for_run(&conn, &ws, &identity.id).unwrap();
    assert!(grant.is_administrator);
    assert_eq!(grant.workspace_id, ws);
    assert!(
        agent_identity_service::grant_for_run(&conn, "another-workspace", &identity.id).is_err()
    );
}
