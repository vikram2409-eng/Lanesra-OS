//! AI & Agentic Layer, Phase 5: proves `chat_service::send_message`'s
//! tool-calling loop end to end against a real local HTTP listener
//! standing in for the LLM provider (same raw-socket test-double pattern
//! `agent_reporting.rs`/`ai_settings.rs` already use, not a live
//! account). Unlike those single-shot stubs, `send_message` can make
//! several HTTP requests in one call (one per round), so `spawn_sequence_stub`
//! below serves one scripted Anthropic-shaped response body per request,
//! in order.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_service, chat_service, company_service, connection_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Chat Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [11u8; 32]
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

/// Serves one canned `/v1/messages` response body per incoming connection,
/// in order - `send_message`'s loop makes one HTTP request per round, so a
/// scripted multi-round exchange (tool_use, tool_use, then a final text
/// reply) is expressed as a queue here, unlike `agent_reporting.rs`'s
/// single-shot stub which only ever needs one reply. The last body in the
/// queue repeats for any request beyond the queue's length - a pathological
/// "always calls a tool" stub for the `MAX_ROUNDS` test uses this to keep
/// answering forever. Also returns a shared request counter so a test can
/// assert exactly how many rounds actually ran.
fn spawn_sequence_stub(bodies: Vec<String>) -> (u16, Arc<Mutex<u32>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let queue = Arc::new(Mutex::new(VecDeque::from(bodies)));
    let count = Arc::new(Mutex::new(0u32));
    let count_clone = count.clone();
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
    (port, count)
}

#[tokio::test]
async fn records_mode_creates_a_company_then_lists_it_before_answering() {
    let (conn, ws, admin) = setup_workspace();
    let (port, _count) = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "create_record", serde_json::json!({"object_key": "Company", "data": {"name": "Acme Corp", "status": "Prospect"}})),
        anthropic_tool_use_body("t2", "list_records", serde_json::json!({"object_key": "Company"})),
        anthropic_text_body("Created Acme Corp and confirmed it's in your company list."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let appended = chat_service::send_message(&conn, &ws, &master_key(), &admin, "records", "create a company called Acme Corp, then list companies").await.unwrap();
    let last = appended.last().unwrap();
    assert_eq!(last.role, "assistant");
    assert!(last.content.as_deref().unwrap().contains("Acme Corp"));

    let companies = company_service::list(&conn, &ws).unwrap();
    assert!(companies.iter().any(|c| c.name == "Acme Corp"), "expected a real Company row, got {companies:?}");
}

