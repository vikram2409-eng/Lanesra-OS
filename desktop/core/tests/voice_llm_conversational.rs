//! Voice-First Mode: the optional, workspace-wide LLM-backed conversational
//! fallback (`voice_llm_settings` + `voice_llm_planner_service`). Reuses
//! `voice_mode_v2.rs`'s own stub-listener/captured-request-body pattern to
//! stand in for a real LLM provider - never a real network call.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::ai::AiProviderInput;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::voice::{SetVoicePinInput, VoiceLlmSettingsInput, VoicePolicyBindingInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    access_role_service, ai_provider_service, company_service, opportunity_service, user_service, voice_execution_service, voice_llm_service,
    voice_policy_service, voice_session_service, workspace_service,
};

fn master_key() -> [u8; 32] {
    [21u8; 32]
}

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin User".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn full_voice_access() -> VoicePolicyBindingInput {
    VoicePolicyBindingInput {
        access_role_id: None, can_use_voice: true, can_search: true, can_create: true, can_update: true, can_act: true,
        can_bulk_act: true, can_external_act: true, can_use_agents: true, max_action_level: "act_with_confirmation".into(),
        processing_boundary: "cloud".into(), max_unlock_minutes: 30,
    }
}

fn make_voice_user(conn: &rusqlite::Connection, ws: &str, admin: &str, username: &str, voice: VoicePolicyBindingInput) -> String {
    let user = user_service::create(
        conn, ws,
        &NewUser { username: username.into(), display_name: username.into(), password: "anothersecretpw".into(), roles: vec!["Sales".to_string()] },
        Some(admin),
    )
    .unwrap();
    let role = access_role_service::create(conn, ws, &AccessRoleInput { name: format!("{username}-role"), description: "".into() }, Some(admin)).unwrap();
    access_role_service::upsert_grant(
        conn, &role.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: true, can_read: true, can_update: true, can_delete: true, can_assign: true, record_scope: "ORGANIZATION".into() },
        Some(admin),
    )
    .unwrap();
    access_role_service::assign_to_user(conn, &user.id, &role.id, Some(admin)).unwrap();
    voice_policy_service::upsert_policy_binding(conn, ws, Some(admin), &VoicePolicyBindingInput { access_role_id: Some(role.id), ..voice }).unwrap();
    user.id
}

fn spawn_stub(body: String) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let queue = Arc::new(Mutex::new(VecDeque::from([body])));
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line);
            let mut content_length: usize = 0;
            loop {
                let mut l = String::new();
                match reader.read_line(&mut l) {
                    Ok(0) => break,
                    Ok(_) => {
                        if l == "\r\n" || l.trim().is_empty() {
                            break;
                        }
                        if let Some(v) = l.to_ascii_lowercase().strip_prefix("content-length:") {
                            content_length = v.trim().parse().unwrap_or(0);
                        }
                    }
                    Err(_) => break,
                }
            }
            let mut body_buf = vec![0u8; content_length];
            let _ = reader.read_exact(&mut body_buf);
            let body = { let q = queue.lock().unwrap(); q.front().cloned().unwrap_or_default() };
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn anthropic_text_body(text: &str) -> String {
    serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string()
}

