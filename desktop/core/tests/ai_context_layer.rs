//! AI & Agentic Layer, Phase 7b: the Context Layer - memory history
//! (`ai_agent_repo::update_memory`'s snapshot-before-overwrite behavior)
//! and ranked full-text search over Custom Object records
//! (`search_service::search_custom_records`, backed by the
//! `record_search_fts` FTS5 index from migration 0042). Reuses
//! `ai_agent_foundry.rs`'s own stub-listener/captured-request-body
//! pattern for the tool round-trip test, and `search.rs`'s own custom-
//! object/custom-field builders for the search tests.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_service, chat_service, custom_field_service, custom_object_service, custom_record_service, search_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Context Layer Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [71u8; 32]
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

/// Same raw-socket stub as `ai_agent_foundry.rs`'s own `spawn_sequence_stub`
/// - one canned `/v1/messages` response per request, in order (repeating
/// the last once exhausted), capturing each request's raw body so a test
/// can assert a tool's result actually reached the model.
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
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, captured)
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

fn asset_object_input() -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: "Asset".into(), plural_label: "Assets".into(), icon: "🔧".into(), prefix: "AST".into(), digits: 4 }
}

// --- Memory history ---------------------------------------------------

#[test]
fn no_history_is_written_for_the_first_ever_memory_write() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Memory Keeper", vec![]);

    ai_agent_service::set_memory(&conn, &agent.id, "First memory ever.", Some(&admin)).unwrap();

    let history = ai_agent_service::list_memory_history(&conn, &agent.id, Some(&admin)).unwrap();
    assert!(history.is_empty(), "a write from empty memory shouldn't create a snapshot of nothing");
}

#[test]
fn each_subsequent_admin_edit_snapshots_the_prior_value() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Memory Keeper", vec![]);

    ai_agent_service::set_memory(&conn, &agent.id, "v1".into(), Some(&admin)).unwrap();
    ai_agent_service::set_memory(&conn, &agent.id, "v2".into(), Some(&admin)).unwrap();
    ai_agent_service::set_memory(&conn, &agent.id, "v3".into(), Some(&admin)).unwrap();

    let history = ai_agent_service::list_memory_history(&conn, &agent.id, Some(&admin)).unwrap();
    // Most-recent-first: the last snapshot taken was v2 (just before the
    // v2 -> v3 overwrite), the first was v1 (just before v1 -> v2).
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].memory_md, "v2");
    assert_eq!(history[0].changed_by, admin);
    assert_eq!(history[1].memory_md, "v1");

    let stored = ai_agent_service::get(&conn, &agent.id).unwrap().unwrap();
    assert_eq!(stored.memory_md, "v3");
}

#[test]
fn a_no_op_write_of_the_same_value_does_not_add_a_snapshot() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Memory Keeper", vec![]);

    ai_agent_service::set_memory(&conn, &agent.id, "steady state".into(), Some(&admin)).unwrap();
    ai_agent_service::set_memory(&conn, &agent.id, "steady state".into(), Some(&admin)).unwrap();

    let history = ai_agent_service::list_memory_history(&conn, &agent.id, Some(&admin)).unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn the_agents_own_update_memory_tool_is_snapshotted_as_changed_by_agent() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Memory Keeper", vec![]);
    ai_agent_service::set_memory(&conn, &agent.id, "seeded by admin".into(), Some(&admin)).unwrap();

    let (port, _captured) = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "update_memory", serde_json::json!({"content": "revised by the agent itself"})),
        anthropic_text_body("Noted."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "remember this instead").await.unwrap();

    let history = ai_agent_service::list_memory_history(&conn, &agent.id, Some(&admin)).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].memory_md, "seeded by admin");
    assert_eq!(history[0].changed_by, "agent");
}

// --- Ranked full-text search over Custom Object records ----------------

