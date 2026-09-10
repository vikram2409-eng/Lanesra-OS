//! AI & Agentic Layer, Phase 6a: the AI Agent Foundry's Agents/Skills
//! CRUD, chat with a named Agent (`chat_service::send_agent_message`),
//! Memory, Skills, and delegation/hierarchy - end to end against a real
//! local HTTP listener standing in for the LLM provider, same
//! raw-socket test-double pattern `chat.rs`/`agent_reporting.rs` already
//! use. Unlike those, a couple of tests here need to inspect what was
//! actually *sent* to the provider (to prove memory made it into the
//! next round's system prompt), so `spawn_sequence_stub` below also
//! captures each request's raw body, not just the response queue.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::{AiAgentInput, AiSkillInput};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_service, chat_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Foundry Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [22u8; 32]
}

fn configure_anthropic_key(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap();
}

fn anthropic_tool_use_body(id: &str, name: &str, input: serde_json::Value) -> String {
    serde_json::json!({"content": [{"type": "tool_use", "id": id, "name": name, "input": input}]}).to_string()
}

fn anthropic_text_body(text: &str) -> String {
    serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string()
}

fn non_admin_user(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    user_service::create(
        conn, ws,
        &NewUser { username: "rep".into(), display_name: "Sales Rep".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] },
        Some(admin),
    )
    .unwrap()
    .id
}

/// Serves one canned `/v1/messages` response body per incoming
/// connection, in order (repeating the last one once exhausted) - see
/// `chat.rs`'s own identical stub for why. Also captures each request's
/// raw JSON body, so a test can assert what was actually sent (e.g. that
/// an agent's memory made it into a later round's system prompt).
fn spawn_sequence_stub(bodies: Vec<String>) -> (u16, Arc<Mutex<u32>>, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let queue = Arc::new(Mutex::new(VecDeque::from(bodies)));
    let count = Arc::new(Mutex::new(0u32));
    let captured = Arc::new(Mutex::new(Vec::new()));
    let count_clone = count.clone();
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
            *count_clone.lock().unwrap() += 1;
            let body = {
                let mut q = queue.lock().unwrap();
                if q.len() > 1 { q.pop_front().unwrap() } else { q.front().cloned().unwrap_or_default() }
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, count, captured)
}

fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str, action_names: Vec<String>) -> lanesra_core::models::ai_agent::AiAgentDefinition {
    ai_agent_service::create(
        conn, ws,
        &AiAgentInput {
            name: name.into(), description: None, icon: "🤖".into(), system_prompt: format!("You are {name}."),
            action_names, delegate_agent_ids: vec![], skill_ids: vec![],
        },
        Some(admin),
    )
    .unwrap()
}

#[tokio::test]
async fn agent_and_skill_crud_and_non_admin_rejected_on_create() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);

    let agent_input = AiAgentInput {
        name: "Helper".into(), description: Some("A helper".into()), icon: "🤖".into(), system_prompt: "Be helpful.".into(),
        action_names: vec!["list_records".into()], delegate_agent_ids: vec![], skill_ids: vec![],
    };
    let denied = ai_agent_service::create(&conn, &ws, &agent_input, Some(&rep));
    assert!(denied.unwrap_err().to_string().contains("Administrator"));

    let agent = ai_agent_service::create(&conn, &ws, &agent_input, Some(&admin)).unwrap();
    assert_eq!(agent.name, "Helper");

    let skill_input = AiSkillInput { name: "Checklist".into(), description: "A checklist".into(), instructions_md: "Do the checklist.".into() };
    let denied_skill = ai_agent_service::create_skill(&conn, &ws, &skill_input, Some(&rep));
    assert!(denied_skill.unwrap_err().to_string().contains("Administrator"));
    let skill = ai_agent_service::create_skill(&conn, &ws, &skill_input, Some(&admin)).unwrap();

    let agents = ai_agent_service::list(&conn, &ws, true).unwrap();
    assert!(agents.iter().any(|a| a.id == agent.id));
    let skills = ai_agent_service::list_skills(&conn, &ws, true).unwrap();
    assert!(skills.iter().any(|s| s.id == skill.id));
}

