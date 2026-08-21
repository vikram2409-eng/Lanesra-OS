//! Integration Hub (spec §10): proves `webhook_service` end-to-end - HMAC
//! signing a receiver can independently verify, retry-on-5xx with a
//! permanent-4xx short-circuit, Degraded after repeated failure, object
//! scoping, and the `event_hooks` -> `integration_pending_events` ->
//! `drain_pending_events` fan-out path - against a real local HTTP
//! listener spun up inside the test itself.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use hmac::{Hmac, Mac};
use sha2::Sha256;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::integration::{ConnectionInput, WebhookInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, webhook_service};
use lanesra_core::services::{connection_service, secret_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Webhook Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service_setup(&conn, &setup);
    (conn, workspace, admin)
}

fn workspace_service_setup(conn: &rusqlite::Connection, setup: &WorkspaceSetup) -> (String, String) {
    let (workspace, admin) = lanesra_core::services::workspace_service::first_run_setup(conn, setup).unwrap();
    (workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [9u8; 32]
}

#[derive(Clone)]
struct Captured {
    method_and_path: String,
    signature: String,
    body: String,
}

/// A raw-socket HTTP/1.1 server whose status code for each request is
/// pulled off `responses` in order (repeating the last one once
/// exhausted) - lets a test script "first two calls 500, then 200" to
/// prove retry actually happens, without a mocking framework.
fn spawn_scripted_server(responses: Vec<u16>) -> (u16, Arc<Mutex<Vec<Captured>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = captured.clone();
    let counter = Arc::new(AtomicUsize::new(0));
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line);
            let mut signature = String::new();
            let mut content_length = 0usize;
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) if line == "\r\n" => break,
                    Ok(_) => {
                        // Header names are case-insensitive on the wire -
                        // reqwest/hyper may send this one in any case, so
                        // match on the lowercased line like content-length
                        // already does below, not a literal-case prefix.
                        let lower = line.to_ascii_lowercase();
                        if let Some(v) = lower.strip_prefix("x-lanesra-signature: ") {
                            signature = v.trim().to_string();
                        }
                        if let Some(v) = lower.strip_prefix("content-length: ") {
                            content_length = v.trim().parse().unwrap_or(0);
                        }
                    }
                    Err(_) => break,
                }
            }
            let mut body_buf = vec![0u8; content_length];
            let _ = reader.read_exact(&mut body_buf);
            let body = String::from_utf8_lossy(&body_buf).to_string();

            let idx = counter.fetch_add(1, Ordering::SeqCst);
            let status = responses.get(idx).or_else(|| responses.last()).copied().unwrap_or(200);
            captured_clone.lock().unwrap().push(Captured { method_and_path: request_line.trim().to_string(), signature, body });

            let reason = match status { 200 => "OK", 500 => "Internal Server Error", 400 => "Bad Request", other => if other >= 500 { "Server Error" } else { "Error" } };
            let response_body = "{}";
            let response = format!("HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response_body.len(), response_body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, captured)
}

fn make_webhook_connection(conn: &rusqlite::Connection, workspace_id: &str, admin_id: &str, base_url: &str) -> String {
    connection_service::create(
        conn,
        workspace_id,
        &master_key(),
        &ConnectionInput { name: "Local receiver".into(), connection_type: "webhook".into(), base_url: Some(base_url.to_string()), auth_mode: "none".into(), secret_value: None, config_json: "{}".into(), owner_user_id: None },
        Some(admin_id),
    )
    .unwrap()
    .id
}

/// Reads the webhook's real signing secret straight out of the database,
/// the same way `webhook_service::resolve_webhook_secret` does - so the
/// test can independently recompute the HMAC and compare it against what
/// the receiver actually got, proving §10.3's signature scheme end to end.
fn read_webhook_secret(conn: &rusqlite::Connection, master_key: &[u8; 32], webhook_id: &str) -> String {
    let secret_id: Option<String> = conn.query_row("SELECT secret_id FROM integration_webhooks WHERE id = ?1", [webhook_id], |r| r.get(0)).unwrap();
    let stored = lanesra_core::repositories::integration_secret_repo::get(conn, &secret_id.unwrap()).unwrap().unwrap();
    secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce).unwrap()
}

fn expected_signature(secret: &str, body: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body.as_bytes());
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

