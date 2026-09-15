//! Integration Hub Tool Bridge: proves the fail-closed schema handling at
//! import time (`connector_service::is_locally_typed`), the per-connector
//! agent-tool gating and dispatch (`connector_tool_service`), and the
//! Agent Foundry wiring (`ai_agent_service::validate_action_names`,
//! `chat_service::agent_requires_admin` via `send_agent_message`) - same
//! `setup_workspace`/echo-server test-double conventions
//! `integration_connector.rs`/`ai_agent_foundry.rs` already use.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::integration::{ConnectionInput, ConnectionRefInput, ConnectorImportInput};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, chat_service, connection_ref_service, connection_service, connector_service, connector_tool_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Tool Bridge Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [42u8; 32]
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

/// Same minimal raw-socket echo server `integration_connector.rs` uses -
/// always answers 200 with the request line echoed back in the JSON body.
fn spawn_echo_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line);
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) if line == "\r\n" || line.is_empty() => break,
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }
            let echoed = request_line.trim().replace('"', "'");
            let body = format!("{{\"ok\":true,\"request_line\":\"{echoed}\"}}");
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let mut stream = stream;
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn setup_reference(conn: &rusqlite::Connection, workspace_id: &str, admin_id: &str, base_url: &str) -> String {
    let connection = connection_service::create(
        conn, workspace_id, &master_key(),
        &ConnectionInput { name: "Local test API".into(), connection_type: "rest".into(), base_url: Some(base_url.to_string()), auth_mode: "none".into(), secret_value: None, config_json: "{}".into(), owner_user_id: None },
        Some(admin_id),
    )
    .unwrap();
    let reference = connection_ref_service::create(
        conn, workspace_id,
        &ConnectionRefInput { reference_name: "Widgets API".into(), reference_key: "widgets_api".into(), expected_connection_type: "rest".into(), connection_id: Some(connection.id.clone()) },
        Some(admin_id),
    )
    .unwrap();
    reference.reference_key
}

/// One read (GET, no body) operation, one write operation with a
/// confidently-typed nested body, one write operation with a bare
/// `{"type":"object"}` body (locally typed but not informative), and one
/// write operation whose body is an unresolved `$ref` (not locally typed
/// at all) - covers every fail-closed branch this bridge needs.
const OPENAPI_JSON: &str = r##"{
  "openapi": "3.0.0",
  "info": {"title": "Widgets API", "version": "1.0.0"},
  "paths": {
    "/items/{id}": {
      "get": {
        "operationId": "getItem",
        "summary": "Fetch one item",
        "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}]
      }
    },
    "/items": {
      "post": {
        "operationId": "createItem",
        "summary": "Create an item",
        "requestBody": {
          "required": true,
          "content": {"application/json": {"schema": {
            "type": "object",
            "properties": {"name": {"type": "string"}, "qty": {"type": "integer"}},
            "required": ["name"]
          }}}
        }
      },
      "put": {
        "operationId": "vagueUpdate",
        "summary": "Update with an ambiguous body",
        "requestBody": {
          "required": true,
          "content": {"application/json": {"schema": {"type": "object"}}}
        }
      }
    },
    "/items/ref": {
      "post": {
        "operationId": "refBased",
        "summary": "Create via an unresolved $ref body",
        "requestBody": {
          "required": true,
          "content": {"application/json": {"schema": {"$ref": "#/components/schemas/Widget"}}}
        }
      }
    }
  }
}"##;

fn import_widgets(conn: &rusqlite::Connection, workspace_id: &str, admin_id: &str) -> lanesra_core::models::integration::Connector {
    let input = ConnectorImportInput {
        name: "Widgets".into(),
        description: None,
        spec_text: OPENAPI_JSON.into(),
        spec_format: "json".into(),
        selected_operation_ids: vec!["getItem".into(), "createItem".into(), "vagueUpdate".into(), "refBased".into()],
    };
    connector_service::import(conn, workspace_id, &input, Some(admin_id)).unwrap()
}