#[test]
fn search_custom_records_ranks_by_relevance_and_is_workspace_scoped() {
    let (conn, ws, admin) = setup_workspace();
    let asset_def = custom_object_service::create(&conn, &ws, &asset_object_input(), Some(&admin)).unwrap();
    let notes_field =
        custom_field_service::create_definition(&conn, &ws, &searchable_text_field(&asset_def.key, "Notes", true), Some(&admin)).unwrap();

    let leaking = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: asset_def.key.clone(), primary_name: "Rooftop HVAC Unit 3".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    )
    .unwrap();
    let mut values = HashMap::new();
    values.insert(notes_field.key.clone(), "Reported leaking coolant near the compressor".into());
    custom_field_service::set_entity_values(&conn, &asset_def.key, &leaking.id, &values, Some(&admin)).unwrap();

    let quiet = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: asset_def.key.clone(), primary_name: "Lobby Water Fountain".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    )
    .unwrap();

    let hits = search_service::search_custom_records(&conn, &ws, "leaking coolant", None, 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].record_id, leaking.id);
    assert_eq!(hits[0].object_key, asset_def.key);
    assert_eq!(hits[0].title, "Rooftop HVAC Unit 3");
    assert!(hits[0].snippet.contains("leaking"));

    // A term that isn't in either record - no results.
    assert!(search_service::search_custom_records(&conn, &ws, "solar panel", None, 10).unwrap().is_empty());
    let _ = quiet;

    // Scoped to a different (non-existent, for this workspace) object_key -
    // no results even though the term matches globally.
    assert!(search_service::search_custom_records(&conn, &ws, "leaking coolant", Some("nonexistent_object"), 10).unwrap().is_empty());
}

#[test]
fn archiving_a_record_removes_it_from_search() {
    let (conn, ws, admin) = setup_workspace();
    let asset_def = custom_object_service::create(&conn, &ws, &asset_object_input(), Some(&admin)).unwrap();
    let asset = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: asset_def.key.clone(), primary_name: "Basement Generator".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(search_service::search_custom_records(&conn, &ws, "generator", None, 10).unwrap().len(), 1);

    custom_record_service::archive(&conn, &asset.id, Some(&admin)).unwrap();
    assert!(search_service::search_custom_records(&conn, &ws, "generator", None, 10).unwrap().is_empty());
}

#[test]
fn a_non_searchable_field_value_is_never_matched() {
    let (conn, ws, admin) = setup_workspace();
    let asset_def = custom_object_service::create(&conn, &ws, &asset_object_input(), Some(&admin)).unwrap();
    let internal_field =
        custom_field_service::create_definition(&conn, &ws, &searchable_text_field(&asset_def.key, "Internal Code", false), Some(&admin)).unwrap();
    let asset = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: asset_def.key.clone(), primary_name: "Server Rack B".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    )
    .unwrap();
    let mut values = HashMap::new();
    values.insert(internal_field.key.clone(), "quantumfluxcapacitor".into());
    custom_field_service::set_entity_values(&conn, &asset_def.key, &asset.id, &values, Some(&admin)).unwrap();

    assert!(search_service::search_custom_records(&conn, &ws, "quantumfluxcapacitor", None, 10).unwrap().is_empty());
}

#[tokio::test]
async fn search_records_tool_result_reaches_the_model() {
    let (conn, ws, admin) = setup_workspace();
    let asset_def = custom_object_service::create(&conn, &ws, &asset_object_input(), Some(&admin)).unwrap();
    custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: asset_def.key.clone(), primary_name: "Rooftop Chiller Unit 9".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    )
    .unwrap();

    let agent = make_agent(&conn, &ws, &admin, "Facilities Bot", vec!["search_records".into()]);
    let (port, captured) = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "search_records", serde_json::json!({"query": "chiller"})),
        anthropic_text_body("Found it."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "any chiller issues?").await.unwrap();

    let requests = captured.lock().unwrap();
    assert!(requests.len() >= 2, "expected a second round carrying the tool result");
    assert!(requests[1].contains("Rooftop Chiller Unit 9"), "search hit missing from the model's next-round request: {}", requests[1]);
}

fn searchable_text_field(entity_type: &str, label: &str, is_searchable: bool) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: entity_type.into(),
        label: label.into(),
        field_type: "text".into(),
        options: vec![],
        required: false,
        show_in_list: false,
        sort_order: 0,
        min_value: None,
        max_value: None,
        max_length: None,
        regex_pattern: None,
        is_searchable,
        is_filterable: false,
        is_reportable: false,
        default_value: None,
        is_unique: false,
        help_text: None,
        placeholder: None,
        is_hidden_by_default: false,
    }
}
