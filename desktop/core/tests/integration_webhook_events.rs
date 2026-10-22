//! WP-0185: the `field.changed`, `workflow.completed` and `workflow.failed`
//! webhook events, queued on the real write paths and the workflow runner
//! and delivered by the drain to a local receiver.
//!
//! Covers: REQ-INT-003, REQ-INT-001 (WP-0185)

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::{CustomRecordInput, CustomRecordUpdate};
use lanesra_core::models::integration::{ConnectionInput, WebhookInput};
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    company_service, connection_service, custom_object_service, custom_record_service,
    webhook_service, workflow_service,
};
use serde_json::{json, Value};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Webhook Co".into(),
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
    let (workspace, admin) = workspace_service_setup(&conn, &setup);
    (conn, workspace, admin)
}

fn workspace_service_setup(
    conn: &rusqlite::Connection,
    setup: &WorkspaceSetup,
) -> (String, String) {
    let (workspace, admin) =
        lanesra_core::services::workspace_service::first_run_setup(conn, setup).unwrap();
    (workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [9u8; 32]
}

#[derive(Clone)]
struct Captured {
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
            let mut content_length = 0usize;
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) if line == "\r\n" => break,
                    Ok(_) => {
                        let lower = line.to_ascii_lowercase();
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
            let status = responses
                .get(idx)
                .or_else(|| responses.last())
                .copied()
                .unwrap_or(200);
            captured_clone.lock().unwrap().push(Captured { body });

            let reason = match status {
                200 => "OK",
                500 => "Internal Server Error",
                400 => "Bad Request",
                other => {
                    if other >= 500 {
                        "Server Error"
                    } else {
                        "Error"
                    }
                }
            };
            let response_body = "{}";
            let response = format!("HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response_body.len(), response_body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, captured)
}

/// A webhook on a local receiver for `events`, optionally scoped to one
/// object.
fn webhook(
    conn: &rusqlite::Connection,
    workspace_id: &str,
    admin_id: &str,
    events: &[&str],
    object_scope: Option<&str>,
) -> Arc<Mutex<Vec<Captured>>> {
    let (port, captured) = spawn_scripted_server(vec![200]);
    let connection = connection_service::create(
        conn,
        workspace_id,
        &master_key(),
        &ConnectionInput {
            name: format!("Receiver {port}"),
            connection_type: "webhook".into(),
            base_url: Some(format!("http://127.0.0.1:{port}")),
            auth_mode: "none".into(),
            secret_value: None,
            config_json: "{}".into(),
            owner_user_id: None,
        },
        Some(admin_id),
    )
    .unwrap();
    webhook_service::create(
        conn,
        workspace_id,
        &master_key(),
        &WebhookInput {
            name: format!("Events {port}"),
            connection_id: connection.id,
            event_types: events.iter().map(|e| e.to_string()).collect(),
            object_scope: object_scope.map(String::from),
            filter_json: None,
            payload_version: None,
            retry_policy_json: None,
        },
        Some(admin_id),
    )
    .unwrap();
    captured
}

fn pending(conn: &rusqlite::Connection, event_type: &str) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM integration_pending_events WHERE event_type = ?1",
        [event_type],
        |r| r.get(0),
    )
    .unwrap()
}

async fn deliver(conn: &rusqlite::Connection) {
    webhook_service::drain_pending_events(conn, &master_key())
        .await
        .unwrap();
}

/// The `data` of every delivery the receiver got, for one event type.
fn received(captured: &Arc<Mutex<Vec<Captured>>>, event_type: &str) -> Vec<Value> {
    captured
        .lock()
        .unwrap()
        .iter()
        .map(|c| serde_json::from_str::<Value>(&c.body).unwrap())
        .filter(|b| b["event_type"] == event_type)
        .map(|b| b["data"].clone())
        .collect()
}

fn company(name: &str, status: &str) -> CompanyInput {
    serde_json::from_value(json!({
        "name": name, "status": status, "customer_number": null, "legal_name": null,
        "industry": null, "website": null, "email": null, "phone": null, "tax_number": null,
        "billing_address": null, "shipping_address": null, "tags": null, "notes": null,
        "owner_user_id": null
    }))
    .unwrap()
}

