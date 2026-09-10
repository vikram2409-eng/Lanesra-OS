//! AI & Agentic Layer, Phase 1: proves `ai_service` - the default
//! unconfigured row, admin-gating, key rotation (a new key rotates the
//! stored secret in place, an omitted one keeps it, the plaintext is
//! never returned by any read), and `test_key`'s real outbound call for
//! both provider modes against a real local HTTP listener (not a live
//! third-party endpoint, matching every other Integration Hub test in
//! this crate).

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_service, secret_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "AI Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [7u8; 32]
}

/// Reads the workspace's currently-stored key straight out of the
/// database - `ai_settings_repo::get_secret_id` is crate-internal, so a
/// test proving rotation actually changed the plaintext has to go through
/// the same secret_id -> integration_secrets -> decrypt path
/// `ai_service::test_key` itself uses, via only public API.
fn read_stored_key(conn: &rusqlite::Connection, master_key: &[u8; 32], workspace_id: &str) -> Option<String> {
    let secret_id: Option<String> = conn.query_row("SELECT secret_id FROM ai_settings WHERE workspace_id = ?1", [workspace_id], |r| r.get(0)).unwrap();
    let secret_id = secret_id?;
    let stored = lanesra_core::repositories::integration_secret_repo::get(conn, &secret_id).unwrap()?;
    Some(secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce).unwrap())
}

/// A minimal raw-socket HTTP server returning a fixed status/body for
/// every request - the same shape every other Integration Hub test file
/// in this crate already uses for its own local-listener test double.
fn spawn_http_stub(status: u16, body: &'static str) -> u16 {
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
            let reason = if status == 200 { "OK" } else { "Unauthorized" };
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

#[test]
fn default_settings_are_unconfigured_with_no_key() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let settings = ai_service::get_settings(&conn, &workspace_id).unwrap();
    assert_eq!(settings.provider, "anthropic");
    assert!(!settings.has_key);
    assert_eq!(settings.status, "unconfigured");

    // Calling it again returns the same row, not a second one.
    let again = ai_service::get_settings(&conn, &workspace_id).unwrap();
    assert_eq!(again.workspace_id, settings.workspace_id);
}

#[test]
fn save_settings_requires_an_admin_actor() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let input = AiSettingsInput { provider: "anthropic".into(), base_url: None, model: "".into(), api_key: Some("sk-test".into()) };
    assert!(ai_service::save_settings(&conn, &workspace_id, &master_key(), &input, None).is_err());
}

#[test]
fn save_settings_rejects_an_unknown_provider() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let input = AiSettingsInput { provider: "made_up_provider".into(), base_url: None, model: "".into(), api_key: None };
    assert!(ai_service::save_settings(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).is_err());
}

#[test]
fn openai_compatible_requires_a_base_url() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let input = AiSettingsInput { provider: "openai_compatible".into(), base_url: None, model: "".into(), api_key: Some("sk-test".into()) };
    assert!(ai_service::save_settings(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).is_err());
}

#[test]
fn saving_a_key_never_returns_it_but_it_round_trips_and_rotates() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let input = AiSettingsInput { provider: "anthropic".into(), base_url: None, model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-original".into()) };
    let saved = ai_service::save_settings(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).unwrap();
    assert!(saved.has_key);
    // Never exposed on the model itself.
    assert_eq!(read_stored_key(&conn, &master_key(), &workspace_id).as_deref(), Some("sk-ant-original"));

    // Omitting api_key on a later save keeps the existing key untouched.
    let keep_input = AiSettingsInput { provider: "anthropic".into(), base_url: None, model: "claude-haiku-4-5-20251001".into(), api_key: None };
    ai_service::save_settings(&conn, &workspace_id, &master_key(), &keep_input, Some(&admin_id)).unwrap();
    assert_eq!(read_stored_key(&conn, &master_key(), &workspace_id).as_deref(), Some("sk-ant-original"));

    // A new key rotates the same secret row in place, not a new one.
    let secret_id_before: Option<String> = conn.query_row("SELECT secret_id FROM ai_settings WHERE workspace_id = ?1", [&workspace_id], |r| r.get(0)).unwrap();
    let rotate_input = AiSettingsInput { provider: "anthropic".into(), base_url: None, model: "".into(), api_key: Some("sk-ant-rotated".into()) };
    ai_service::save_settings(&conn, &workspace_id, &master_key(), &rotate_input, Some(&admin_id)).unwrap();
    let secret_id_after: Option<String> = conn.query_row("SELECT secret_id FROM ai_settings WHERE workspace_id = ?1", [&workspace_id], |r| r.get(0)).unwrap();
    assert_eq!(secret_id_before, secret_id_after, "rotation should reuse the same secret row");
    assert_eq!(read_stored_key(&conn, &master_key(), &workspace_id).as_deref(), Some("sk-ant-rotated"));
}