#[test]
fn import_populates_request_schema_json_only_when_confidently_typed() {
    let (conn, ws, admin) = setup_workspace();
    let connector = import_widgets(&conn, &ws, &admin);

    let create_item = connector.actions.iter().find(|a| a.action_key == "createItem").unwrap();
    let schema: serde_json::Value = serde_json::from_str(create_item.request_schema_json.as_deref().expect("a fully-typed nested body should be stored")).unwrap();
    assert!(schema["properties"]["name"]["type"] == "string");
    assert!(schema["properties"]["qty"]["type"] == "integer");

    // A bare `{"type":"object"}` body has nothing ambiguous to reject at
    // import time (no $ref, no oneOf/anyOf/allOf) - it's stored verbatim,
    // even though it's not informative. Whether it's *informative enough*
    // to become a tool is a separate, later check (see the next test).
    let vague_update = connector.actions.iter().find(|a| a.action_key == "vagueUpdate").unwrap();
    assert!(vague_update.request_schema_json.is_some());

    // An unresolved $ref is genuinely ambiguous - never stored.
    let ref_based = connector.actions.iter().find(|a| a.action_key == "refBased").unwrap();
    assert!(ref_based.request_schema_json.is_none());
}

#[test]
fn agent_tools_are_gated_by_connector_opt_in_write_flag_and_schema_informativeness() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_echo_server();
    let reference_key = setup_reference(&conn, &ws, &admin, &format!("http://127.0.0.1:{port}"));
    let connector = import_widgets(&conn, &ws, &admin);

    // Not opted in at all - no tools.
    assert!(connector_tool_service::agent_tools(&conn, &ws).unwrap().is_empty());

    // Read-only opt-in: only the GET action becomes a tool.
    connector_service::update_agent_tool_settings(&conn, &ws, &connector.id, true, false, Some(&reference_key), Some(&admin)).unwrap();
    let read_only = connector_tool_service::agent_tools(&conn, &ws).unwrap();
    assert_eq!(read_only.len(), 1);
    assert_eq!(read_only[0].name, format!("connector_action:{}:getItem", connector.id));

    // Also opt into write: the confidently-typed POST joins the read
    // tool, but the ambiguous PUT and the $ref-based POST never do -
    // fail closed on both a missing and a present-but-uninformative
    // request_schema_json.
    connector_service::update_agent_tool_settings(&conn, &ws, &connector.id, true, true, Some(&reference_key), Some(&admin)).unwrap();
    let names: Vec<String> = connector_tool_service::agent_tools(&conn, &ws).unwrap().into_iter().map(|t| t.name).collect();
    assert!(names.contains(&format!("connector_action:{}:getItem", connector.id)));
    assert!(names.contains(&format!("connector_write_action:{}:createItem", connector.id)));
    assert!(!names.iter().any(|n| n.contains("vagueUpdate")), "an uninformative body schema must not become a tool");
    assert!(!names.iter().any(|n| n.contains("refBased")), "a missing request_schema_json must not become a tool");
    assert_eq!(names.len(), 2);

    // Same set, described for the admin UI.
    let options = connector_tool_service::list_options(&conn, &ws).unwrap();
    assert_eq!(options.len(), 2);
    assert!(options.iter().find(|o| o.action_key == "createItem").unwrap().requires_admin);
    assert!(!options.iter().find(|o| o.action_key == "getItem").unwrap().requires_admin);
}

#[test]
fn update_agent_tool_settings_requires_and_validates_a_reference_key() {
    let (conn, ws, admin) = setup_workspace();
    let connector = import_widgets(&conn, &ws, &admin);

    assert!(
        connector_service::update_agent_tool_settings(&conn, &ws, &connector.id, true, false, None, Some(&admin)).is_err(),
        "enabling agent tools without a reference key must be rejected"
    );
    assert!(
        connector_service::update_agent_tool_settings(&conn, &ws, &connector.id, true, false, Some("not_a_real_reference"), Some(&admin)).is_err(),
        "an unknown reference key must be rejected"
    );

    let port = spawn_echo_server();
    let reference_key = setup_reference(&conn, &ws, &admin, &format!("http://127.0.0.1:{port}"));
    assert!(connector_service::update_agent_tool_settings(&conn, &ws, &connector.id, true, false, Some(&reference_key), Some(&admin)).is_ok());
}

