//! AI Agent Platform v2 (GitHub issue #170, backend half): the MCP
//! **client** role - discovery against a real local JSON-RPC 2.0 stub
//! standing in for an external MCP server, the two-level write gating
//! (a tool's own `is_write` flag AND the server's `agent_write_tools_enabled`
//! flag), re-discovery preserving an admin's prior classification, the
//! Tool-Call Firewall bridge (`chat_service::agent_requires_admin`,
//! `ai_agent_service::validate_action_names`) and real dispatch through
//! `tools/call` including the `isError:true` failure path. Same
//! `setup_workspace`/stub-listener conventions
//! `ai_agent_connector_tools.rs`/`voice_llm_conversational.rs` already use.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_mcp::{McpServerInput, McpServerUpdate};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, chat_service, mcp_client_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "MCP Client Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [77u8; 32]
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

/// A real local JSON-RPC 2.0 MCP server: answers `initialize` with an
/// empty result, `tools/list` with two tools (`get_widget` read,
/// `create_widget` write), and `tools/call` with a real `isError:true`
/// failure whenever the call's own arguments carry `"shouldFail": true`
/// and a real success echoing the arguments otherwise - covers every
/// branch `mcp_client_service::rpc_call`/`dispatch` needs to distinguish
/// without inventing an undiscoverable tool name.
fn spawn_mcp_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
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
            let req: serde_json::Value = serde_json::from_slice(&body_buf).unwrap_or(serde_json::json!({}));
            let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
            let result = match method {
                "initialize" => serde_json::json!({"protocolVersion": "2024-11-05"}),
                "tools/list" => serde_json::json!({"tools": [
                    {"name": "get_widget", "description": "Fetch a widget by id", "inputSchema": {"type": "object", "properties": {"id": {"type": "string"}}}},
                    {"name": "create_widget", "description": "Create a new widget", "inputSchema": {"type": "object", "properties": {"name": {"type": "string"}}}}
                ]}),
                "tools/call" => {
                    let args = req["params"]["arguments"].clone();
                    if args.get("shouldFail").and_then(|v| v.as_bool()).unwrap_or(false) {
                        serde_json::json!({"content": [{"type": "text", "text": "boom: the external tool failed"}], "isError": true})
                    } else {
                        serde_json::json!({"content": [{"type": "text", "text": format!("ok, args={args}")}], "isError": false})
                    }
                }
                _ => serde_json::json!({}),
            };
            let body = serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": result}).to_string();
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn create_test_server(conn: &rusqlite::Connection, ws: &str, admin: &str, port: u16) -> lanesra_core::models::ai_mcp::McpServer {
    mcp_client_service::create_server(
        conn, ws, &master_key(),
        &McpServerInput { name: "Widgets MCP".into(), base_url: format!("http://127.0.0.1:{port}"), auth_mode: "none".into(), secret_value: None },
        Some(admin),
    )
    .unwrap()
}

#[test]
fn a_non_administrator_cannot_manage_mcp_servers() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);
    let port = spawn_mcp_server();
    assert!(
        mcp_client_service::create_server(&conn, &ws, &master_key(), &McpServerInput { name: "X".into(), base_url: format!("http://127.0.0.1:{port}"), auth_mode: "none".into(), secret_value: None }, Some(&rep)).is_err()
    );
    let server = create_test_server(&conn, &ws, &admin, port);
    assert!(mcp_client_service::list_servers(&conn, &ws, Some(&rep)).is_err());
    assert!(mcp_client_service::list_tools(&conn, &ws, &server.id, Some(&rep)).is_err());
}

#[tokio::test]
async fn discover_tools_populates_read_only_disabled_tools_by_default_and_preserves_flags_on_rediscovery() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_mcp_server();
    let server = create_test_server(&conn, &ws, &admin, port);

    let tools = mcp_client_service::discover_tools(&conn, &ws, &master_key(), &server.id, Some(&admin)).await.unwrap();
    assert_eq!(tools.len(), 2);
    assert!(tools.iter().all(|t| !t.enabled && !t.is_write), "a freshly discovered tool must default to disabled/read-only");

    let reloaded = mcp_client_service::get_server(&conn, &ws, &server.id, Some(&admin)).unwrap();
    assert_eq!(reloaded.last_discovery_status.as_deref(), Some("connected"));

    // An admin opts create_widget in as an enabled write tool.
    mcp_client_service::set_tool_flags(&conn, &ws, &server.id, "create_widget", true, true, Some(&admin)).unwrap();

    // Re-discovering against the same live server must not reset that choice.
    let tools_again = mcp_client_service::discover_tools(&conn, &ws, &master_key(), &server.id, Some(&admin)).await.unwrap();
    let create_widget = tools_again.iter().find(|t| t.tool_name == "create_widget").unwrap();
    assert!(create_widget.enabled && create_widget.is_write, "re-discovery must preserve an admin's prior classification");
    let get_widget = tools_again.iter().find(|t| t.tool_name == "get_widget").unwrap();
    assert!(!get_widget.enabled && !get_widget.is_write, "a tool never opted into must stay at its conservative default");
}

