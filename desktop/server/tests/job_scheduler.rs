//! Integration Hub (spec §15): proves the server's real background
//! scheduler thread (`lanesra_server::job_scheduler`) actually fires an
//! Integration Job on its own, with nobody calling `run_now` - a real
//! local HTTP listener standing in for "the external system", a real
//! on-disk SQLite file (the scheduler opens its own connection to it -
//! see that module's doc comment for why), and a short tick interval so
//! the test doesn't need to wait a real 60s.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use hmac::{Hmac, Mac};
use sha2::Sha256;

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::integration::{
    ConnectionInput, ExternalObjectInput, IntegrationJobInput, WebhookInput,
};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    company_service, connection_service, external_object_service, integration_job_service,
    secret_service, webhook_service, workspace_service,
};

fn master_key() -> [u8; 32] {
    [21u8; 32]
}

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

#[tokio::test]
async fn the_background_scheduler_thread_runs_a_due_job_with_no_manual_run_now() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("lanesra.sqlite3");
    let key_file_path = dir.path().join("secret.key");

    let (workspace_id, admin_id, job_id) = {
        let conn = open_workspace_db(&db_path).unwrap();
        let setup = WorkspaceSetup {
            business_name: "Scheduler Co".into(),
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

        let port = spawn_json_server(r#"[{"name":"Scheduled Co","status":"Prospect"}]"#);
        let connection = connection_service::create(
            &conn,
            &workspace.id,
            &master_key(),
            &ConnectionInput {
                name: "External CRM".into(),
                connection_type: "rest".into(),
                base_url: Some(format!("http://127.0.0.1:{port}")),
                auth_mode: "none".into(),
                secret_value: None,
                config_json: "{}".into(),
                owner_user_id: None,
            },
            Some(&admin.id),
        )
        .unwrap();
        let external_object = external_object_service::create(
            &conn,
            &workspace.id,
            &ExternalObjectInput {
                object_key: "crm_companies".into(),
                display_name: "CRM Companies".into(),
                connection_id: connection.id,
                resource_path: "/companies".into(),
                field_map: vec![],
                cache_ttl_seconds: None,
            },
            Some(&admin.id),
        )
        .unwrap();
        let job = integration_job_service::create(
            &conn,
            &workspace.id,
            &IntegrationJobInput {
                name: "Sync CRM Companies".into(),
                external_object_id: external_object.id,
                target_object_key: "Company".into(),
                match_key: "name".into(),
                cursor_field: None,
                interval_minutes: 1,
            },
            Some(&admin.id),
        )
        .unwrap();

        // Never run before -> due immediately once the scheduler ticks.
        assert!(job.last_run_at.is_none());
        (workspace.id, admin.id, job.id)
    };
    // The setup connection above is dropped here - the scheduler opens
    // its own, exactly as it will against a real workspace database.

    lanesra_server::job_scheduler::spawn(
        db_path.clone(),
        key_file_path,
        Duration::from_millis(150),
    );

    // Poll (from a third, independent connection) for the job run to
    // land - proves the scheduler thread did real work on its own,
    // without the test ever calling run_now.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let conn = open_workspace_db(&db_path).unwrap();
        let job = integration_job_service::get(&conn, &workspace_id, &job_id).unwrap();
        if job.last_run_status.is_some() {
            assert_eq!(
                job.last_run_status.as_deref(),
                Some("success"),
                "the scheduled run should have succeeded"
            );
            let companies = company_service::list(&conn, &workspace_id).unwrap();
            assert!(
                companies.iter().any(|c| c.name == "Scheduled Co"),
                "the scheduler's own run should have created the record: {companies:?}"
            );
            break;
        }
        if std::time::Instant::now() > deadline {
            panic!("the background scheduler never ran the due job within the deadline");
        }
    }
    let _ = admin_id; // kept for clarity/documentation of setup, not asserted on further
}

// ---------------------------------------------------------------------
// WP-0182: outbound webhooks for real record events
// ---------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Captured {
    request_line: String,
    signature: String,
    body: String,
}

/// A local receiver that answers 200 to everything and records each
/// request's line, `X-Lanesra-Signature` header and body.
fn spawn_capture_server() -> (u16, Arc<Mutex<Vec<Captured>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let sink = captured.clone();
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
                    Ok(_) if line == "\r\n" || line.is_empty() => break,
                    Ok(_) => {
                        let lower = line.to_ascii_lowercase();
                        if let Some(v) = lower.strip_prefix("content-length:") {
                            content_length = v.trim().parse().unwrap_or(0);
                        }
                        if lower.starts_with("x-lanesra-signature:") {
                            signature = line["x-lanesra-signature:".len()..].trim().to_string();
                        }
                    }
                    Err(_) => break,
                }
            }
            let mut body = vec![0u8; content_length];
            let _ = reader.read_exact(&mut body);
            sink.lock().unwrap().push(Captured {
                request_line: request_line.trim().to_string(),
                signature,
                body: String::from_utf8_lossy(&body).to_string(),
            });
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
        }
    });
    (port, captured)
}