#[test]
fn create_requires_a_webhook_type_connection_and_a_known_event_type() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let rest_connection = connection_service::create(
        &conn, &workspace_id, &master_key(),
        &ConnectionInput { name: "Not a webhook".into(), connection_type: "rest".into(), base_url: Some("http://example.invalid".into()), auth_mode: "none".into(), secret_value: None, config_json: "{}".into(), owner_user_id: None },
        Some(&admin_id),
    ).unwrap();

    let err = webhook_service::create(&conn, &workspace_id, &master_key(), &WebhookInput { name: "Bad".into(), connection_id: rest_connection.id, event_types: vec!["record.created".into()], object_scope: None, filter_json: None, payload_version: None, retry_policy_json: None }, Some(&admin_id));
    assert!(err.is_err(), "a non-webhook connection must be rejected");

    let webhook_connection_id = make_webhook_connection(&conn, &workspace_id, &admin_id, "http://example.invalid");
    let err = webhook_service::create(&conn, &workspace_id, &master_key(), &WebhookInput { name: "Bad event".into(), connection_id: webhook_connection_id, event_types: vec!["not.a.real.event".into()], object_scope: None, filter_json: None, payload_version: None, retry_policy_json: None }, Some(&admin_id));
    assert!(err.is_err(), "an unknown event type must be rejected");
}

#[tokio::test]
async fn fire_event_signs_the_payload_so_the_receiver_can_independently_verify_it() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let (port, captured) = spawn_scripted_server(vec![200]);
    let connection_id = make_webhook_connection(&conn, &workspace_id, &admin_id, &format!("http://127.0.0.1:{port}"));
    let webhook = webhook_service::create(&conn, &workspace_id, &master_key(), &WebhookInput { name: "On create".into(), connection_id, event_types: vec!["record.created".into()], object_scope: None, filter_json: None, payload_version: None, retry_policy_json: None }, Some(&admin_id)).unwrap();

    webhook_service::test_delivery(&conn, &workspace_id, &master_key(), &webhook.id, Some(&admin_id)).await.unwrap();

    let calls = captured.lock().unwrap().clone();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].method_and_path.starts_with("POST "), "webhook delivery is always a POST: {}", calls[0].method_and_path);
    let secret = read_webhook_secret(&conn, &master_key(), &webhook.id);
    assert_eq!(calls[0].signature, expected_signature(&secret, &calls[0].body), "receiver must be able to recompute the same signature");
    assert!(calls[0].body.contains("This is a test delivery"));

    let deliveries = webhook_service::list_deliveries(&conn, &workspace_id, &webhook.id).unwrap();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].status, "succeeded");
    assert_eq!(deliveries[0].http_status, Some(200));
}

#[tokio::test]
async fn fire_event_retries_on_5xx_and_succeeds_once_the_receiver_recovers() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let (port, captured) = spawn_scripted_server(vec![500, 500, 200]);
    let connection_id = make_webhook_connection(&conn, &workspace_id, &admin_id, &format!("http://127.0.0.1:{port}"));
    let webhook = webhook_service::create(&conn, &workspace_id, &master_key(), &WebhookInput { name: "Retry test".into(), connection_id, event_types: vec!["record.created".into()], object_scope: None, filter_json: None, payload_version: None, retry_policy_json: None }, Some(&admin_id)).unwrap();

    webhook_service::test_delivery(&conn, &workspace_id, &master_key(), &webhook.id, Some(&admin_id)).await.unwrap();

    assert_eq!(captured.lock().unwrap().len(), 3, "two failures then a success = three attempts");
    let deliveries = webhook_service::list_deliveries(&conn, &workspace_id, &webhook.id).unwrap();
    assert_eq!(deliveries.len(), 3);
    // Most recent first (list_deliveries orders by created_at DESC).
    assert_eq!(deliveries[0].status, "succeeded");
    assert_eq!(deliveries[0].attempt_number, 3);
    assert_eq!(deliveries[1].status, "failed");
    assert_eq!(deliveries[2].status, "failed");
}

#[tokio::test]
async fn fire_event_does_not_retry_a_permanent_4xx() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let (port, captured) = spawn_scripted_server(vec![400]);
    let connection_id = make_webhook_connection(&conn, &workspace_id, &admin_id, &format!("http://127.0.0.1:{port}"));
    let webhook = webhook_service::create(&conn, &workspace_id, &master_key(), &WebhookInput { name: "4xx test".into(), connection_id, event_types: vec!["record.created".into()], object_scope: None, filter_json: None, payload_version: None, retry_policy_json: None }, Some(&admin_id)).unwrap();

    webhook_service::test_delivery(&conn, &workspace_id, &master_key(), &webhook.id, Some(&admin_id)).await.unwrap();

    assert_eq!(captured.lock().unwrap().len(), 1, "a permanent 4xx must not be retried");
    let deliveries = webhook_service::list_deliveries(&conn, &workspace_id, &webhook.id).unwrap();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].status, "failed");
}

