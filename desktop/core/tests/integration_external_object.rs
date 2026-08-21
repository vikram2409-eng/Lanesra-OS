//! Integration Hub (spec §16): proves `external_object_service` -
//! read-only External/Virtual Objects surfaced through an existing
//! Connection. Tested against a real local HTTP listener spun up inside
//! the test itself (standing in for "the external system"), not a live
//! third-party API - but the exact same request-building/response-
//! parsing code path a live REST/OData API would hit.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::integration::{ConnectionInput, ExternalObjectInput, FieldMapEntry, IntegrationJobInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{connection_service, external_object_service, integration_job_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "External Objects Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [11u8; 32]
}

/// A minimal raw-socket HTTP/1.1 server that always answers 200 with a
/// fixed JSON body, ignoring the request entirely - enough to prove
/// `list_records` performs a real GET against the bound Connection's
/// base URL + resource path and parses whatever shape comes back.
fn spawn_json_server(body: &'static str) -> u16 {
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
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let mut stream = stream;
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn spawn_status_server(status_line: &'static str) -> u16 {
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
            let body = "not found";
            let response = format!("{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let mut stream = stream;
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn setup_rest_connection(conn: &rusqlite::Connection, workspace_id: &str, admin_id: &str, base_url: &str) -> String {
    connection_service::create(
        conn,
        workspace_id,
        &master_key(),
        &ConnectionInput { name: "External system".into(), connection_type: "rest".into(), base_url: Some(base_url.to_string()), auth_mode: "none".into(), secret_value: None, config_json: "{}".into(), owner_user_id: None },
        Some(admin_id),
    )
    .unwrap()
    .id
}

fn field_map(source: &str, target: &str) -> Vec<FieldMapEntry> {
    vec![FieldMapEntry { source_column: source.into(), target_field: target.into(), transform: None, default_value: None, constant: None }]
}

#[test]
fn create_rejects_a_non_rest_odata_connection() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let connection = connection_service::create(
        &conn, &workspace_id, &master_key(),
        &ConnectionInput { name: "Some SMTP".into(), connection_type: "smtp".into(), base_url: None, auth_mode: "none".into(), secret_value: None, config_json: "{}".into(), owner_user_id: None },
        Some(&admin_id),
    ).unwrap();
    let input = ExternalObjectInput { object_key: "widgets".into(), display_name: "Widgets".into(), connection_id: connection.id, resource_path: "/widgets".into(), field_map: vec![], cache_ttl_seconds: None };
    let err = external_object_service::create(&conn, &workspace_id, &input, Some(&admin_id));
    assert!(err.is_err(), "a connection that isn't rest/odata must be rejected");
}

#[test]
fn create_rejects_a_duplicate_object_key_and_non_admins() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let connection_id = setup_rest_connection(&conn, &workspace_id, &admin_id, "http://127.0.0.1:1");
    let input = ExternalObjectInput { object_key: "widgets".into(), display_name: "Widgets".into(), connection_id: connection_id.clone(), resource_path: "/widgets".into(), field_map: vec![], cache_ttl_seconds: None };

    assert!(external_object_service::create(&conn, &workspace_id, &input, None).is_err(), "non-admins must be rejected");

    external_object_service::create(&conn, &workspace_id, &input, Some(&admin_id)).unwrap();
    let dup = ExternalObjectInput { object_key: "widgets".into(), display_name: "Widgets Again".into(), connection_id, resource_path: "/widgets2".into(), field_map: vec![], cache_ttl_seconds: None };
    assert!(external_object_service::create(&conn, &workspace_id, &dup, Some(&admin_id)).is_err(), "object_key must be unique per workspace");
}

#[test]
fn list_and_delete_round_trip() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let connection_id = setup_rest_connection(&conn, &workspace_id, &admin_id, "http://127.0.0.1:1");
    let input = ExternalObjectInput { object_key: "widgets".into(), display_name: "Widgets".into(), connection_id, resource_path: "/widgets".into(), field_map: vec![], cache_ttl_seconds: Some(60) };
    let created = external_object_service::create(&conn, &workspace_id, &input, Some(&admin_id)).unwrap();
    assert_eq!(created.cache_ttl_seconds, Some(60));

    let listed = external_object_service::list_for_workspace(&conn, &workspace_id).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].object_key, "widgets");

    external_object_service::delete(&conn, &workspace_id, &created.id, Some(&admin_id)).unwrap();
    assert!(external_object_service::list_for_workspace(&conn, &workspace_id).unwrap().is_empty());
}

#[test]
fn delete_is_blocked_while_an_integration_job_still_points_at_it() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let connection_id = setup_rest_connection(&conn, &workspace_id, &admin_id, "http://127.0.0.1:1");
    let input = ExternalObjectInput { object_key: "widgets".into(), display_name: "Widgets".into(), connection_id, resource_path: "/widgets".into(), field_map: vec![], cache_ttl_seconds: None };
    let external_object = external_object_service::create(&conn, &workspace_id, &input, Some(&admin_id)).unwrap();
    let job = integration_job_service::create(
        &conn, &workspace_id,
        &IntegrationJobInput { name: "Sync Widgets".into(), external_object_id: external_object.id.clone(), target_object_key: "Company".into(), match_key: "name".into(), cursor_field: None, interval_minutes: 30 },
        Some(&admin_id),
    ).unwrap();

    let err = external_object_service::delete(&conn, &workspace_id, &external_object.id, Some(&admin_id));
    assert!(err.is_err(), "an External Object with a dependent Job must not be deletable");

    integration_job_service::delete(&conn, &workspace_id, &job.id, Some(&admin_id)).unwrap();
    external_object_service::delete(&conn, &workspace_id, &external_object.id, Some(&admin_id)).unwrap();
    assert!(external_object_service::list_for_workspace(&conn, &workspace_id).unwrap().is_empty());
}

