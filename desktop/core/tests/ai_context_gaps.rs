//! AI & Agentic Layer: closing two real context gaps in what an agent
//! (or MCP/REST caller, since both sit on the same `api_object_service`)
//! can see about how this workspace is actually built -
//!
//! - **Relationship metadata**: `get_object_metadata`'s new `relationships`
//!   list (schema-level: which relationships an object participates in,
//!   from either direction) and `get_related_records` (data-level: the
//!   actual linked records for one record, via the same
//!   `relationship_service::related_records_for` the desktop UI's own
//!   "Related records" panel already calls).
//! - **`get_platform_overview`**: a static explainer of how Lanesra OS's
//!   own primitives compose, plus a live per-workspace summary - tested
//!   here via the same "tool result reaches the model" round trip
//!   `ai_context_layer.rs`'s own `search_records_tool_result_reaches_the_
//!   model` test established, since `chat_service`'s tool dispatch is
//!   private and only reachable that way from outside the crate.
//!
//! Reuses `relationships.rs`'s own Vendor/Company builders and
//! `ai_context_layer.rs`'s own stub-listener pattern.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::relationship::RelationshipDefinitionInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_service, api_object_service, chat_service, company_service, custom_object_service, custom_record_service, relationship_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Context Gaps Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [61u8; 32]
}

fn vendor_object() -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 }
}

fn vendor_company_relationship() -> RelationshipDefinitionInput {
    RelationshipDefinitionInput {
        source_entity_type: "".into(), // filled in by the caller once the Vendor object's real key is known
        target_entity_type: "Company".into(),
        relationship_type: "many_to_one".into(),
        forward_label: "Client".into(),
        reverse_label: "Vendors".into(),
        is_required: false,
        show_related_list: true,
        delete_behavior: "restrict".into(),
        sort_order: 0,
    }
}

fn company_input(name: &str) -> CompanyInput {
    CompanyInput { name: name.into(), status: "Active Customer".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }
}

#[test]
fn get_object_metadata_lists_relationships_from_both_directions() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();
    let def = relationship_service::create(&conn, &ws, &RelationshipDefinitionInput { source_entity_type: vendor.key.clone(), ..vendor_company_relationship() }, Some(&admin)).unwrap();

    let vendor_meta = api_object_service::get_metadata(&conn, &ws, &vendor.key).unwrap();
    assert_eq!(vendor_meta.relationships.len(), 1);
    let forward = &vendor_meta.relationships[0];
    assert_eq!(forward.relationship_key, def.key);
    assert_eq!(forward.related_object_key, "Company");
    assert_eq!(forward.direction, "forward");
    assert_eq!(forward.label, "Client", "the source side sees its own forward_label");

    let company_meta = api_object_service::get_metadata(&conn, &ws, "Company").unwrap();
    let reverse = company_meta.relationships.iter().find(|r| r.relationship_key == def.key).expect("Company should see the relationship too");
    assert_eq!(reverse.related_object_key, vendor.key);
    assert_eq!(reverse.direction, "reverse");
    assert_eq!(reverse.label, "Vendors", "the target side sees the reverse_label, not the forward one");
}

#[test]
fn get_related_records_follows_a_link_from_either_side_of_it() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();
    let def = relationship_service::create(&conn, &ws, &RelationshipDefinitionInput { source_entity_type: vendor.key.clone(), ..vendor_company_relationship() }, Some(&admin)).unwrap();
    let acme = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();
    let vendor_record = custom_record_service::create(&conn, &ws, &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Acme's Supplier".into(), status: "Active".into(), owner_user_id: None, notes: None }, Some(&admin)).unwrap();
    relationship_service::link(&conn, &ws, &def.id, &vendor.key, &vendor_record.id, "Company", &acme.id, Some(&admin)).unwrap();

    let from_vendor = api_object_service::related_records(&conn, &ws, &vendor.key, &vendor_record.id).unwrap();
    assert_eq!(from_vendor.len(), 1);
    assert_eq!(from_vendor[0].entity_type, "Company");
    assert_eq!(from_vendor[0].entity_id, acme.id);
    assert_eq!(from_vendor[0].display_name, "Acme");

    let from_company = api_object_service::related_records(&conn, &ws, "Company", &acme.id).unwrap();
    assert_eq!(from_company.len(), 1);
    assert_eq!(from_company[0].entity_type, vendor.key);
    assert_eq!(from_company[0].entity_id, vendor_record.id);

    // An unrelated record sees nothing.
    let other = company_service::create(&conn, &ws, &company_input("Unrelated Co"), Some(&admin)).unwrap();
    assert!(api_object_service::related_records(&conn, &ws, "Company", &other.id).unwrap().is_empty());
}

// --- get_platform_overview: only reachable through the tool-calling loop ---

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

fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str, action_names: Vec<String>) -> lanesra_core::models::ai_agent::AiAgentDefinition {
    ai_agent_service::create(
        conn, ws,
        &AiAgentInput { name: name.into(), description: None, icon: "🤖".into(), system_prompt: format!("You are {name}."), action_names, delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(admin),
    )
    .unwrap()
}

/// Same raw-socket stub `ai_context_layer.rs`'s own `spawn_sequence_stub`
/// already uses.
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

#[tokio::test]
async fn get_platform_overview_tool_result_reaches_the_model_with_a_live_workspace_summary() {
    let (conn, ws, admin) = setup_workspace();
    // Give the workspace something real to summarize.
    custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();

    let agent = make_agent(&conn, &ws, &admin, "Orientation Bot", vec!["get_platform_overview".into()]);
    let (port, captured) = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "get_platform_overview", serde_json::json!({})),
        anthropic_text_body("Here's how it's built."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "how do I build a new capability here?").await.unwrap();

    let requests = captured.lock().unwrap();
    assert!(requests.len() >= 2, "expected a second round carrying the tool result");
    let second = &requests[1];
    assert!(second.contains("Custom Object"), "the static platform explainer should reach the model: {second}");
    assert!(second.contains("\\\"custom_object_count\\\":1") || second.contains("custom_object_count"), "the live workspace summary should reach the model: {second}");
}