#[tokio::test]
async fn discover_tools_against_an_unreachable_server_records_a_failed_discovery() {
    let (conn, ws, admin) = setup_workspace();
    let server = mcp_client_service::create_server(
        &conn, &ws, &master_key(),
        &McpServerInput { name: "Unreachable".into(), base_url: "http://127.0.0.1:1".into(), auth_mode: "none".into(), secret_value: None },
        Some(&admin),
    )
    .unwrap();

    let err = mcp_client_service::discover_tools(&conn, &ws, &master_key(), &server.id, Some(&admin)).await.unwrap_err();
    assert!(!err.to_string().is_empty());

    let reloaded = mcp_client_service::get_server(&conn, &ws, &server.id, Some(&admin)).unwrap();
    assert_eq!(reloaded.last_discovery_status.as_deref(), Some("failed"));
    assert!(reloaded.last_discovery_message.is_some());
}

#[tokio::test]
async fn agent_tools_are_gated_by_the_two_level_write_flag_and_the_server_opt_in() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_mcp_server();
    let server = create_test_server(&conn, &ws, &admin, port);
    mcp_client_service::discover_tools(&conn, &ws, &master_key(), &server.id, Some(&admin)).await.unwrap();
    mcp_client_service::set_tool_flags(&conn, &ws, &server.id, "get_widget", false, true, Some(&admin)).unwrap();
    mcp_client_service::set_tool_flags(&conn, &ws, &server.id, "create_widget", true, true, Some(&admin)).unwrap();

    // Neither server-level gate is on yet - no agent tools at all, even
    // though both tools are individually enabled.
    assert!(mcp_client_service::agent_tools(&conn, &ws).unwrap().is_empty());

    // Turn on the server's agent-tools gate but not its write gate: only
    // the read tool becomes available.
    mcp_client_service::update_server(
        &conn, &ws, &master_key(), &server.id,
        &McpServerUpdate { name: server.name.clone(), base_url: server.base_url.clone().unwrap(), auth_mode: server.auth_mode.clone(), secret_value: None, agent_tools_enabled: true, agent_write_tools_enabled: false },
        Some(&admin),
    )
    .unwrap();
    let names: Vec<String> = mcp_client_service::agent_tools(&conn, &ws).unwrap().into_iter().map(|t| t.name).collect();
    assert_eq!(names, vec![format!("mcp_tool:{}:get_widget", server.id)]);

    // Also turn on the write gate: the write tool joins under its own
    // write-prefixed name.
    mcp_client_service::update_server(
        &conn, &ws, &master_key(), &server.id,
        &McpServerUpdate { name: server.name.clone(), base_url: server.base_url.clone().unwrap(), auth_mode: server.auth_mode.clone(), secret_value: None, agent_tools_enabled: true, agent_write_tools_enabled: true },
        Some(&admin),
    )
    .unwrap();
    let names: Vec<String> = mcp_client_service::agent_tools(&conn, &ws).unwrap().into_iter().map(|t| t.name).collect();
    assert!(names.contains(&format!("mcp_tool:{}:get_widget", server.id)));
    assert!(names.contains(&format!("mcp_write_tool:{}:create_widget", server.id)));
    assert_eq!(names.len(), 2);

    let options = mcp_client_service::list_options(&conn, &ws).unwrap();
    assert_eq!(options.len(), 2);
    assert!(options.iter().find(|o| o.tool_name.contains("create_widget")).unwrap().requires_admin);
    assert!(!options.iter().find(|o| o.tool_name.contains("get_widget")).unwrap().requires_admin);
}

