//! Integration Hub (spec §6/§6.3/§17): proves `connector_service`'s
//! OpenAPI 3.x parsing (JSON and YAML, discovered operations, warnings
//! for every unsupported construct) and `connector_execution_service`'s
//! actual HTTP invocation - against a real local HTTP listener spun up
//! inside the test itself, not a live third-party API (this environment
//! has no access to one), but the exact same request-building code path
//! a live API would hit.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::domain::ids::new_uuid;
use lanesra_core::models::integration::{ConnectionInput, ConnectionRefInput, ConnectorImportInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::integration_pending_action_repo;
use lanesra_core::services::{connection_ref_service, connection_service, connector_execution_service, connector_service, integration_log_service, secret_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Connector Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [7u8; 32]
}

/// A minimal raw-socket HTTP/1.1 server: reads the request line and
/// headers, ignores the body, and always answers 200 with a JSON body
/// that echoes the request line back - enough to prove a Connector
/// Action actually substituted path/query params into the real request
/// it sent, without pulling in a web framework just for a test double.
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

const OPENAPI_JSON: &str = r#"{
  "openapi": "3.0.0",
  "info": {"title": "Widgets API", "version": "1.2.0"},
  "paths": {
    "/items/{id}": {
      "parameters": [{"name": "trace", "in": "header", "schema": {"type": "string"}}],
      "get": {
        "operationId": "getItem",
        "summary": "Fetch one item",
        "parameters": [
          {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}},
          {"name": "limit", "in": "query", "schema": {"type": "integer"}},
          {"name": "session", "in": "cookie", "schema": {"type": "string"}},
          {"in": "query", "schema": {"type": "string"}}
        ]
      },
      "post": {
        "summary": "No operationId here",
        "requestBody": {
          "required": true,
          "content": {"application/json": {"schema": {"type": "object"}}}
        }
      }
    },
    "/callbacks-demo": {
      "get": {
        "operationId": "withCallbacks",
        "callbacks": {"onEvent": {}}
      }
    },
    "/external": {"$ref": "external.yaml#/paths/~1external"}
  }
}"#;

#[test]
fn preview_import_discovers_operations_and_warns_about_every_unsupported_construct() {
    let preview = connector_service::preview_import(OPENAPI_JSON, "json").unwrap();
    assert_eq!(preview.title, "Widgets API");
    assert_eq!(preview.version, "1.2.0");

    // getItem: found with 3 usable params (id, limit, trace merged from the
    // shared path-item level) - cookie and the nameless param are dropped
    // with warnings, not silently kept.
    let get_item = preview.operations.iter().find(|o| o.operation_id == "getItem").expect("getItem discovered");
    assert_eq!(get_item.http_method, "GET");
    assert_eq!(get_item.path_template, "/items/{id}");
    let param_names: Vec<&str> = get_item.params.iter().map(|p| p.name.as_str()).collect();
    assert!(param_names.contains(&"id"));
    assert!(param_names.contains(&"limit"));
    assert!(param_names.contains(&"trace"), "shared path-item parameter should merge in");
    assert!(!param_names.contains(&"session"), "cookie params are not supported");

    // The POST has no operationId - a synthesized one is used, and the
    // request body becomes a "body" param.
    let synthesized = preview.operations.iter().find(|o| o.http_method == "POST").expect("POST discovered");
    assert!(synthesized.operation_id.starts_with("post_"));
    assert!(synthesized.params.iter().any(|p| p.name == "body" && p.location == "body"));

    // callbacks-demo is skipped entirely (not discovered as an operation).
    assert!(!preview.operations.iter().any(|o| o.operation_id == "withCallbacks"));

    // Warnings cover every construct this parser can't safely represent.
    assert!(preview.warnings.iter().any(|w| w.contains("cookie")));
    assert!(preview.warnings.iter().any(|w| w.contains("missing 'name' or 'in'")));
    assert!(preview.warnings.iter().any(|w| w.contains("operationId")));
    assert!(preview.warnings.iter().any(|w| w.contains("callbacks")));
    assert!(preview.warnings.iter().any(|w| w.contains("$ref")));
}

#[test]
fn preview_import_parses_yaml_identically_to_json() {
    let yaml = r#"
openapi: "3.0.0"
info:
  title: Widgets API
  version: "1.2.0"
paths:
  /items/{id}:
    get:
      operationId: getItem
      parameters:
        - name: id
          in: path
          required: true
          schema: {type: string}
"#;
    let preview = connector_service::preview_import(yaml, "yaml").unwrap();
    assert_eq!(preview.title, "Widgets API");
    assert_eq!(preview.operations.len(), 1);
    assert_eq!(preview.operations[0].operation_id, "getItem");
}

