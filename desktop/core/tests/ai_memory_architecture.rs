//! AI Agent Platform v2, Phase 4 (GitHub issue #169): the three itemized
//! memory types (Session/Working/Entity) on top of migration
//! `0063_memory_architecture.sql`'s `ai_memory_items` table -
//! `ai_memory_service`'s write-side policy gate, context-scoped reads, the
//! admin Memory Inspector's forget action, TTL reclaim, and the real
//! `remember`/`get_memory` tool wiring through `chat_service`'s agent
//! tool-calling loop, including Session Memory's ambient injection into a
//! later conversation turn's system prompt. Agent Memory (`memory_md`) is
//! unaffected by this phase - see `ai_agent_foundry.rs`'s own coverage of
//! `update_memory`.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_policy::AiAgentPolicyInput;
use lanesra_core::models::ai_memory::{AgentMemoryContext, MemoryItemInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_memory_service, ai_service, chat_service, policy_engine_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Memory Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [97u8; 32]
}

fn configure_anthropic_key(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap();
}

fn anthropic_text_body(text: &str) -> String {
    serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string()
}

fn anthropic_tool_use_body(id: &str, name: &str, input: serde_json::Value) -> String {
    serde_json::json!({"content": [{"type": "tool_use", "id": id, "name": name, "input": input}]}).to_string()
}

fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> lanesra_core::models::ai_agent::AiAgentDefinition {
    ai_agent_service::create(
        conn, ws,
        &AiAgentInput { name: name.into(), description: None, icon: "🤖".into(), system_prompt: format!("You are {name}."), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(admin),
    )
    .unwrap()
}

fn memory_input(memory_type: &str, entity_type: Option<&str>, entity_id: Option<&str>, content: &str, classification: &str, ttl_seconds: Option<i64>) -> MemoryItemInput {
    MemoryItemInput {
        memory_type: memory_type.into(),
        entity_type: entity_type.map(String::from),
        entity_id: entity_id.map(String::from),
        content: content.into(),
        source: "agent_inference".into(),
        confidence: None,
        classification: classification.into(),
        ttl_seconds,
    }
}

/// Same raw-socket stub `ai_orchestration_topologies.rs` uses - serves one
/// canned response body per request in order (repeating the last once
/// exhausted), capturing each request's raw JSON body so a test can assert
/// exactly what each outbound request carried.
fn spawn_sequence_stub(bodies: Vec<String>) -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let queue = Arc::new(Mutex::new(VecDeque::from(bodies)));
    let captured = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = captured.clone();
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
            captured_clone.lock().unwrap().push(String::from_utf8_lossy(&body_buf).to_string());
            let body = {
                let mut q = queue.lock().unwrap();
                if q.len() > 1 { q.pop_front().unwrap() } else { q.front().cloned().unwrap_or_default() }
            };
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, captured)
}

// --- Direct service-level coverage ------------------------------------

#[test]
fn entity_memory_remember_list_and_forget_round_trip() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Rememberer");
    let ctx = AgentMemoryContext::default();

    let input = memory_input("entity", Some("Company"), Some("acme-1"), "Prefers phone calls.", "standard", None);
    let item = ai_memory_service::remember(&conn, &ws, &agent.id, &input, &ctx, "agent").unwrap();
    assert_eq!(item.memory_type, "entity");
    assert!(item.expires_at.is_none(), "Entity Memory has no TTL by default: {item:?}");

    let items = ai_memory_service::list_context(&conn, &agent.id, "entity", Some("Company"), Some("acme-1"), &ctx).unwrap();
    assert_eq!(items.len(), 1, "{items:?}");
    assert_eq!(items[0].content, "Prefers phone calls.");

    ai_memory_service::forget(&conn, &ws, &item.id, Some(&admin)).unwrap();
    let items = ai_memory_service::list_context(&conn, &agent.id, "entity", Some("Company"), Some("acme-1"), &ctx).unwrap();
    assert!(items.is_empty(), "{items:?}");
}

#[test]
fn working_memory_is_rejected_outside_a_real_run_context() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Worker");
    let ctx = AgentMemoryContext::default();
    let input = memory_input("working", None, None, "step complete", "standard", None);
    let err = ai_memory_service::remember(&conn, &ws, &agent.id, &input, &ctx, "agent").unwrap_err();
    assert!(err.to_string().contains("Pipeline or Execution Graph run"), "{err}");
}

#[test]
fn session_memory_is_rejected_outside_a_real_chat_session_context() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Chatter");
    let ctx = AgentMemoryContext::default();
    let input = memory_input("session", None, None, "prefers email", "standard", None);
    let err = ai_memory_service::remember(&conn, &ws, &agent.id, &input, &ctx, "agent").unwrap_err();
    assert!(err.to_string().contains("interactive chat"), "{err}");
}

#[test]
fn entity_memory_requires_both_entity_type_and_entity_id() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Rememberer");
    let ctx = AgentMemoryContext::default();
    let input = memory_input("entity", Some("Company"), None, "a fact", "standard", None);
    let err = ai_memory_service::remember(&conn, &ws, &agent.id, &input, &ctx, "agent").unwrap_err();
    assert!(err.to_string().contains("entity_type and entity_id"), "{err}");
}

#[test]
fn restricted_classification_is_excluded_by_the_default_policy() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Guarded");
    let ctx = AgentMemoryContext::default();
    let input = memory_input("entity", Some("Contact"), Some("c1"), "SSN 123-45-6789", "restricted", None);
    let err = ai_memory_service::remember(&conn, &ws, &agent.id, &input, &ctx, "agent").unwrap_err();
    assert!(err.to_string().contains("restricted"), "{err}");
}