#[tokio::test]
async fn webhook_degrades_after_five_consecutive_failed_events() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let (port, _captured) = spawn_scripted_server(vec![500]);
    let connection_id = make_webhook_connection(&conn, &workspace_id, &admin_id, &format!("http://127.0.0.1:{port}"));
    let webhook = webhook_service::create(&conn, &workspace_id, &master_key(), &WebhookInput { name: "Degrade test".into(), connection_id, event_types: vec!["record.created".into()], object_scope: None, filter_json: None, payload_version: None, retry_policy_json: None }, Some(&admin_id)).unwrap();

    for _ in 0..5 {
        webhook_service::test_delivery(&conn, &workspace_id, &master_key(), &webhook.id, Some(&admin_id)).await.unwrap();
    }

    let refreshed = webhook_service::list_for_workspace(&conn, &workspace_id).unwrap();
    let this_one = refreshed.iter().find(|w| w.id == webhook.id).unwrap();
    assert_eq!(this_one.status, "degraded");
}

#[tokio::test]
async fn object_scope_filters_which_events_a_webhook_receives() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let (port, captured) = spawn_scripted_server(vec![200]);
    let connection_id = make_webhook_connection(&conn, &workspace_id, &admin_id, &format!("http://127.0.0.1:{port}"));
    webhook_service::create(&conn, &workspace_id, &master_key(), &WebhookInput { name: "Companies only".into(), connection_id, event_types: vec!["record.created".into()], object_scope: Some("Company".into()), filter_json: None, payload_version: None, retry_policy_json: None }, Some(&admin_id)).unwrap();

    // event_hooks::emit -> integration_pending_events -> drain_pending_events
    // is the real path an entity create goes through; created via
    // company_service::create directly exercises that full chain.
    company_service::create(&conn, &workspace_id, &CompanyInput { name: "Acme".into(), status: "Prospect".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }, Some(&admin_id)).unwrap();
    let drained = webhook_service::drain_pending_events(&conn, &master_key()).await.unwrap();
    assert_eq!(drained, 1);
    assert_eq!(captured.lock().unwrap().len(), 1, "the Company creation should reach this Company-scoped webhook");
}

#[tokio::test]
async fn pause_stops_delivery_and_reactivate_resumes_it() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let (port, captured) = spawn_scripted_server(vec![200]);
    let connection_id = make_webhook_connection(&conn, &workspace_id, &admin_id, &format!("http://127.0.0.1:{port}"));
    let webhook = webhook_service::create(&conn, &workspace_id, &master_key(), &WebhookInput { name: "Pausable".into(), connection_id, event_types: vec!["record.created".into()], object_scope: None, filter_json: None, payload_version: None, retry_policy_json: None }, Some(&admin_id)).unwrap();

    webhook_service::pause(&conn, &workspace_id, &webhook.id, Some(&admin_id)).unwrap();
    company_service::create(&conn, &workspace_id, &CompanyInput { name: "Acme".into(), status: "Prospect".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }, Some(&admin_id)).unwrap();
    webhook_service::drain_pending_events(&conn, &master_key()).await.unwrap();
    assert!(captured.lock().unwrap().is_empty(), "a paused webhook must not receive events");

    webhook_service::reactivate(&conn, &workspace_id, &webhook.id, Some(&admin_id)).unwrap();
    company_service::create(&conn, &workspace_id, &CompanyInput { name: "Globex".into(), status: "Prospect".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }, Some(&admin_id)).unwrap();
    webhook_service::drain_pending_events(&conn, &master_key()).await.unwrap();
    assert_eq!(captured.lock().unwrap().len(), 1, "a reactivated webhook should receive new events again");
}

#[test]
fn delete_removes_the_subscription() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let connection_id = make_webhook_connection(&conn, &workspace_id, &admin_id, "http://example.invalid");
    let webhook = webhook_service::create(&conn, &workspace_id, &master_key(), &WebhookInput { name: "Removable".into(), connection_id, event_types: vec!["record.created".into()], object_scope: None, filter_json: None, payload_version: None, retry_policy_json: None }, Some(&admin_id)).unwrap();
    webhook_service::delete(&conn, &workspace_id, &webhook.id, Some(&admin_id)).unwrap();
    assert!(webhook_service::list_for_workspace(&conn, &workspace_id).unwrap().is_empty());
}