#[test]
fn import_saves_only_the_selected_operations_and_delete_removes_them() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let input = ConnectorImportInput {
        name: "Widgets".into(),
        description: Some("Test connector".into()),
        spec_text: OPENAPI_JSON.into(),
        spec_format: "json".into(),
        selected_operation_ids: vec!["getItem".into()],
    };
    let connector = connector_service::import(&conn, &workspace_id, &input, Some(&admin_id)).unwrap();
    assert_eq!(connector.actions.len(), 1, "only the selected operation should be saved");
    assert_eq!(connector.actions[0].action_key, "getItem");
    assert_eq!(connector.actions[0].http_method, "GET");

    let fetched = connector_service::get(&conn, &workspace_id, &connector.id).unwrap();
    assert_eq!(fetched.actions.len(), 1);
    assert_eq!(connector_service::list_for_workspace(&conn, &workspace_id).unwrap().len(), 1);

    connector_service::delete(&conn, &workspace_id, &connector.id, Some(&admin_id)).unwrap();
    assert!(connector_service::list_for_workspace(&conn, &workspace_id).unwrap().is_empty());
}

#[test]
fn import_rejects_an_empty_selection() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let input = ConnectorImportInput { name: "Widgets".into(), description: None, spec_text: OPENAPI_JSON.into(), spec_format: "json".into(), selected_operation_ids: vec![] };
    assert!(connector_service::import(&conn, &workspace_id, &input, Some(&admin_id)).is_err());
}

fn setup_reference(conn: &rusqlite::Connection, workspace_id: &str, admin_id: &str, base_url: &str) -> String {
    let connection = connection_service::create(
        conn,
        workspace_id,
        &master_key(),
        &ConnectionInput { name: "Local test API".into(), connection_type: "rest".into(), base_url: Some(base_url.to_string()), auth_mode: "none".into(), secret_value: None, config_json: "{}".into(), owner_user_id: None },
        Some(admin_id),
    )
    .unwrap();
    let reference = connection_ref_service::create(
        conn,
        workspace_id,
        &ConnectionRefInput { reference_name: "Widgets API".into(), reference_key: "widgets_api".into(), expected_connection_type: "rest".into(), connection_id: Some(connection.id.clone()) },
        Some(admin_id),
    )
    .unwrap();
    reference.reference_key
}

#[tokio::test]
async fn execute_invokes_the_real_action_against_the_bound_connection() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_echo_server();
    let base_url = format!("http://127.0.0.1:{port}");
    let reference_key = setup_reference(&conn, &workspace_id, &admin_id, &base_url);

    let import_input = ConnectorImportInput { name: "Widgets".into(), description: None, spec_text: OPENAPI_JSON.into(), spec_format: "json".into(), selected_operation_ids: vec!["getItem".into()] };
    let connector = connector_service::import(&conn, &workspace_id, &import_input, Some(&admin_id)).unwrap();

    let params = serde_json::json!({"id": "42", "limit": "10"});
    let result = connector_execution_service::execute(&conn, &workspace_id, &master_key(), &connector.id, "getItem", &reference_key, &params, None).await.unwrap();

    assert!(result.ok, "{result:?}");
    assert_eq!(result.status_code, Some(200));
    let request_line = result.response_body["request_line"].as_str().unwrap();
    assert!(request_line.starts_with("GET /items/42?"), "path param should be substituted: {request_line}");
    assert!(request_line.contains("limit=10"), "query param should be present: {request_line}");

    // The successful call is a real row in the unified execution log.
    let executions = integration_log_service::list_executions(&conn, &workspace_id, &Default::default()).unwrap();
    assert!(executions.iter().any(|e| e.execution_type == "connector_action" && e.status == "success"));
}

#[tokio::test]
async fn execute_logs_a_failure_when_the_connector_does_not_exist() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let result = connector_execution_service::execute(&conn, &workspace_id, &master_key(), "not-a-real-id", "whatever", "not-a-reference", &serde_json::json!({}), None).await.unwrap();
    assert!(!result.ok);
    let executions = integration_log_service::list_executions(&conn, &workspace_id, &Default::default()).unwrap();
    assert!(executions.iter().any(|e| e.execution_type == "connector_action" && e.status == "failed"), "even an early resolution failure must be logged");
}

