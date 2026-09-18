//! Voice-First Mode, PR 2: RUN_AGENT/RUN_PIPELINE - Voice as a new *caller*
//! into the existing AI Agent Foundry/Orchestration entry points
//! (`chat_service::send_agent_message`/`ai_orchestration_service::run_manual`),
//! never a second agent-runtime. Reuses `ai_agent_guardrails.rs`'s own
//! stub-listener/captured-request-body pattern to stand in for a real LLM
//! provider.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::voice::{ConfirmVoicePlanInput, SetVoicePinInput, VoicePolicyBindingInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    access_role_service, ai_agent_service, ai_orchestration_service, ai_service, user_service, voice_execution_service, voice_policy_service,
    voice_session_service, workspace_service,
};

fn master_key() -> [u8; 32] {
    [11u8; 32]
}

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Test Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin User".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn make_voice_user(conn: &rusqlite::Connection, ws: &str, admin: &str, username: &str, voice: VoicePolicyBindingInput) -> String {
    let user = user_service::create(
        conn,
        ws,
        &NewUser { username: username.into(), display_name: username.into(), password: "anothersecretpw".into(), roles: vec!["Sales".to_string()] },
        Some(admin),
    )
    .unwrap();
    let role = access_role_service::create(conn, ws, &AccessRoleInput { name: format!("{username}-role"), description: "".into() }, Some(admin)).unwrap();
    access_role_service::upsert_grant(
        conn,
        &role.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: true, can_read: true, can_update: true, can_delete: true, can_assign: true, record_scope: "ORGANIZATION".into() },
        Some(admin),
    )
    .unwrap();
    access_role_service::assign_to_user(conn, &user.id, &role.id, Some(admin)).unwrap();
    voice_policy_service::upsert_policy_binding(conn, ws, Some(admin), &VoicePolicyBindingInput { access_role_id: Some(role.id), ..voice }).unwrap();
    user.id
}

fn full_voice_access() -> VoicePolicyBindingInput {
    VoicePolicyBindingInput {
        access_role_id: None,
        can_use_voice: true,
        can_search: true,
        can_create: true,
        can_update: true,
        can_act: true,
        can_bulk_act: true,
        can_external_act: true,
        can_use_agents: true,
        max_action_level: "act_with_confirmation".into(),
        processing_boundary: "cloud".into(),
        max_unlock_minutes: 30,
    }
}