#[tokio::test]
async fn a_record_only_agent_is_usable_by_a_non_administrator() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);
    let agent = make_agent(&conn, &ws, &admin, "Records Helper", vec!["list_records".into()]);

    let (port, _count, _captured) = spawn_sequence_stub(vec![anthropic_text_body("Hi there!")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let appended = chat_service::send_agent_message(&conn, &ws, &master_key(), &rep, &agent.id, "hello").await.unwrap();
    assert_eq!(appended.last().unwrap().content.as_deref(), Some("Hi there!"));
}

#[tokio::test]
async fn an_agent_with_any_admin_action_requires_administrator() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);
    let agent = make_agent(&conn, &ws, &admin, "Admin Helper", vec!["list_business_rules".into()]);

    // No AI provider key configured at all - reaching ai_service would
    // fail for that reason instead, so getting the admin-only rejection
    // proves the gate runs before any network call.
    let result = chat_service::send_agent_message(&conn, &ws, &master_key(), &rep, &agent.id, "hello").await;
    assert!(result.unwrap_err().to_string().contains("Administrator"));
}

#[tokio::test]
async fn update_memory_persists_and_is_included_in_the_next_rounds_request() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Memory Keeper", vec![]);

    let (port, _count, captured) = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "update_memory", serde_json::json!({"content": "The user prefers short answers."})),
        anthropic_text_body("Got it, noted for next time."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "remember I like short answers").await.unwrap();

    let stored = ai_agent_service::get(&conn, &agent.id).unwrap().unwrap();
    assert_eq!(stored.memory_md, "The user prefers short answers.");

    // A second message - this round's outbound request should now carry
    // the memory in its system prompt.
    let (port2, _count2, captured2) = spawn_sequence_stub(vec![anthropic_text_body("Sure, short answer.")]);
    configure_anthropic_key(&conn, &ws, &admin, port2);
    chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "another question").await.unwrap();

    let requests = captured2.lock().unwrap();
    assert!(requests[0].contains("The user prefers short answers."), "memory missing from request: {}", requests[0]);
    drop(requests);
    let _ = captured; // first stub's captures aren't needed further
}

#[tokio::test]
async fn an_attached_skills_use_skill_tool_returns_its_full_instructions() {
    let (conn, ws, admin) = setup_workspace();
    let skill = ai_agent_service::create_skill(
        &conn, &ws,
        &AiSkillInput { name: "Weekly Hygiene".into(), description: "Flags stale records".into(), instructions_md: "Step 1: find records untouched 30+ days. Step 2: flag them.".into() },
        Some(&admin),
    )
    .unwrap();
    let agent = ai_agent_service::create(
        &conn, &ws,
        &AiAgentInput { name: "Hygiene Bot".into(), description: None, icon: "🤖".into(), system_prompt: "Keep records tidy.".into(), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![skill.id.clone()] },
        Some(&admin),
    )
    .unwrap();

    let (port, _count, _captured) = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "use_skill", serde_json::json!({"name": "Weekly Hygiene"})),
        anthropic_text_body("Done - followed the checklist."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let appended = chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "run the hygiene checklist").await.unwrap();
    let tool_msg = appended.iter().find(|m| m.role == "tool").expect("expected a tool result message");
    let content = tool_msg.content.as_deref().unwrap_or_default();
    assert!(content.contains("Step 1: find records untouched"), "{content}");
}