fn expected_signature(secret: &str, body: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body.as_bytes());
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

fn read_webhook_secret(
    conn: &rusqlite::Connection,
    master_key: &[u8; 32],
    webhook_id: &str,
) -> String {
    let secret_id: Option<String> = conn
        .query_row(
            "SELECT secret_id FROM integration_webhooks WHERE id = ?1",
            [webhook_id],
            |r| r.get(0),
        )
        .unwrap();
    let stored =
        lanesra_core::repositories::integration_secret_repo::get(conn, &secret_id.unwrap())
            .unwrap()
            .unwrap();
    secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce).unwrap()
}

fn company_input(name: &str, status: &str) -> CompanyInput {
    CompanyInput {
        name: name.into(),
        status: status.into(),
        owner_user_id: None,
        tax_number: None,
        billing_address: None,
        shipping_address: None,
        tags: None,
        notes: None,
        ..Default::default()
    }
}

struct Fixture {
    dir: tempfile::TempDir,
    db_path: std::path::PathBuf,
    key_file_path: std::path::PathBuf,
    workspace_id: String,
    admin_id: String,
    key: [u8; 32],
}

/// A workspace on disk whose key file the scheduler will resolve itself,
/// so a webhook created here with `key` decrypts inside the scheduler.
fn webhook_fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("lanesra.sqlite3");
    let key_file_path = dir.path().join("secret.key");
    let conn = open_workspace_db(&db_path).unwrap();
    let setup = WorkspaceSetup {
        business_name: "Webhook Scheduler Co".into(),
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
    let key = secret_service::resolve_master_key(&key_file_path).unwrap();
    Fixture {
        dir,
        db_path,
        key_file_path,
        workspace_id: workspace.id,
        admin_id: admin.id,
        key,
    }
}

fn add_webhook(
    f: &Fixture,
    port: u16,
    name: &str,
    events: &[&str],
    object_scope: Option<&str>,
) -> String {
    let conn = open_workspace_db(&f.db_path).unwrap();
    let connection = connection_service::create(
        &conn,
        &f.workspace_id,
        &f.key,
        &ConnectionInput {
            name: format!("{name} receiver"),
            connection_type: "webhook".into(),
            base_url: Some(format!("http://127.0.0.1:{port}")),
            auth_mode: "none".into(),
            secret_value: None,
            config_json: "{}".into(),
            owner_user_id: None,
        },
        Some(&f.admin_id),
    )
    .unwrap();
    webhook_service::create(
        &conn,
        &f.workspace_id,
        &f.key,
        &WebhookInput {
            name: name.into(),
            connection_id: connection.id,
            event_types: events.iter().map(|e| e.to_string()).collect(),
            object_scope: object_scope.map(|s| s.to_string()),
            filter_json: None,
            payload_version: None,
            retry_policy_json: None,
        },
        Some(&f.admin_id),
    )
    .unwrap()
    .id
}

fn pending_event_count(f: &Fixture) -> i64 {
    let conn = open_workspace_db(&f.db_path).unwrap();
    conn.query_row("SELECT COUNT(*) FROM integration_pending_events", [], |r| {
        r.get(0)
    })
    .unwrap()
}