#[tokio::test]
async fn drain_pending_actions_invokes_and_removes_every_queued_call() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_echo_server();
    let base_url = format!("http://127.0.0.1:{port}");
    let reference_key = setup_reference(&conn, &workspace_id, &admin_id, &base_url);
    let import_input = ConnectorImportInput { name: "Widgets".into(), description: None, spec_text: OPENAPI_JSON.into(), spec_format: "json".into(), selected_operation_ids: vec!["getItem".into()] };
    let connector = connector_service::import(&conn, &workspace_id, &import_input, Some(&admin_id)).unwrap();

    integration_pending_action_repo::enqueue(
        &conn, &new_uuid(), &workspace_id, &connector.id, "getItem", &reference_key,
        &serde_json::json!({"id": "1", "limit": "5"}).to_string(), Some("Company"), None,
    )
    .unwrap();
    integration_pending_action_repo::enqueue(
        &conn, &new_uuid(), &workspace_id, &connector.id, "getItem", &reference_key,
        &serde_json::json!({"id": "2", "limit": "5"}).to_string(), Some("Company"), None,
    )
    .unwrap();

    let drained = connector_execution_service::drain_pending_actions(&conn, &workspace_id, &master_key(), 10).await.unwrap();
    assert_eq!(drained, 2);
    assert_eq!(integration_pending_action_repo::list_batch(&conn, &workspace_id, 10).unwrap().len(), 0);

    let executions = integration_log_service::list_executions(&conn, &workspace_id, &Default::default()).unwrap();
    assert_eq!(executions.iter().filter(|e| e.execution_type == "connector_action").count(), 2);
}

#[test]
fn secret_encrypt_decrypt_round_trip_matches_the_connection_service_convention() {
    // Sanity check that the master key shape this file's tests use round-
    // trips through the real encryption path connection_service relies on.
    let (ciphertext, nonce) = secret_service::encrypt(&master_key(), "hunter2").unwrap();
    let decrypted = secret_service::decrypt(&master_key(), &ciphertext, &nonce).unwrap();
    assert_eq!(decrypted, "hunter2");
}

const FLAT_FORM_OPENAPI_JSON: &str = r#"{
  "openapi": "3.0.0",
  "info": {"title": "Flat Form API", "version": "1.0.0"},
  "paths": {
    "/customers": {
      "post": {
        "operationId": "createCustomer",
        "requestBody": {
          "required": true,
          "content": {
            "application/x-www-form-urlencoded": {
              "schema": {"type": "object", "properties": {"name": {"type": "string"}, "email": {"type": "string"}}}
            }
          }
        }
      }
    }
  }
}"#;

const NESTED_FORM_OPENAPI_JSON: &str = r#"{
  "openapi": "3.0.0",
  "info": {"title": "Nested Form API", "version": "1.0.0"},
  "paths": {
    "/customers": {
      "post": {
        "operationId": "createCustomer",
        "requestBody": {
          "required": true,
          "content": {
            "application/x-www-form-urlencoded": {
              "schema": {"type": "object", "properties": {"name": {"type": "string"}, "address": {"type": "object", "properties": {"city": {"type": "string"}}}}}
            }
          }
        }
      }
    }
  }
}"#;