#[tokio::test]
async fn updating_two_fields_delivers_two_field_changed_events_and_a_no_op_save_none() {
    // Covers: REQ-INT-003 (WP-0185 AC-1)
    let (conn, ws, admin) = setup_workspace();
    let captured = webhook(&conn, &ws, &admin, &["field.changed"], None);
    let created =
        company_service::create(&conn, &ws, &company("Acme", "Prospect"), Some(&admin)).unwrap();
    assert_eq!(
        pending(&conn, "field.changed"),
        0,
        "creating is not a field change"
    );

    company_service::update(
        &conn,
        &created.id,
        &company("Acme Ltd", "Active Customer"),
        Some(&admin),
    )
    .unwrap();
    assert_eq!(pending(&conn, "field.changed"), 2);
    deliver(&conn).await;
    let mut events = received(&captured, "field.changed");
    events.sort_by_key(|e| e["field"].as_str().unwrap().to_string());
    assert_eq!(events.len(), 2, "{events:?}");
    assert_eq!(events[0]["field"], "name");
    assert_eq!(events[0]["previous"], "Acme");
    assert_eq!(events[0]["current"], "Acme Ltd");
    assert_eq!(events[0]["source"], "builtin");
    assert_eq!(events[1]["field"], "status");
    assert_eq!(events[1]["previous"], "Prospect");
    assert_eq!(events[1]["current"], "Active Customer");

    company_service::update(
        &conn,
        &created.id,
        &company("Acme Ltd", "Active Customer"),
        Some(&admin),
    )
    .unwrap();
    assert_eq!(
        pending(&conn, "field.changed"),
        0,
        "an unchanged save queues nothing"
    );
}

#[tokio::test]
async fn custom_object_fields_change_too_and_object_scope_is_respected() {
    // Covers: REQ-INT-003 (WP-0185 AC-2)
    let (conn, ws, admin) = setup_workspace();
    let def = custom_object_service::create(
        &conn,
        &ws,
        &CustomObjectDefinitionInput {
            singular_label: "Vendor".into(),
            plural_label: "Vendors".into(),
            icon: "◆".into(),
            prefix: "VEN".into(),
            digits: 4,
        },
        Some(&admin),
    )
    .unwrap();
    let all = webhook(&conn, &ws, &admin, &["field.changed"], None);
    let companies_only = webhook(&conn, &ws, &admin, &["field.changed"], Some("Company"));
    let record = custom_record_service::create(
        &conn,
        &ws,
        &CustomRecordInput {
            object_key: def.key.clone(),
            primary_name: "Quokka Supply".into(),
            status: "Active".into(),
            owner_user_id: None,
            notes: None,
        },
        Some(&admin),
    )
    .unwrap();
    let update: CustomRecordUpdate = serde_json::from_value(json!({
        "primary_name": "Quokka Supply Co", "status": "Active", "owner_user_id": null, "notes": null
    }))
    .unwrap();
    custom_record_service::update(&conn, &record.id, &update, Some(&admin)).unwrap();
    deliver(&conn).await;

    let events = received(&all, "field.changed");
    assert_eq!(events.len(), 1, "{events:?}");
    assert_eq!(events[0]["previous"], "Quokka Supply");
    assert_eq!(events[0]["current"], "Quokka Supply Co");
    assert!(
        received(&companies_only, "field.changed").is_empty(),
        "scoped to Company"
    );
}

fn status_rule(name: &str, assignee: Option<&str>) -> WorkflowDefinitionInput {
    WorkflowDefinitionInput {
        app_id: None,
        entity_type: "Company".into(),
        name: name.into(),
        description: None,
        trigger_type: "status_changed".into(),
        trigger_status: Some("Active Customer".into()),
        trigger_field_key: None,
        trigger_field_source: "builtin".into(),
        trigger_offset_days: 0,
        match_type: "all".into(),
        priority: 0,
        conditions: vec![],
        actions: vec![WorkflowActionInput {
            action_type: "create_task".into(),
            params_json: json!({"title": "Onboard", "description": null, "due_in_days": 1,
                                 "assignee_user_id": assignee})
            .to_string(),
        }],
    }
}