#[test]
fn restricted_classification_is_allowed_once_a_policy_relaxes_the_exclusion() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Guarded");
    policy_engine_service::upsert_policy(
        &conn, &ws, Some(&agent.id),
        &AiAgentPolicyInput { require_approval_at_or_above: None, blocked_tool_names: vec![], exclude_restricted_memory: false },
        Some(&admin),
    )
    .unwrap();
    let ctx = AgentMemoryContext::default();
    let input = memory_input("entity", Some("Contact"), Some("c1"), "SSN 123-45-6789", "restricted", None);
    let item = ai_memory_service::remember(&conn, &ws, &agent.id, &input, &ctx, "agent").unwrap();
    assert_eq!(item.classification, "restricted");
}

#[test]
fn sweep_expired_removes_only_items_past_their_expires_at() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Sweeper");
    let ctx = AgentMemoryContext { session_key: Some("sess-1".into()), run_id: None };
    let input = memory_input("session", None, None, "temporary note", "standard", Some(3600));
    let item = ai_memory_service::remember(&conn, &ws, &agent.id, &input, &ctx, "agent").unwrap();
    assert!(item.expires_at.is_some(), "{item:?}");

    // Force it safely into the past rather than relying on real time
    // passing - same convention execution_graph_runtime.rs's delay test
    // already uses.
    conn.execute("UPDATE ai_memory_items SET expires_at = '2020-01-01T00:00:00Z' WHERE id = ?1", [&item.id]).unwrap();

    let swept = ai_memory_service::sweep_expired(&conn, &ws).unwrap();
    assert_eq!(swept, 1);
    let items = ai_memory_service::list_context(&conn, &agent.id, "session", None, None, &ctx).unwrap();
    assert!(items.is_empty(), "{items:?}");
}

#[test]
fn admin_memory_inspector_lists_across_types_and_forget_requires_an_administrator() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Inspected");
    let ctx = AgentMemoryContext::default();
    let entity_input = memory_input("entity", Some("Company"), Some("acme-1"), "Prefers phone calls.", "standard", None);
    let item = ai_memory_service::remember(&conn, &ws, &agent.id, &entity_input, &ctx, "agent").unwrap();

    let all = ai_memory_service::list_all(&conn, &ws, None, None, None, Some(&admin)).unwrap();
    assert_eq!(all.len(), 1, "{all:?}");
    let entity_only = ai_memory_service::list_all(&conn, &ws, Some("entity"), None, None, Some(&admin)).unwrap();
    assert_eq!(entity_only.len(), 1);
    let session_only = ai_memory_service::list_all(&conn, &ws, Some("session"), None, None, Some(&admin)).unwrap();
    assert!(session_only.is_empty());

    // No actor at all - rejected before any role check even runs.
    assert!(ai_memory_service::forget(&conn, &ws, &item.id, None).is_err());

    // A real, non-Administrator user - rejected on the role check itself.
    let non_admin = lanesra_core::services::user_service::create(
        &conn, &ws,
        &lanesra_core::models::user::NewUser { username: "regular".into(), display_name: "Regular User".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    )
    .unwrap();
    let err = ai_memory_service::forget(&conn, &ws, &item.id, Some(&non_admin.id)).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("administrator"), "{err}");

    ai_memory_service::forget(&conn, &ws, &item.id, Some(&admin)).unwrap();
    assert!(ai_memory_service::list_all(&conn, &ws, None, None, None, Some(&admin)).unwrap().is_empty());
}

// --- Real chat_service tool wiring --------------------------------------

#[tokio::test]
async fn remember_and_get_memory_tools_round_trip_for_entity_memory() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Rememberer");

    let (port, captured) = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "remember", serde_json::json!({"memory_type": "entity", "entity_type": "Company", "entity_id": "acme-1", "content": "Prefers phone calls."})),
        anthropic_tool_use_body("t2", "get_memory", serde_json::json!({"memory_type": "entity", "entity_type": "Company", "entity_id": "acme-1"})),
        anthropic_text_body("Noted - Acme Corp prefers phone calls."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "Remember something about Acme Corp.").await.unwrap();

    let requests = captured.lock().unwrap();
    assert_eq!(requests.len(), 3, "{requests:?}");
    // Round 3's outbound request carries round 2's get_memory tool result
    // in its history - proving the real fact remembered in round 1
    // actually flowed through storage and back out again, not just that
    // the tool calls didn't error.
    assert!(requests[2].contains("Prefers phone calls."), "{}", requests[2]);
}

#[tokio::test]
async fn session_memory_is_injected_ambiently_into_a_later_conversation_turns_system_prompt() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Rememberer");

    let (port, captured) = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "remember", serde_json::json!({"memory_type": "session", "content": "Prefers email over phone."})),
        anthropic_text_body("Got it, I'll remember that."),
        anthropic_text_body("You prefer email."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "Please remember I prefer email.").await.unwrap();
    chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "What do you remember about me?").await.unwrap();

    let requests = captured.lock().unwrap();
    assert_eq!(requests.len(), 3, "{requests:?}");
    assert!(!requests[0].contains("Prefers email over phone."), "the very first turn's own system prompt predates the remember call: {}", requests[0]);
    assert!(requests[2].contains("Prefers email over phone."), "a later turn's system prompt should ambiently include the remembered session note: {}", requests[2]);
}