#[tokio::test]
async fn admin_mode_creates_a_real_business_rule_via_chat() {
    let (conn, ws, admin) = setup_workspace();
    let (port, _count) = spawn_sequence_stub(vec![
        anthropic_tool_use_body(
            "t1", "create_business_rule",
            serde_json::json!({
                "entity_type": "Company", "name": "Chat-built rule", "description": null, "match_type": "all", "priority": 0,
                "effective_start_date": null, "effective_end_date": null,
                "conditions": [{"field_source": "builtin", "field_key": "status", "operator": "equals", "value": "Prospect"}],
                "actions": [{"action_type": "show_error", "target_field_key": null, "target_field_source": "custom", "action_value": null, "message": "Chat-built rule fired"}],
            }),
        ),
        anthropic_text_body("Created the business rule 'Chat-built rule'."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    chat_service::send_message(&conn, &ws, &master_key(), &admin, "admin", "create a business rule named 'Chat-built rule' on Company with no conditions").await.unwrap();

    let rules = lanesra_core::services::business_rule_service::list_rules(&conn, &ws, "Company", true).unwrap();
    assert!(rules.iter().any(|r| r.name == "Chat-built rule"), "expected a real BusinessRule row, got {rules:?}");
}

#[tokio::test]
async fn admin_mode_creates_a_real_workflow_via_chat() {
    let (conn, ws, admin) = setup_workspace();
    let (port, _count) = spawn_sequence_stub(vec![
        anthropic_tool_use_body(
            "t1", "create_workflow",
            serde_json::json!({
                "entity_type": "Company", "name": "Chat-built workflow", "description": null,
                "trigger_type": "record_created", "trigger_status": null, "trigger_field_key": null,
                "trigger_field_source": "builtin", "trigger_offset_days": 0, "match_type": "all", "priority": 0,
                "conditions": [],
                "actions": [{"action_type": "create_task", "params_json": "{\"title\":\"Follow up\",\"due_in_days\":1}"}],
            }),
        ),
        anthropic_text_body("Created the workflow 'Chat-built workflow'."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    chat_service::send_message(&conn, &ws, &master_key(), &admin, "admin", "create a workflow named 'Chat-built workflow' that fires when a Company is created").await.unwrap();

    let workflows = lanesra_core::services::workflow_service::list_rules(&conn, &ws, "Company", Some(&admin)).unwrap();
    assert!(workflows.iter().any(|w| w.name == "Chat-built workflow"), "expected a real WorkflowDefinition row, got {workflows:?}");
}

#[tokio::test]
async fn a_non_administrator_is_rejected_before_any_tool_runs() {
    let (conn, ws, admin) = setup_workspace();
    let standard_user = user_service::create(
        &conn, &ws,
        &NewUser { username: "rep".into(), display_name: "Sales Rep".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    )
    .unwrap();

    // No AI provider key configured at all - if this reached ai_service it
    // would fail for that reason instead, so getting the admin-only
    // rejection here (not a "configure a key" error) proves require_admin
    // runs first, before any tool or network call.
    let result = chat_service::send_message(&conn, &ws, &master_key(), &standard_user.id, "admin", "create a business rule").await;
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Administrator"), "{err}");
}

#[tokio::test]
async fn a_connections_secret_is_created_for_real_but_never_persisted_in_chat_history() {
    let (conn, ws, admin) = setup_workspace();
    let secret = "sk-super-secret-token-should-never-be-stored-in-chat";
    let (port, _count) = spawn_sequence_stub(vec![
        anthropic_tool_use_body(
            "t1", "create_connection",
            serde_json::json!({
                "name": "Chat-built connection", "connection_type": "rest", "base_url": "https://api.example.com",
                "auth_mode": "api_key", "secret_value": secret, "config_json": "{}",
            }),
        ),
        anthropic_text_body("Created the Connection 'Chat-built connection'. Add its credential via Integration Hub -> Connections -> Edit."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    // The user's own message deliberately never mentions the secret - the
    // secret only ever shows up as the model's own tool-call argument
    // (scripted above), which is what dispatch_admin_tool must strip. If
    // the user's own typed message contained the secret, its persisted
    // content would legitimately contain it too - that's not what this
    // boundary is about.
    chat_service::send_message(&conn, &ws, &master_key(), &admin, "admin", "create a REST connection called 'Chat-built connection'").await.unwrap();

    // The Connection is real...
    let connections = connection_service::list_for_workspace(&conn, &ws).unwrap();
    assert!(connections.iter().any(|c| c.name == "Chat-built connection"), "expected a real Connection row, got {connections:?}");

    // ...but the secret is nowhere in the persisted transcript, even though
    // the model itself supplied it as a tool argument.
    let history = chat_service::get_history(&conn, &ws, &admin, "admin").unwrap();
    for m in &history {
        if let Some(content) = &m.content {
            assert!(!content.contains(secret), "secret leaked into a chat message's content: {content}");
        }
        if let Some(tool_calls) = &m.tool_calls {
            let raw = tool_calls.to_string();
            assert!(!raw.contains(secret), "secret leaked into a chat message's tool_calls: {raw}");
        }
    }
}

#[tokio::test]
async fn the_round_cap_stops_a_pathological_always_tool_calling_stub() {
    let (conn, ws, admin) = setup_workspace();
    // Always answers with the same harmless, argument-free tool call -
    // never a final text reply - so the loop must give up on its own via
    // MAX_ROUNDS rather than running forever.
    let (port, count) = spawn_sequence_stub(vec![anthropic_tool_use_body("t", "list_objects", serde_json::json!({}))]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let result = chat_service::send_message(&conn, &ws, &master_key(), &admin, "records", "loop forever").await;
    let err = result.unwrap_err().to_string();
    assert!(err.contains("more steps"), "{err}");
    // One HTTP request per round, capped at MAX_ROUNDS (8) - not unbounded.
    assert_eq!(*count.lock().unwrap(), 8);
}

#[tokio::test]
async fn conversation_history_persists_and_is_reused_across_calls() {
    let (conn, ws, admin) = setup_workspace();

    let (port1, _c1) = spawn_sequence_stub(vec![anthropic_text_body("Hi there!")]);
    configure_anthropic_key(&conn, &ws, &admin, port1);
    chat_service::send_message(&conn, &ws, &master_key(), &admin, "records", "hello").await.unwrap();

    let (port2, _c2) = spawn_sequence_stub(vec![anthropic_text_body("Hi again!")]);
    configure_anthropic_key(&conn, &ws, &admin, port2);
    chat_service::send_message(&conn, &ws, &master_key(), &admin, "records", "hello again").await.unwrap();

    let history = chat_service::get_history(&conn, &ws, &admin, "records").unwrap();
    assert_eq!(history.len(), 4, "{history:?}");
    assert_eq!(history[0].role, "user");
    assert_eq!(history[0].content.as_deref(), Some("hello"));
    assert_eq!(history[1].role, "assistant");
    assert_eq!(history[1].content.as_deref(), Some("Hi there!"));
    assert_eq!(history[2].role, "user");
    assert_eq!(history[2].content.as_deref(), Some("hello again"));
    assert_eq!(history[3].role, "assistant");
    assert_eq!(history[3].content.as_deref(), Some("Hi again!"));
}

#[tokio::test]
async fn an_empty_message_is_rejected_before_any_network_call() {
    let (conn, ws, admin) = setup_workspace();
    // No key configured at all - if this reached ai_service::complete_with_tools
    // it would fail for that reason instead, so succeeding here proves the
    // empty-message check runs first.
    let result = chat_service::send_message(&conn, &ws, &master_key(), &admin, "records", "   ").await;
    let err = result.unwrap_err().to_string();
    assert!(err.contains("something"), "{err}");
}