#[tokio::test]
async fn a_workflow_run_delivers_completed_or_failed() {
    // Covers: REQ-INT-003 (WP-0185 AC-3)
    let (conn, ws, admin) = setup_workspace();
    let captured = webhook(
        &conn,
        &ws,
        &admin,
        &["workflow.completed", "workflow.failed"],
        None,
    );
    let rule =
        workflow_service::create_rule(&conn, &ws, &status_rule("Onboard", None), Some(&admin))
            .unwrap();
    let created =
        company_service::create(&conn, &ws, &company("Acme", "Prospect"), Some(&admin)).unwrap();
    company_service::update(
        &conn,
        &created.id,
        &company("Acme", "Active Customer"),
        Some(&admin),
    )
    .unwrap();
    deliver(&conn).await;
    let completed = received(&captured, "workflow.completed");
    assert_eq!(completed.len(), 1, "{completed:?}");
    assert_eq!(completed[0]["workflow_id"], rule.id.as_str());
    assert_eq!(completed[0]["workflow_name"], "Onboard");
    assert_eq!(completed[0]["trigger_record"]["id"], created.id.as_str());
    assert!(received(&captured, "workflow.failed").is_empty());

    // A run whose action fails: the task's assignee does not exist.
    let broken = workflow_service::create_rule(
        &conn,
        &ws,
        &status_rule("Broken", Some("no-such-user")),
        Some(&admin),
    );
    let Ok(_) = broken else {
        panic!("rule creation refused the bad assignee up front: {broken:?}");
    };
    let second =
        company_service::create(&conn, &ws, &company("Beta", "Prospect"), Some(&admin)).unwrap();
    company_service::update(
        &conn,
        &second.id,
        &company("Beta", "Active Customer"),
        Some(&admin),
    )
    .unwrap();
    deliver(&conn).await;
    let failed = received(&captured, "workflow.failed");
    assert_eq!(failed.len(), 1, "{failed:?}");
    assert_eq!(failed[0]["workflow_name"], "Broken");
    let error = failed[0]["error"].as_str().unwrap();
    assert!(!error.is_empty() && error.len() <= 500, "{error}");
    let key = master_key()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert!(!failed[0].to_string().contains(&key));
}

#[tokio::test]
async fn with_no_subscriber_nothing_is_queued() {
    // Covers: REQ-INT-003 (WP-0185 AC-4)
    let (conn, ws, admin) = setup_workspace();
    // Subscribed to record events only.
    webhook(&conn, &ws, &admin, &["record.created"], None);
    workflow_service::create_rule(&conn, &ws, &status_rule("Onboard", None), Some(&admin)).unwrap();
    let created =
        company_service::create(&conn, &ws, &company("Acme", "Prospect"), Some(&admin)).unwrap();
    let updated = company_service::update(
        &conn,
        &created.id,
        &company("Acme Ltd", "Active Customer"),
        Some(&admin),
    )
    .unwrap();
    assert_eq!(updated.name, "Acme Ltd");
    for event in ["field.changed", "workflow.completed", "workflow.failed"] {
        assert_eq!(pending(&conn, event), 0, "{event}");
    }
}

#[tokio::test]
async fn a_queueing_failure_never_fails_the_write_or_the_workflow() {
    // Covers: REQ-INT-003 (WP-0185 AC-5)
    let (conn, ws, admin) = setup_workspace();
    webhook(
        &conn,
        &ws,
        &admin,
        &["field.changed", "workflow.completed"],
        None,
    );
    workflow_service::create_rule(&conn, &ws, &status_rule("Onboard", None), Some(&admin)).unwrap();
    let created =
        company_service::create(&conn, &ws, &company("Acme", "Prospect"), Some(&admin)).unwrap();
    // Break the queue: every insert into it now fails.
    conn.execute_batch(
        "CREATE TRIGGER break_queue BEFORE INSERT ON integration_pending_events
         BEGIN SELECT RAISE(ABORT, 'queue unavailable'); END;",
    )
    .unwrap();
    let updated = company_service::update(
        &conn,
        &created.id,
        &company("Acme Ltd", "Active Customer"),
        Some(&admin),
    )
    .unwrap();
    assert_eq!(updated.status, "Active Customer");
    let stored = company_service::get(&conn, &created.id).unwrap();
    assert_eq!(stored.name, "Acme Ltd", "the write committed");
    let runs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM workflow_runs WHERE outcome = 'success'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(-1);
    assert_eq!(runs, 1, "the workflow ran and recorded success");
}