#[tokio::test]
async fn list_records_handles_a_bare_json_array() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_json_server(r#"[{"sku":"A1","label":"Widget A"},{"sku":"B2","label":"Widget B"}]"#);
    let base_url = format!("http://127.0.0.1:{port}");
    let connection_id = setup_rest_connection(&conn, &workspace_id, &admin_id, &base_url);
    let input = ExternalObjectInput { object_key: "widgets".into(), display_name: "Widgets".into(), connection_id, resource_path: "/widgets".into(), field_map: vec![], cache_ttl_seconds: None };
    external_object_service::create(&conn, &workspace_id, &input, Some(&admin_id)).unwrap();

    let records = external_object_service::list_records(&conn, &workspace_id, &master_key(), "widgets").await.unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["sku"], "A1");
    assert_eq!(records[1]["label"], "Widget B");
}

#[tokio::test]
async fn list_records_handles_an_object_wrapped_array_and_applies_the_field_map() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_json_server(r#"{"data":[{"sku":"A1","label":"Widget A"}]}"#);
    let base_url = format!("http://127.0.0.1:{port}");
    let connection_id = setup_rest_connection(&conn, &workspace_id, &admin_id, &base_url);
    let input = ExternalObjectInput {
        object_key: "widgets".into(), display_name: "Widgets".into(), connection_id, resource_path: "/widgets".into(),
        field_map: field_map("sku", "product_code"), cache_ttl_seconds: None,
    };
    external_object_service::create(&conn, &workspace_id, &input, Some(&admin_id)).unwrap();

    let records = external_object_service::list_records(&conn, &workspace_id, &master_key(), "widgets").await.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["product_code"], "A1", "field_map should remap sku -> product_code");
    assert!(records[0].get("label").is_none(), "an unmapped source field should not leak through once a field_map is set");
}

#[tokio::test]
async fn list_records_also_recognizes_items_results_and_value_wrapper_keys() {
    for key in ["items", "results", "value"] {
        let (conn, workspace_id, admin_id) = setup_workspace();
        let body: String = format!(r#"{{"{key}":[{{"sku":"Z9"}}]}}"#);
        let body: &'static str = Box::leak(body.into_boxed_str());
        let port = spawn_json_server(body);
        let base_url = format!("http://127.0.0.1:{port}");
        let connection_id = setup_rest_connection(&conn, &workspace_id, &admin_id, &base_url);
        let input = ExternalObjectInput { object_key: "widgets".into(), display_name: "Widgets".into(), connection_id, resource_path: "/widgets".into(), field_map: vec![], cache_ttl_seconds: None };
        external_object_service::create(&conn, &workspace_id, &input, Some(&admin_id)).unwrap();

        let records = external_object_service::list_records(&conn, &workspace_id, &master_key(), "widgets").await.unwrap();
        assert_eq!(records.len(), 1, "wrapper key '{key}' should be recognized");
        assert_eq!(records[0]["sku"], "Z9");
    }
}

#[tokio::test]
async fn list_records_rejects_a_json_object_with_no_recognized_array_key() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_json_server(r#"{"unexpected":"shape"}"#);
    let base_url = format!("http://127.0.0.1:{port}");
    let connection_id = setup_rest_connection(&conn, &workspace_id, &admin_id, &base_url);
    let input = ExternalObjectInput { object_key: "widgets".into(), display_name: "Widgets".into(), connection_id, resource_path: "/widgets".into(), field_map: vec![], cache_ttl_seconds: None };
    external_object_service::create(&conn, &workspace_id, &input, Some(&admin_id)).unwrap();

    let err = external_object_service::list_records(&conn, &workspace_id, &master_key(), "widgets").await;
    assert!(err.is_err(), "an object with no recognized array key must be a clear validation error, not a silent empty list");
}

#[tokio::test]
async fn list_records_surfaces_a_non_success_http_status() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let port = spawn_status_server("HTTP/1.1 404 Not Found");
    let base_url = format!("http://127.0.0.1:{port}");
    let connection_id = setup_rest_connection(&conn, &workspace_id, &admin_id, &base_url);
    let input = ExternalObjectInput { object_key: "widgets".into(), display_name: "Widgets".into(), connection_id, resource_path: "/widgets".into(), field_map: vec![], cache_ttl_seconds: None };
    external_object_service::create(&conn, &workspace_id, &input, Some(&admin_id)).unwrap();

    let key = master_key();
    let err = external_object_service::list_records(&conn, &workspace_id, &key, "widgets").await;
    assert!(err.is_err());
    assert!(format!("{}", err.unwrap_err()).contains("404"));
}

#[tokio::test]
async fn list_records_errors_for_an_unknown_object_key() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let err = external_object_service::list_records(&conn, &workspace_id, &master_key(), "no-such-object").await;
    assert!(err.is_err());
}