#[tokio::test]
async fn dispatch_invokes_the_real_tool_call_surfaces_tool_level_errors_and_rejects_ineligible_or_unknown_names() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_mcp_server();
    let server = create_test_server(&conn, &ws, &admin, port);
    mcp_client_service::discover_tools(&conn, &ws, &master_key(), &server.id, Some(&admin)).await.unwrap();
    mcp_client_service::set_tool_flags(&conn, &ws, &server.id, "get_widget", false, true, Some(&admin)).unwrap();
    mcp_client_service::update_server(
        &conn, &ws, &master_key(), &server.id,
        &McpServerUpdate { name: server.name.clone(), base_url: server.base_url.clone().unwrap(), auth_mode: server.auth_mode.clone(), secret_value: None, agent_tools_enabled: true, agent_write_tools_enabled: false },
        Some(&admin),
    )
    .unwrap();

    let tool_name = format!("mcp_tool:{}:get_widget", server.id);
    let result = mcp_client_service::dispatch(&conn, &ws, &master_key(), Some(&admin), &tool_name, &serde_json::json!({"id": "7"})).await.unwrap();
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("\"id\":\"7\""), "the real request arguments should round-trip through the stub server: {text}");

    // create_widget exists on the server but was never opted into at all -
    // not currently available even though the tool name is well-formed.
    let unavailable = format!("mcp_write_tool:{}:create_widget", server.id);
    let err = mcp_client_service::dispatch(&conn, &ws, &master_key(), Some(&admin), &unavailable, &serde_json::json!({})).await.unwrap_err();
    assert!(err.to_string().contains("not currently available"));

    let err = mcp_client_service::dispatch(&conn, &ws, &master_key(), Some(&admin), "list_records", &serde_json::json!({})).await.unwrap_err();
    assert!(err.to_string().contains("is not an MCP tool name"));

    // A real tools/call reporting isError:true must surface as a real Err
    // carrying the tool's own error text, not a silently-successful result.
    let err = mcp_client_service::dispatch(&conn, &ws, &master_key(), Some(&admin), &tool_name, &serde_json::json!({"shouldFail": true})).await.unwrap_err();
    assert!(err.to_string().contains("the external tool failed"));
}

#[test]
fn validate_action_names_rejects_an_mcp_tool_that_is_not_currently_available() {
    let (conn, ws, admin) = setup_workspace();
    let unavailable = AiAgentInput {
        name: "Bad Agent".into(), description: None, icon: "🤖".into(), system_prompt: "You help.".into(),
        action_names: vec!["mcp_tool:not_a_real_server:get_widget".into()],
        delegate_agent_ids: vec![], skill_ids: vec![],
    };
    let err = ai_agent_service::create(&conn, &ws, &unavailable, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("isn't currently available"), "{err}");
}

#[tokio::test]
async fn an_agent_with_a_write_capable_mcp_tool_requires_administrator() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);
    let port = spawn_mcp_server();
    let server = create_test_server(&conn, &ws, &admin, port);
    mcp_client_service::discover_tools(&conn, &ws, &master_key(), &server.id, Some(&admin)).await.unwrap();
    mcp_client_service::set_tool_flags(&conn, &ws, &server.id, "create_widget", true, true, Some(&admin)).unwrap();
    mcp_client_service::update_server(
        &conn, &ws, &master_key(), &server.id,
        &McpServerUpdate { name: server.name.clone(), base_url: server.base_url.clone().unwrap(), auth_mode: server.auth_mode.clone(), secret_value: None, agent_tools_enabled: true, agent_write_tools_enabled: true },
        Some(&admin),
    )
    .unwrap();

    let agent = ai_agent_service::create(
        &conn, &ws,
        &AiAgentInput {
            name: "Writer".into(), description: None, icon: "🤖".into(), system_prompt: "You create widgets.".into(),
            action_names: vec![format!("mcp_write_tool:{}:create_widget", server.id)],
            delegate_agent_ids: vec![], skill_ids: vec![],
        },
        Some(&admin),
    )
    .unwrap();

    // No AI provider is configured at all, so reaching ai_service would
    // fail for that unrelated reason instead - getting the admin-only
    // rejection proves the Tool-Call Firewall's admin gate runs before any
    // network call, same as the connector-tool and fixed-admin-catalog
    // cases elsewhere in this suite.
    let result = chat_service::send_agent_message(&conn, &ws, &master_key(), &rep, &agent.id, "please create a widget").await;
    assert!(result.unwrap_err().to_string().contains("Administrator"));
}

#[tokio::test]
async fn deleting_a_server_removes_its_tools_and_revokes_agent_eligibility_immediately() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_mcp_server();
    let server = create_test_server(&conn, &ws, &admin, port);
    mcp_client_service::discover_tools(&conn, &ws, &master_key(), &server.id, Some(&admin)).await.unwrap();
    mcp_client_service::set_tool_flags(&conn, &ws, &server.id, "get_widget", false, true, Some(&admin)).unwrap();
    mcp_client_service::update_server(
        &conn, &ws, &master_key(), &server.id,
        &McpServerUpdate { name: server.name.clone(), base_url: server.base_url.clone().unwrap(), auth_mode: server.auth_mode.clone(), secret_value: None, agent_tools_enabled: true, agent_write_tools_enabled: false },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(mcp_client_service::agent_tools(&conn, &ws).unwrap().len(), 1);

    mcp_client_service::delete_server(&conn, &ws, &server.id, Some(&admin)).unwrap();
    assert!(mcp_client_service::agent_tools(&conn, &ws).unwrap().is_empty());
    assert!(mcp_client_service::get_server(&conn, &ws, &server.id, Some(&admin)).is_err());
}