/// Connector Template Library Phase 2's conservative boundary: a *flat*
/// form-urlencoded body (every top-level property a bare primitive, like
/// Stripe/Twilio) is recognized as a first-class supported media type with
/// zero warnings and a real `request_content_type`; a *nested* one falls
/// back to today's pre-existing "treated as opaque JSON" warning path,
/// unchanged - and critically, its `request_content_type` must come back
/// `None` (i.e. `application/json`, matching what the warning promises),
/// not the form-urlencoded string, or `connector_execution_service::
/// build_and_send` would silently mis-flatten a nested body at send time.
#[test]
fn flat_form_body_is_recognized_but_a_nested_one_falls_back_to_opaque() {
    let flat = connector_service::preview_import(FLAT_FORM_OPENAPI_JSON, "json").unwrap();
    assert!(flat.warnings.is_empty(), "a flat form-urlencoded body should import with zero warnings: {:?}", flat.warnings);
    let flat_op = flat.operations.iter().find(|o| o.operation_id == "createCustomer").unwrap();
    assert_eq!(flat_op.request_content_type.as_deref(), Some("application/x-www-form-urlencoded"));

    let nested = connector_service::preview_import(NESTED_FORM_OPENAPI_JSON, "json").unwrap();
    assert!(
        nested.warnings.iter().any(|w| w.contains("treated as opaque JSON")),
        "a nested form-urlencoded body should still warn and fall back: {:?}",
        nested.warnings
    );
    let nested_op = nested.operations.iter().find(|o| o.operation_id == "createCustomer").unwrap();
    assert_eq!(nested_op.request_content_type.as_deref(), Some("application/json"), "the opaque fallback must not be mistaken for a real form body downstream");

    // The distinction survives all the way into a saved Connector Action,
    // not just the preview.
    let (conn, workspace_id, admin_id) = setup_workspace();
    let import_input = ConnectorImportInput { name: "Flat".into(), description: None, spec_text: FLAT_FORM_OPENAPI_JSON.into(), spec_format: "json".into(), selected_operation_ids: vec!["createCustomer".into()] };
    let connector = connector_service::import(&conn, &workspace_id, &import_input, Some(&admin_id)).unwrap();
    assert_eq!(connector.actions[0].request_content_type.as_deref(), Some("application/x-www-form-urlencoded"));

    let import_input2 = ConnectorImportInput { name: "Nested".into(), description: None, spec_text: NESTED_FORM_OPENAPI_JSON.into(), spec_format: "json".into(), selected_operation_ids: vec!["createCustomer".into()] };
    let connector2 = connector_service::import(&conn, &workspace_id, &import_input2, Some(&admin_id)).unwrap();
    assert_eq!(connector2.actions[0].request_content_type, None, "nested form body must persist as the implicit application/json default, not form-urlencoded");
}

/// A minimal raw-socket HTTP/1.1 server that captures the Content-Type
/// header and the full request body, then echoes both back in its JSON
/// response - proves an actual outbound request used the right encoding,
/// not just that `build_and_send` picked a code path.
fn spawn_capturing_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line);
            let mut content_type = String::new();
            let mut content_length: usize = 0;
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) if line == "\r\n" || line.is_empty() => break,
                    Ok(_) => {
                        let lower = line.to_ascii_lowercase();
                        if let Some(v) = lower.strip_prefix("content-type:") {
                            content_type = v.trim().to_string();
                        } else if let Some(v) = lower.strip_prefix("content-length:") {
                            content_length = v.trim().parse().unwrap_or(0);
                        }
                    }
                    Err(_) => break,
                }
            }
            let mut body_buf = vec![0u8; content_length];
            let _ = std::io::Read::read_exact(&mut reader, &mut body_buf);
            let body_str = String::from_utf8_lossy(&body_buf).replace('"', "'");
            let echoed_line = request_line.trim().replace('"', "'");
            let body = format!("{{\"ok\":true,\"request_line\":\"{echoed_line}\",\"content_type\":\"{content_type}\",\"captured_body\":\"{body_str}\"}}");
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let mut stream = stream;
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

/// End-to-end: a flat form-urlencoded Connector Action actually sends
/// `Content-Type: application/x-www-form-urlencoded` with correctly
/// encoded fields against a real local listener - not JSON, matching
/// what Stripe/Twilio's real APIs require.
#[tokio::test]
async fn execute_sends_a_form_urlencoded_body_with_the_right_content_type() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_capturing_server();
    let base_url = format!("http://127.0.0.1:{port}");
    let reference_key = setup_reference(&conn, &workspace_id, &admin_id, &base_url);

    let import_input = ConnectorImportInput { name: "Flat".into(), description: None, spec_text: FLAT_FORM_OPENAPI_JSON.into(), spec_format: "json".into(), selected_operation_ids: vec!["createCustomer".into()] };
    let connector = connector_service::import(&conn, &workspace_id, &import_input, Some(&admin_id)).unwrap();
    assert_eq!(connector.actions[0].request_content_type.as_deref(), Some("application/x-www-form-urlencoded"));

    let params = serde_json::json!({"body": {"name": "Ada Lovelace", "email": "ada@example.com"}});
    let result = connector_execution_service::execute(&conn, &workspace_id, &master_key(), &connector.id, "createCustomer", &reference_key, &params, None).await.unwrap();

    assert!(result.ok, "{result:?}");
    let content_type = result.response_body["content_type"].as_str().unwrap();
    assert_eq!(content_type, "application/x-www-form-urlencoded");
    let captured_body = result.response_body["captured_body"].as_str().unwrap();
    assert!(captured_body.contains("name=Ada"), "form body should be present, url-encoded: {captured_body}");
    assert!(captured_body.contains("email=ada%40example.com") || captured_body.contains("email=ada@example.com"), "email field should be present: {captured_body}");
}