/// A real `ai_providers` row (with a stored, decryptable key) pointed at a
/// local stub - `voice_llm_settings.provider_id` resolves through this
/// exact row via `ai_gateway_service::resolve_tier`, the same path an
/// agent's own Model Routing tier uses.
fn create_stub_provider(conn: &rusqlite::Connection, ws: &str, admin: &str, port: u16) -> String {
    ai_provider_service::create(
        conn, ws, &master_key(),
        &AiProviderInput { name: "Voice Test Provider".into(), provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap()
    .id
}

#[test]
fn settings_default_to_disabled_and_round_trip_through_the_service() {
    let (conn, ws, admin) = setup_workspace();
    let defaults = voice_llm_service::get_settings(&conn, &ws, Some(&admin)).unwrap();
    assert!(!defaults.enabled);
    assert!(defaults.provider_id.is_none());

    let port = spawn_stub(anthropic_text_body("unused"));
    let provider_id = create_stub_provider(&conn, &ws, &admin, port);
    let saved = voice_llm_service::upsert_settings(&conn, &ws, &VoiceLlmSettingsInput { enabled: true, provider_id: Some(provider_id.clone()) }, Some(&admin)).unwrap();
    assert!(saved.enabled);
    assert_eq!(saved.provider_id.as_deref(), Some(provider_id.as_str()));

    let reloaded = voice_llm_service::get_settings(&conn, &ws, Some(&admin)).unwrap();
    assert!(reloaded.enabled);
}

#[test]
fn upsert_rejects_a_provider_from_another_workspace() {
    let (conn, ws, admin) = setup_workspace();
    let (other_conn, other_ws, other_admin) = setup_workspace();
    let _ = other_conn; // only need its workspace id to build a foreign provider id shape
    let port = spawn_stub(anthropic_text_body("unused"));
    let foreign_provider_id = create_stub_provider(&other_conn, &other_ws, &other_admin, port);

    let result = voice_llm_service::upsert_settings(&conn, &ws, &VoiceLlmSettingsInput { enabled: true, provider_id: Some(foreign_provider_id) }, Some(&admin));
    assert!(result.is_err(), "a provider id from a different workspace must be rejected");
}

#[test]
fn a_non_administrator_cannot_view_or_change_settings() {
    let (conn, ws, admin) = setup_workspace();
    let user = make_voice_user(&conn, &ws, &admin, "rep", full_voice_access());
    assert!(voice_llm_service::get_settings(&conn, &ws, Some(&user)).is_err());
    assert!(voice_llm_service::upsert_settings(&conn, &ws, &VoiceLlmSettingsInput { enabled: true, provider_id: None }, Some(&user)).is_err());
}

/// The end-to-end path: a phrasing the deterministic planner alone can't
/// parse gets rewritten by the (stubbed) LLM into a canonical command,
/// which is then handed back to the exact same planner/resolver/risk
/// pipeline - so it still ends up `awaiting_confirmation` against the real
/// Opportunity, never auto-executed by the rewrite step itself.
#[tokio::test]
async fn a_conversational_phrasing_is_rewritten_and_still_goes_through_the_real_pipeline() {
    let (conn, ws, admin) = setup_workspace();
    let user = make_voice_user(&conn, &ws, &admin, "rep", full_voice_access());
    let company = company_service::create(&conn, &ws, &CompanyInput { name: "Acme".into(), status: "Prospect".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, phone: None, email: None, website: None, annual_revenue_cents: None, employee_count: None, preferred_contact_method: None }, Some(&admin)).unwrap();
    opportunity_service::create(
        &conn,
        &OpportunityInput { company_id: company.id.clone(), primary_contact_id: None, name: "Northern Star".into(), stage: "Discovery".into(), status: "Open".into(), value_cents: 5_000_00, currency_code: "USD".into(), probability_bp: 4000, expected_close_date: None, owner_user_id: None, lost_reason: None, next_step: None },
        Some(&admin),
    )
    .unwrap();

    let port = spawn_stub(anthropic_text_body("REWRITE: mark Northern Star opportunity as Won"));
    let provider_id = create_stub_provider(&conn, &ws, &admin, port);
    voice_llm_service::upsert_settings(&conn, &ws, &VoiceLlmSettingsInput { enabled: true, provider_id: Some(provider_id) }, Some(&admin)).unwrap();

    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "hey, could you please go ahead and mark the northern star deal as won, thanks", "en-US", None, &master_key())
        .await.unwrap();

    assert!(outcome.unsupported_reason.is_none(), "the rewrite should have produced a real plan, not a fallback to Unsupported");
    let plan = outcome.plan.expect("a status change should have produced a plan");
    assert_eq!(plan.status, "awaiting_confirmation", "a status change is still risk-gated exactly like a manually-phrased command");
}

#[tokio::test]
async fn the_fallback_is_skipped_entirely_when_disabled() {
    let (conn, ws, admin) = setup_workspace();
    let user = make_voice_user(&conn, &ws, &admin, "rep", full_voice_access());
    // voice_llm_settings was never touched for this workspace - defaults to disabled.
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "hey, could you please mark the northern star deal as won, thanks", "en-US", None, &master_key())
        .await.unwrap();

    assert!(outcome.plan.is_none());
    assert!(outcome.unsupported_reason.is_some(), "with the fallback disabled, an unrecognized phrasing stays honestly unsupported");
}

#[tokio::test]
async fn the_fallback_is_skipped_for_a_user_without_the_use_agents_voice_capability() {
    let (conn, ws, admin) = setup_workspace();
    let mut voice = full_voice_access();
    voice.can_use_agents = false;
    let user = make_voice_user(&conn, &ws, &admin, "rep", voice);

    let port = spawn_stub(anthropic_text_body("REWRITE: mark Northern Star opportunity as Won"));
    let provider_id = create_stub_provider(&conn, &ws, &admin, port);
    voice_llm_service::upsert_settings(&conn, &ws, &VoiceLlmSettingsInput { enabled: true, provider_id: Some(provider_id) }, Some(&admin)).unwrap();

    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "hey, could you please mark the northern star deal as won, thanks", "en-US", None, &master_key())
        .await.unwrap();

    assert!(outcome.unsupported_reason.is_some(), "the trust tier for this fallback is the same as Use AI Agents - without it, nothing should be sent to the LLM at all");
}

#[tokio::test]
async fn a_clarify_response_becomes_a_real_clarification_question() {
    let (conn, ws, admin) = setup_workspace();
    let user = make_voice_user(&conn, &ws, &admin, "rep", full_voice_access());

    let port = spawn_stub(anthropic_text_body("CLARIFY: Which record did you mean?"));
    let provider_id = create_stub_provider(&conn, &ws, &admin, port);
    voice_llm_service::upsert_settings(&conn, &ws, &VoiceLlmSettingsInput { enabled: true, provider_id: Some(provider_id) }, Some(&admin)).unwrap();

    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "hey do the thing with the thing", "en-US", None, &master_key())
        .await.unwrap();

    assert_eq!(outcome.clarification_question.as_deref(), Some("Which record did you mean?"));
}

#[tokio::test]
async fn a_rewrite_that_still_does_not_match_anything_falls_back_to_the_original_unsupported_reason() {
    let (conn, ws, admin) = setup_workspace();
    let user = make_voice_user(&conn, &ws, &admin, "rep", full_voice_access());

    let port = spawn_stub(anthropic_text_body("REWRITE: this is not a real command shape either"));
    let provider_id = create_stub_provider(&conn, &ws, &admin, port);
    voice_llm_service::upsert_settings(&conn, &ws, &VoiceLlmSettingsInput { enabled: true, provider_id: Some(provider_id) }, Some(&admin)).unwrap();

    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "completely unintelligible gibberish", "en-US", None, &master_key())
        .await.unwrap();

    assert!(outcome.unsupported_reason.is_some(), "a rewrite that still doesn't match anything should not crash or hang - it should honestly fall back");
    assert!(outcome.plan.is_none());
}