async fn wait_until(what: &str, mut done: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    loop {
        if done() {
            return;
        }
        if std::time::Instant::now() > deadline {
            panic!("timed out waiting for: {what}");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// Covers: REQ-INT-001 (WP-0182 AC-1, AC-2)
#[tokio::test]
async fn the_background_scheduler_delivers_signed_webhooks_for_real_record_changes() {
    let f = webhook_fixture();
    let (port, captured) = spawn_capture_server();
    let webhook_id = add_webhook(
        &f,
        port,
        "All record events",
        &["record.created", "record.updated", "record.archived"],
        None,
    );

    // The record changes happen BEFORE the scheduler starts and nobody ever
    // calls `drain_pending_events` or the Test button: the only thing that
    // can deliver them is the scheduler's own tick.
    let company_id = {
        let conn = open_workspace_db(&f.db_path).unwrap();
        let company = company_service::create(
            &conn,
            &f.workspace_id,
            &company_input("Hooked Co", "Prospect"),
            Some(&f.admin_id),
        )
        .unwrap();
        company_service::update(
            &conn,
            &company.id,
            &company_input("Hooked Co", "Active Customer"),
            Some(&f.admin_id),
        )
        .unwrap();
        company_service::archive(&conn, &company.id, Some(&f.admin_id)).unwrap();
        company.id
    };
    assert_eq!(
        pending_event_count(&f),
        3,
        "create, update and archive queue three events"
    );

    lanesra_server::job_scheduler::spawn(
        f.db_path.clone(),
        f.key_file_path.clone(),
        Duration::from_millis(150),
    );

    wait_until("three webhook deliveries", || {
        captured.lock().unwrap().len() >= 3
    })
    .await;
    let calls = captured.lock().unwrap().clone();
    assert_eq!(
        calls.len(),
        3,
        "exactly one delivery per record event: {calls:?}"
    );

    let secret = {
        let conn = open_workspace_db(&f.db_path).unwrap();
        read_webhook_secret(&conn, &f.key, &webhook_id)
    };
    let mut types: Vec<String> = Vec::new();
    for call in &calls {
        assert!(call.request_line.starts_with("POST "), "{call:?}");
        assert_eq!(
            call.signature,
            expected_signature(&secret, &call.body),
            "the receiver must be able to verify the signature"
        );
        assert!(call.signature.starts_with("sha256="));
        let payload: serde_json::Value = serde_json::from_str(&call.body).unwrap();
        for field in [
            "event_id",
            "event_type",
            "occurred_at",
            "workspace_id",
            "object_key",
            "record_id",
            "correlation_id",
            "data",
        ] {
            assert!(
                payload.get(field).is_some(),
                "payload is missing {field}: {payload}"
            );
        }
        assert_eq!(payload["object_key"], "Company");
        assert_eq!(payload["record_id"], company_id.as_str());
        types.push(payload["event_type"].as_str().unwrap().to_string());
    }
    types.sort();
    assert_eq!(
        types,
        ["record.archived", "record.created", "record.updated"]
    );

    // Delivery history records each one, and the queue is empty afterwards.
    let conn = open_workspace_db(&f.db_path).unwrap();
    let deliveries = webhook_service::list_deliveries(&conn, &f.workspace_id, &webhook_id).unwrap();
    assert_eq!(deliveries.len(), 3);
    assert!(deliveries.iter().all(|d| d.status == "succeeded"));
    wait_until("the queue to empty", || pending_event_count(&f) == 0).await;
    drop(f.dir);
}

// Covers: REQ-INT-001 (WP-0182 AC-3)
#[tokio::test]
async fn a_paused_or_other_object_webhook_receives_nothing_and_the_queue_is_cleared() {
    let f = webhook_fixture();
    let (live_port, live) = spawn_capture_server();
    let (paused_port, paused) = spawn_capture_server();
    let (other_port, other_scope) = spawn_capture_server();

    // One live webhook so the event is queued at all; one that is paused
    // after the event is queued; one scoped to a different object.
    add_webhook(&f, live_port, "Live", &["record.created"], None);
    let paused_id = add_webhook(&f, paused_port, "Paused later", &["record.created"], None);
    add_webhook(
        &f,
        other_port,
        "Contacts only",
        &["record.created"],
        Some("Contact"),
    );

    {
        let conn = open_workspace_db(&f.db_path).unwrap();
        company_service::create(
            &conn,
            &f.workspace_id,
            &company_input("Quiet Co", "Prospect"),
            Some(&f.admin_id),
        )
        .unwrap();
        webhook_service::pause(&conn, &f.workspace_id, &paused_id, Some(&f.admin_id)).unwrap();
    }
    assert_eq!(pending_event_count(&f), 1);

    lanesra_server::job_scheduler::spawn(
        f.db_path.clone(),
        f.key_file_path.clone(),
        Duration::from_millis(150),
    );

    wait_until("the live webhook delivery", || {
        live.lock().unwrap().len() == 1
    })
    .await;
    wait_until("the queue to empty", || pending_event_count(&f) == 0).await;
    // Give any wrongly delivered request time to land before asserting none did.
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(live.lock().unwrap().len(), 1);
    assert!(
        paused.lock().unwrap().is_empty(),
        "a paused webhook must receive nothing"
    );
    assert!(
        other_scope.lock().unwrap().is_empty(),
        "a webhook scoped to another object must receive nothing"
    );
    drop(f.dir);
}