#[tokio::test]
async fn delegation_runs_a_sub_agent_and_returns_its_answer() {
    let (conn, ws, admin) = setup_workspace();
    let helper = make_agent(&conn, &ws, &admin, "Helper", vec![]);
    let supervisor = ai_agent_service::create(
        &conn, &ws,
        &AiAgentInput { name: "Supervisor".into(), description: None, icon: "🤖".into(), system_prompt: "Delegate math questions.".into(), action_names: vec![], delegate_agent_ids: vec![helper.id.clone()], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();

    let (port, _count, _captured) = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "delegate_to_agent", serde_json::json!({"agent_name": "Helper", "input": "what is 2+2?"})),
        anthropic_text_body("4"),
        anthropic_text_body("The Helper says the answer is 4."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let appended = chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &supervisor.id, "what's 2+2? ask the helper").await.unwrap();
    assert_eq!(appended.last().unwrap().content.as_deref(), Some("The Helper says the answer is 4."));
    let tool_msg = appended.iter().find(|m| m.role == "tool").expect("expected the delegate_to_agent tool result");
    assert!(tool_msg.content.as_deref().unwrap_or_default().contains('4'));
}

#[tokio::test]
async fn the_delegation_depth_guard_stops_a_chain_with_a_tool_error_not_a_crash() {
    let (conn, ws, admin) = setup_workspace();
    // A straight-line chain of 5 distinct agents, A1 -> A2 -> A3 -> A4 ->
    // A5 - not a cycle, so the FIFO request order is unambiguous: each
    // level's guard is acquired at depths 1..5 in order, and
    // MAX_DELEGATION_DEPTH (4) means A5's own attempt (the 5th nested
    // `run_agent_once` call) is rejected *before* it ever makes a
    // request - A4's own delegate_to_agent(A5) call gets back a tool
    // error instead. Built back-to-front since each agent's
    // `delegate_agent_ids` must name an already-existing agent.
    let a5 = make_agent(&conn, &ws, &admin, "A5", vec![]);
    let a4 = ai_agent_service::create(&conn, &ws, &agent_input("A4", vec![a5.id.clone()]), Some(&admin)).unwrap();
    let a3 = ai_agent_service::create(&conn, &ws, &agent_input("A3", vec![a4.id.clone()]), Some(&admin)).unwrap();
    let a2 = ai_agent_service::create(&conn, &ws, &agent_input("A2", vec![a3.id.clone()]), Some(&admin)).unwrap();
    let a1 = ai_agent_service::create(&conn, &ws, &agent_input("A1", vec![a2.id.clone()]), Some(&admin)).unwrap();

    // FIFO order: A1's request, A2's, A3's, A4's (all tool_use, each
    // delegating one level deeper) - A5 never gets a turn, so no 5th
    // tool_use is needed - then A4's own next round (a final answer,
    // since its delegate attempt errored), unwinding back up through
    // A3's, A2's and finally A1's own final round. 8 requests total; a
    // 9th would mean A5 incorrectly got to run.
    let bodies = vec![
        anthropic_tool_use_body("t1", "delegate_to_agent", serde_json::json!({"agent_name": "A2", "input": "go"})),
        anthropic_tool_use_body("t2", "delegate_to_agent", serde_json::json!({"agent_name": "A3", "input": "go"})),
        anthropic_tool_use_body("t3", "delegate_to_agent", serde_json::json!({"agent_name": "A4", "input": "go"})),
        anthropic_tool_use_body("t4", "delegate_to_agent", serde_json::json!({"agent_name": "A5", "input": "go"})),
        anthropic_text_body("A4: hit my depth limit, answering directly"),
        anthropic_text_body("A3: relaying A4's answer"),
        anthropic_text_body("A2: relaying A3's answer"),
        anthropic_text_body("Final answer from A1"),
    ];
    let (port, count, _captured) = spawn_sequence_stub(bodies);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let appended = chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &a1.id, "start the chain").await.unwrap();
    assert_eq!(appended.last().unwrap().content.as_deref(), Some("Final answer from A1"));
    // Exactly 8, not 9+ - proves A5's own frame never actually ran (the
    // depth guard rejected it before any network call), and not fewer -
    // proves the chain unwound all the way back to A1 rather than
    // erroring out partway.
    assert_eq!(*count.lock().unwrap(), 8);
}

fn agent_input(name: &str, delegate_agent_ids: Vec<String>) -> AiAgentInput {
    AiAgentInput { name: name.into(), description: None, icon: "🤖".into(), system_prompt: format!("You are {name}."), action_names: vec![], delegate_agent_ids, skill_ids: vec![] }
}

#[tokio::test]
async fn agent_conversation_history_persists_and_is_reused_across_calls() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Chatty", vec![]);

    let (port1, _c1, _cap1) = spawn_sequence_stub(vec![anthropic_text_body("Hi there!")]);
    configure_anthropic_key(&conn, &ws, &admin, port1);
    chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "hello").await.unwrap();

    let (port2, _c2, _cap2) = spawn_sequence_stub(vec![anthropic_text_body("Hi again!")]);
    configure_anthropic_key(&conn, &ws, &admin, port2);
    chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "hello again").await.unwrap();

    let history = chat_service::get_agent_history(&conn, &ws, &admin, &agent.id).unwrap();
    assert_eq!(history.len(), 4, "{history:?}");
    assert_eq!(history[0].content.as_deref(), Some("hello"));
    assert_eq!(history[1].content.as_deref(), Some("Hi there!"));
    assert_eq!(history[2].content.as_deref(), Some("hello again"));
    assert_eq!(history[3].content.as_deref(), Some("Hi again!"));
}