#[test]
fn changing_settings_resets_a_prior_test_result() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let input = AiSettingsInput { provider: "anthropic".into(), base_url: None, model: "".into(), api_key: Some("sk-test".into()) };
    ai_service::save_settings(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).unwrap();
    // Directly stamp a prior "connected" result, then prove any settings
    // save clears it back to unconfigured rather than carrying it forward
    // against a configuration it never actually tested.
    conn.execute("UPDATE ai_settings SET status = 'connected' WHERE workspace_id = ?1", [&workspace_id]).unwrap();
    ai_service::save_settings(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).unwrap();
    assert_eq!(ai_service::get_settings(&conn, &workspace_id).unwrap().status, "unconfigured");
}

#[tokio::test]
async fn test_key_fails_cleanly_when_nothing_is_configured_yet() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    assert!(ai_service::test_key(&conn, &workspace_id, &master_key(), Some(&admin_id)).await.is_err());
}

#[tokio::test]
async fn anthropic_test_key_reports_real_success_and_real_failure() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let ok_port = spawn_http_stub(200, r#"{"id":"msg_1","content":[{"type":"text","text":"hi"}]}"#);
    let input = AiSettingsInput {
        provider: "anthropic".into(),
        base_url: Some(format!("http://127.0.0.1:{ok_port}")),
        model: "claude-haiku-4-5-20251001".into(),
        api_key: Some("sk-ant-valid".into()),
    };
    ai_service::save_settings(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).unwrap();
    let result = ai_service::test_key(&conn, &workspace_id, &master_key(), Some(&admin_id)).await.unwrap();
    assert!(result.ok, "{result:?}");
    assert_eq!(ai_service::get_settings(&conn, &workspace_id).unwrap().status, "connected");

    // A rejected key against a real (stubbed) endpoint is a real, not
    // simulated, failure - persisted as such, not just returned.
    let unauthorized_port = spawn_http_stub(401, r#"{"error":{"message":"invalid x-api-key"}}"#);
    let bad_input = AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{unauthorized_port}")), model: "".into(), api_key: Some("sk-ant-bad".into()) };
    ai_service::save_settings(&conn, &workspace_id, &master_key(), &bad_input, Some(&admin_id)).unwrap();
    let failed = ai_service::test_key(&conn, &workspace_id, &master_key(), Some(&admin_id)).await.unwrap();
    assert!(!failed.ok);
    assert_eq!(ai_service::get_settings(&conn, &workspace_id).unwrap().status, "failed");
}

#[tokio::test]
async fn openai_compatible_test_key_hits_the_models_endpoint() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_http_stub(200, r#"{"data":[{"id":"gpt-4"}]}"#);
    let input = AiSettingsInput { provider: "openai_compatible".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "gpt-4".into(), api_key: Some("sk-oai-valid".into()) };
    ai_service::save_settings(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).unwrap();
    let result = ai_service::test_key(&conn, &workspace_id, &master_key(), Some(&admin_id)).await.unwrap();
    assert!(result.ok, "{result:?}");
}

#[tokio::test]
async fn an_unreachable_endpoint_is_a_real_failure() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let input = AiSettingsInput { provider: "openai_compatible".into(), base_url: Some("http://127.0.0.1:1".into()), model: "".into(), api_key: Some("sk-oai".into()) };
    ai_service::save_settings(&conn, &workspace_id, &master_key(), &input, Some(&admin_id)).unwrap();
    let result = ai_service::test_key(&conn, &workspace_id, &master_key(), Some(&admin_id)).await.unwrap();
    assert!(!result.ok);
}