fn spawn_sequence_stub(bodies: Vec<String>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let queue = Arc::new(Mutex::new(VecDeque::from(bodies)));
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
            let body = {
                let mut q = queue.lock().unwrap();
                if q.len() > 1 { q.pop_front().unwrap() } else { q.front().cloned().unwrap_or_default() }
            };
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn anthropic_text_body(text: &str) -> String {
    serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string()
}

fn configure_stub_provider(conn: &rusqlite::Connection, ws: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn,
        ws,
        &master_key(),
        &AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap();
}

fn create_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> String {
    ai_agent_service::create(
        conn,
        ws,
        &AiAgentInput { name: name.into(), description: None, icon: "🤖".into(), system_prompt: "You are a helpful assistant.".into(), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(admin),
    )
    .unwrap()
    .id
}

#[tokio::test]
async fn run_agent_resolves_named_agent_confirms_and_speaks_its_reply() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_sequence_stub(vec![anthropic_text_body("The CRM Modernization deal is at 60% probability.")]);
    configure_stub_provider(&conn, &ws, &admin, port);
    create_agent(&conn, &ws, &admin, "Sales Coach");

    let user = make_voice_user(&conn, &ws, &admin, "asker", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "ask the Sales Coach agent to check the CRM Modernization deal", "en-US", None, &master_key()).await.unwrap();
    let plan = outcome.plan.expect("expected a RUN_AGENT plan");
    assert_eq!(plan.status, "awaiting_confirmation", "RUN_AGENT is Medium risk and must always confirm");
    assert_eq!(plan.risk.as_str(), "medium");

    let result = voice_execution_service::confirm_plan(&conn, &session.id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();
    assert_eq!(result.status, "succeeded");
    assert_eq!(result.executions.len(), 1);
    assert_eq!(result.executions[0].entity_type, "AiAgent");
    assert_eq!(result.executions[0].result, "ok");
    assert!(result.executions[0].undo_token.is_none(), "a chat turn has no undo path");
    assert_eq!(result.notes.len(), 1);
    assert!(result.notes[0].contains("60% probability"), "expected the agent's real reply text in notes, got: {:?}", result.notes);
}

#[tokio::test]
async fn run_agent_with_ambiguous_name_asks_for_clarification_instead_of_guessing() {
    let (conn, ws, admin) = setup_workspace();
    create_agent(&conn, &ws, &admin, "Sales Coach");
    create_agent(&conn, &ws, &admin, "Senior Sales Coach");
    let user = make_voice_user(&conn, &ws, &admin, "asker2", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "ask the Sales Coach agent to summarize today", "en-US", None, &master_key()).await.unwrap();
    assert!(outcome.plan.is_none());
    assert!(outcome.clarification_question.is_some());
    assert_eq!(outcome.candidates.len(), 2);
}

#[tokio::test]
async fn run_agent_not_found_is_honestly_unsupported() {
    let (conn, ws, admin) = setup_workspace();
    let user = make_voice_user(&conn, &ws, &admin, "asker3", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "ask the Nonexistent Bot agent to do something", "en-US", None, &master_key()).await.unwrap();
    assert!(outcome.plan.is_none());
    let reason = outcome.unsupported_reason.expect("expected an honest not-found reason");
    assert!(reason.contains("Nonexistent Bot"), "expected the reason to name what wasn't found, got: {reason}");
}

#[tokio::test]
async fn run_agent_is_blocked_without_voice_ai_agents_capability_even_at_act_level() {
    let (conn, ws, admin) = setup_workspace();
    create_agent(&conn, &ws, &admin, "Sales Coach");
    // "act" is the most permissive Max Action Level, but can_use_agents is
    // off - Voice's own narrower capability gate must still block RUN_AGENT
    // regardless of how high max_action_level is set (the same shape
    // voice_mode_v1.rs's own required_capability test already proves for
    // update_status).
    let user = make_voice_user(&conn, &ws, &admin, "asker4", VoicePolicyBindingInput { max_action_level: "act".into(), can_use_agents: false, ..full_voice_access() });
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "ask the Sales Coach agent to summarize today", "en-US", None, &master_key()).await.unwrap();
    assert!(outcome.plan.is_none(), "a capability-blocked RUN_AGENT command must not produce an executable plan");
    let reason = outcome.unsupported_reason.expect("expected a blocked reason");
    assert!(reason.contains("Voice AI Agents"), "expected the reason to name the missing capability, got: {reason}");
}

#[tokio::test]
async fn run_pipeline_resolves_named_pipeline_and_executes_through_the_real_orchestrator() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_sequence_stub(vec![anthropic_text_body("Lead triaged: high priority, route to enterprise sales.")]);
    configure_stub_provider(&conn, &ws, &admin, port);
    let agent_id = create_agent(&conn, &ws, &admin, "Triage Agent");
    let pipeline = ai_orchestration_service::create_pipeline(
        &conn,
        &ws,
        &AiAgentPipelineInput { name: "Lead Triage".into(), description: None, topology: "sequential".into(), steps: vec![PipelineStepInput { agent_id, input_template: "{{trigger_input}}".into(), requires_approval: false }] },
        Some(&admin),
    )
    .unwrap();

    // `ai_orchestration_service::run_manual` is Administrator-gated
    // regardless of caller (the same "Run now" button's own restriction) -
    // Voice is a new caller into that entry point, never a way around it
    // (this feature's own non-negotiable design principle), so this test
    // unlocks Voice for the workspace admin themself rather than an
    // ordinary Access-Role user. Granted through a real, role-scoped
    // binding (not the workspace-default `access_role_id: None` row) -
    // every other passing test in this file already proves that path
    // works end to end.
    let admin_role = access_role_service::create(&conn, &ws, &AccessRoleInput { name: "admin-voice-role".into(), description: "".into() }, Some(&admin)).unwrap();
    access_role_service::assign_to_user(&conn, &admin, &admin_role.id, Some(&admin)).unwrap();
    voice_policy_service::upsert_policy_binding(&conn, &ws, Some(&admin), &VoicePolicyBindingInput { access_role_id: Some(admin_role.id), ..full_voice_access() }).unwrap();
    voice_session_service::set_pin(&conn, &admin, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &admin, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &admin, &format!("run pipeline {} with a new lead from the website", pipeline.name), "en-US", None, &master_key()).await.unwrap();
    let plan = outcome.plan.expect("expected a RUN_AGENT plan for the pipeline");
    assert_eq!(plan.status, "awaiting_confirmation");

    let result = voice_execution_service::confirm_plan(&conn, &session.id, &admin, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();
    assert_eq!(result.status, "succeeded");
    assert_eq!(result.executions[0].entity_type, "AiAgentPipeline");
    assert_eq!(result.notes.len(), 1);
    assert!(result.notes[0].contains("high priority"), "expected the pipeline's real step output in notes, got: {:?}", result.notes);

    let runs = ai_orchestration_service::list_runs(&conn, "pipeline", &pipeline.id, 10).unwrap();
    assert_eq!(runs.len(), 1, "the run should be a real, queryable AiAgentRun - not a Voice-only side record");
}
