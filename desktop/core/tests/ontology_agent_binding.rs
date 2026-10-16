//! Next-Gen program, Ontology Layer epic, issue #344: AI Agent Ontology
//! Binding. Covers the issue's own acceptance criteria: an unscoped agent
//! behaves exactly as today; a scoped agent can only touch its granted
//! object types/capabilities and is denied everywhere else; an agent
//! granted a specific Action Type can invoke it and is denied for any
//! other, even one on the same object type; an Assign-gated Action Type
//! still enforces that requirement even when the agent's own Ontology
//! scope only grants `update`.
//!
//! Same raw-socket Anthropic stub + `send_agent_message` pattern
//! `ai_agent_policy_engine.rs`/`ai_orchestration_topologies.rs` already
//! established - no real LLM in the loop, just a canned `tool_use`
//! response driving the real dispatch path.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::action_type::{ActionParameterSpec, ActionTypeDefinitionInput};
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::{AiAgentInput, ObjectTypeScope};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    access_role_service, action_type_service, ai_agent_service, ai_service, chat_service,
    company_service, user_service, workspace_service,
};

fn setup_workspace(business_name: &str) -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: business_name.into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [91u8; 32]
}

fn configure_anthropic_key(
    conn: &rusqlite::Connection,
    workspace_id: &str,
    admin: &str,
    port: u16,
) {
    ai_service::save_settings(
        conn,
        workspace_id,
        &master_key(),
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

fn anthropic_tool_use_body(id: &str, name: &str, input: serde_json::Value) -> String {
    serde_json::json!({"content": [{"type": "tool_use", "id": id, "name": name, "input": input}]})
        .to_string()
}

fn anthropic_text_body(text: &str) -> String {
    serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string()
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
                if q.len() > 1 {
                    q.pop_front().unwrap()
                } else {
                    q.front().cloned().unwrap_or_default()
                }
            };
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

fn agent_input(
    action_names: Vec<String>,
    object_type_scopes: Vec<ObjectTypeScope>,
    action_type_ids: Vec<String>,
) -> AiAgentInput {
    AiAgentInput {
        name: "Test Agent".into(),
        description: None,
        icon: "🤖".into(),
        system_prompt: "You are a test agent.".into(),
        action_names,
        delegate_agent_ids: vec![],
        skill_ids: vec![],
        object_type_scopes,
        action_type_ids,
    }
}

fn tool_result_content(messages: &[lanesra_core::models::chat::ChatMessage]) -> String {
    messages
        .iter()
        .find(|m| m.role == "tool")
        .and_then(|m| m.content.clone())
        .unwrap_or_default()
}

#[tokio::test]
async fn an_unscoped_agent_behaves_exactly_as_today() {
    let (conn, ws, admin) = setup_workspace("Unscoped Agent Co");
    let company = company_service::create(
        &conn,
        &ws,
        &CompanyInput {
            name: "Acme".into(),
            status: "Prospect".into(),
            ..Default::default()
        },
        Some(&admin),
    )
    .unwrap();

    let agent = ai_agent_service::create(
        &conn,
        &ws,
        &agent_input(vec!["update_record".into()], vec![], vec![]),
        Some(&admin),
    )
    .unwrap();
    let port = spawn_sequence_stub(vec![
        anthropic_tool_use_body(
            "t1",
            "update_record",
            serde_json::json!({"object_key": "Company", "id": company.id, "data": {"name": "Acme Corp", "status": "Prospect"}}),
        ),
        anthropic_text_body("Updated."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let messages = chat_service::send_agent_message(
        &conn,
        &ws,
        &master_key(),
        &admin,
        &agent.id,
        "rename the company",
    )
    .await
    .unwrap();
    let content = tool_result_content(&messages);
    assert!(
        !content.starts_with("Error:"),
        "an unscoped agent must dispatch exactly as it did before this feature: {content}"
    );
}

#[tokio::test]
async fn a_scoped_agent_can_only_touch_granted_object_types_and_capabilities() {
    let (conn, ws, admin) = setup_workspace("Scoped Agent Co");
    let company = company_service::create(
        &conn,
        &ws,
        &CompanyInput {
            name: "Acme".into(),
            status: "Prospect".into(),
            ..Default::default()
        },
        Some(&admin),
    )
    .unwrap();

    // Scoped to Company: read + update only (no create, no Contact at all).
    let agent = ai_agent_service::create(
        &conn,
        &ws,
        &agent_input(
            vec![
                "list_records".into(),
                "create_record".into(),
                "update_record".into(),
            ],
            vec![ObjectTypeScope {
                object_type: "Company".into(),
                capabilities: vec!["read".into(), "update".into()],
            }],
            vec![],
        ),
        Some(&admin),
    )
    .unwrap();

    // Allowed: list_records on Company (read).
    let port = spawn_sequence_stub(vec![
        anthropic_tool_use_body(
            "t1",
            "list_records",
            serde_json::json!({"object_key": "Company"}),
        ),
        anthropic_text_body("done"),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);
    let messages = chat_service::send_agent_message(
        &conn,
        &ws,
        &master_key(),
        &admin,
        &agent.id,
        "list companies",
    )
    .await
    .unwrap();
    assert!(
        !tool_result_content(&messages).starts_with("Error:"),
        "read on a granted object type must be allowed: {}",
        tool_result_content(&messages)
    );

    // Denied: create_record on Company (not a granted capability).
    let port = spawn_sequence_stub(vec![
        anthropic_tool_use_body(
            "t2",
            "create_record",
            serde_json::json!({"object_key": "Company", "data": {"name": "New Co", "status": "Prospect"}}),
        ),
        anthropic_text_body("done"),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);
    let messages = chat_service::send_agent_message(
        &conn,
        &ws,
        &master_key(),
        &admin,
        &agent.id,
        "create a company",
    )
    .await
    .unwrap();
    let content = tool_result_content(&messages);
    assert!(
        content.starts_with("Error:") && content.contains("does not grant"),
        "create must be denied when only read/update are granted: {content}"
    );

    // Denied: update_record on Contact (not a scoped object type at all).
    let port = spawn_sequence_stub(vec![
        anthropic_tool_use_body(
            "t3",
            "update_record",
            serde_json::json!({"object_key": "Contact", "id": "whatever", "data": {}}),
        ),
        anthropic_text_body("done"),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);
    let messages = chat_service::send_agent_message(
        &conn,
        &ws,
        &master_key(),
        &admin,
        &agent.id,
        "update a contact",
    )
    .await
    .unwrap();
    let content = tool_result_content(&messages);
    assert!(
        content.starts_with("Error:") && content.contains("not scoped"),
        "an object type with no scope entry at all must be denied: {content}"
    );

    let _ = company.id;
}

#[tokio::test]
async fn an_agent_granted_a_specific_action_type_can_invoke_it_denied_for_others() {
    let (conn, ws, admin) = setup_workspace("Agent Action Type Co");
    let admin_ref = Some(admin.as_str());
    let company = company_service::create(
        &conn,
        &ws,
        &CompanyInput {
            name: "Acme".into(),
            status: "Prospect".into(),
            ..Default::default()
        },
        admin_ref,
    )
    .unwrap();

    let granted = action_type_service::create(
        &conn,
        &ws,
        &ActionTypeDefinitionInput {
            name: "Mark Inactive".into(),
            description: "".into(),
            target_object_type: "Company".into(),
            required_capability: "update".into(),
            parameters: vec![],
            effect: "set_status".into(),
            effect_config_json: r#"{"target_status":"Inactive"}"#.into(),
        },
        admin_ref,
    )
    .unwrap();
    let not_granted = action_type_service::create(
        &conn,
        &ws,
        &ActionTypeDefinitionInput {
            name: "Mark VIP".into(),
            description: "".into(),
            target_object_type: "Company".into(),
            required_capability: "update".into(),
            parameters: vec![],
            effect: "set_status".into(),
            effect_config_json: r#"{"target_status":"VIP"}"#.into(),
        },
        admin_ref,
    )
    .unwrap();

    let agent = ai_agent_service::create(
        &conn,
        &ws,
        &agent_input(vec![], vec![], vec![granted.id.clone()]),
        admin_ref,
    )
    .unwrap();

    // Granted Action Type succeeds.
    let port = spawn_sequence_stub(vec![
        anthropic_tool_use_body(
            "t1",
            "invoke_action",
            serde_json::json!({"action_type_name": "Mark Inactive", "record_id": company.id, "params": {}}),
        ),
        anthropic_text_body("done"),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);
    let messages = chat_service::send_agent_message(
        &conn,
        &ws,
        &master_key(),
        &admin,
        &agent.id,
        "mark it inactive",
    )
    .await
    .unwrap();
    let content = tool_result_content(&messages);
    assert!(
        !content.starts_with("Error:"),
        "a granted Action Type must invoke successfully: {content}"
    );
    assert_eq!(
        company_service::get(&conn, &company.id).unwrap().status,
        "Inactive"
    );

    // Un-granted Action Type on the same object type is denied.
    let port = spawn_sequence_stub(vec![
        anthropic_tool_use_body(
            "t2",
            "invoke_action",
            serde_json::json!({"action_type_name": "Mark VIP", "record_id": company.id, "params": {}}),
        ),
        anthropic_text_body("done"),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);
    let messages = chat_service::send_agent_message(
        &conn,
        &ws,
        &master_key(),
        &admin,
        &agent.id,
        "mark it VIP",
    )
    .await
    .unwrap();
    let content = tool_result_content(&messages);
    assert!(
        content.starts_with("Error:")
            && content.contains("isn't one of this agent's granted Action Types"),
        "an un-granted Action Type must be denied even on the same object type: {content}"
    );
    assert_eq!(
        company_service::get(&conn, &company.id).unwrap().status,
        "Inactive",
        "the denied call must not have changed anything"
    );

    let _ = not_granted.id;
}

#[tokio::test]
async fn an_assign_gated_action_type_still_enforces_capability_even_with_update_only_scope() {
    let (conn, ws, admin) = setup_workspace("Assign Gate Co");
    let admin_ref = Some(admin.as_str());
    let company = company_service::create(
        &conn,
        &ws,
        &CompanyInput {
            name: "Acme".into(),
            status: "Prospect".into(),
            ..Default::default()
        },
        admin_ref,
    )
    .unwrap();

    // A role granting Update but explicitly not Assign on every object.
    let role = access_role_service::create(
        &conn,
        &ws,
        &AccessRoleInput {
            name: "Updater".into(),
            description: "".into(),
        },
        admin_ref,
    )
    .unwrap();
    access_role_service::upsert_grant(
        &conn,
        &role.id,
        &AccessRoleGrantInput {
            object_key: "*".into(),
            can_create: true,
            can_read: true,
            can_update: true,
            can_delete: false,
            can_assign: false,
            record_scope: "ORGANIZATION".into(),
        },
        admin_ref,
    )
    .unwrap();
    let updater = user_service::create(
        &conn,
        &ws,
        &NewUser {
            username: "updater".into(),
            display_name: "Updater".into(),
            password: "anothersecretpw".into(),
            roles: vec!["Sales".to_string()],
        },
        admin_ref,
    )
    .unwrap();
    access_role_service::assign_to_user(&conn, &updater.id, &role.id, admin_ref).unwrap();

    let reassign = action_type_service::create(
        &conn,
        &ws,
        &ActionTypeDefinitionInput {
            name: "Reassign Owner".into(),
            description: "".into(),
            target_object_type: "Company".into(),
            required_capability: "assign".into(),
            parameters: vec![ActionParameterSpec {
                key: "newOwner".into(),
                label: "New owner".into(),
                field_type: "text".into(),
                required: true,
                default_value: None,
            }],
            effect: "reassign_owner".into(),
            effect_config_json: r#"{"owner_param":"newOwner"}"#.into(),
        },
        admin_ref,
    )
    .unwrap();

    // This agent's own Ontology scope grants only `update` on Company -
    // not `assign` - and it IS granted this specific Action Type.
    let agent = ai_agent_service::create(
        &conn,
        &ws,
        &agent_input(
            vec![],
            vec![ObjectTypeScope {
                object_type: "Company".into(),
                capabilities: vec!["update".into()],
            }],
            vec![reassign.id.clone()],
        ),
        admin_ref,
    )
    .unwrap();

    let port = spawn_sequence_stub(vec![
        anthropic_tool_use_body(
            "t1",
            "invoke_action",
            serde_json::json!({"action_type_name": "Reassign Owner", "record_id": company.id, "params": {"newOwner": updater.id}}),
        ),
        anthropic_text_body("done"),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);
    // Chatting as the Updater (who has Update, not Assign) - the agent's
    // own action_type_ids grant lets the tool call through to
    // action_type_service::invoke, whose own required_capability check
    // must still deny it.
    let messages = chat_service::send_agent_message(
        &conn,
        &ws,
        &master_key(),
        &updater.id,
        &agent.id,
        "reassign it",
    )
    .await
    .unwrap();
    let content = tool_result_content(&messages);
    assert!(content.starts_with("Error:"), "an Assign-gated Action Type must still deny an actor without Assign, even though this agent's own Ontology scope and action_type_ids both permit the call: {content}");
}