#[tokio::test]
async fn dispatch_invokes_the_bound_connection_and_rejects_ineligible_or_unknown_names() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_echo_server();
    let reference_key = setup_reference(&conn, &ws, &admin, &format!("http://127.0.0.1:{port}"));
    let connector = import_widgets(&conn, &ws, &admin);
    connector_service::update_agent_tool_settings(&conn, &ws, &connector.id, true, false, Some(&reference_key), Some(&admin)).unwrap();

    let read_tool_name = format!("connector_action:{}:getItem", connector.id);
    let result = connector_tool_service::dispatch(&conn, &ws, &master_key(), Some(&admin), &read_tool_name, &serde_json::json!({"id": "7"})).await.unwrap();
    assert_eq!(result["ok"], true);
    let request_line = result["response_body"]["request_line"].as_str().unwrap();
    assert!(request_line.starts_with("GET /items/7"), "path param should be substituted: {request_line}");

    // The write action exists but write access was never opted into -
    // still not currently available, even though the name is well-formed.
    let write_tool_name = format!("connector_write_action:{}:createItem", connector.id);
    let err = connector_tool_service::dispatch(&conn, &ws, &master_key(), Some(&admin), &write_tool_name, &serde_json::json!({"body": {"name": "Widget"}})).await.unwrap_err();
    assert!(err.to_string().contains("not currently available"));

    let err = connector_tool_service::dispatch(&conn, &ws, &master_key(), Some(&admin), "list_records", &serde_json::json!({})).await.unwrap_err();
    assert!(err.to_string().contains("not a connector tool name"));
}

#[test]
fn validate_action_names_rejects_a_connector_tool_that_is_not_currently_available() {
    let (conn, ws, admin) = setup_workspace();
    let connector = import_widgets(&conn, &ws, &admin);

    let unavailable = AiAgentInput {
        name: "Bad Agent".into(), description: None, icon: "🤖".into(), system_prompt: "You help.".into(),
        action_names: vec![format!("connector_action:{}:getItem", connector.id)],
        delegate_agent_ids: vec![], skill_ids: vec![],
    };
    let err = ai_agent_service::create(&conn, &ws, &unavailable, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("isn't currently available"), "{err}");

    let port = spawn_echo_server();
    let reference_key = setup_reference(&conn, &ws, &admin, &format!("http://127.0.0.1:{port}"));
    connector_service::update_agent_tool_settings(&conn, &ws, &connector.id, true, true, Some(&reference_key), Some(&admin)).unwrap();

    let available = AiAgentInput {
        name: "Good Agent".into(), description: None, icon: "🤖".into(), system_prompt: "You help.".into(),
        action_names: vec![format!("connector_action:{}:getItem", connector.id)],
        delegate_agent_ids: vec![], skill_ids: vec![],
    };
    assert!(ai_agent_service::create(&conn, &ws, &available, Some(&admin)).is_ok());
}

#[tokio::test]
async fn an_agent_with_a_write_capable_connector_tool_requires_administrator() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);
    let port = spawn_echo_server();
    let reference_key = setup_reference(&conn, &ws, &admin, &format!("http://127.0.0.1:{port}"));
    let connector = import_widgets(&conn, &ws, &admin);
    connector_service::update_agent_tool_settings(&conn, &ws, &connector.id, true, true, Some(&reference_key), Some(&admin)).unwrap();

    let agent = ai_agent_service::create(
        &conn, &ws,
        &AiAgentInput {
            name: "Writer".into(), description: None, icon: "🤖".into(), system_prompt: "You write things.".into(),
            action_names: vec![format!("connector_write_action:{}:createItem", connector.id)],
            delegate_agent_ids: vec![], skill_ids: vec![],
        },
        Some(&admin),
    )
    .unwrap();

    // No AI provider key configured at all - reaching ai_service would
    // fail for that reason instead, so getting the admin-only rejection
    // proves the gate runs before any network call, same as the fixed
    // admin-catalog case in ai_agent_foundry.rs.
    let result = chat_service::send_agent_message(&conn, &ws, &master_key(), &rep, &agent.id, "create a widget").await;
    assert!(result.unwrap_err().to_string().contains("Administrator"));
}
