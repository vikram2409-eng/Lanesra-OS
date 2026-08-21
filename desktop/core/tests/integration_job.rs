//! Integration Hub (spec §15): proves `integration_job_service` - a
//! real pull-sync from an External Object into a Lanesra object
//! (Company), upserting via the exact same generic dispatcher the CSV
//! wizard/REST API use, checkpointing a cursor between runs, and
//! writing real run-history rows - against a local HTTP listener spun
//! up inside the test itself (standing in for "the external system"),
//! not a live third party, but the same request/response path a live
//! REST API would hit. Push-direction jobs are a stated, deliberate gap
//! - see `integration_job_service`'s own doc comment - so not tested
//! here.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::integration::{ConnectionInput, ExternalObjectInput, IntegrationJobInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, connection_service, external_object_service, integration_job_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Jobs Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [13u8; 32]
}

/// A raw-socket HTTP/1.1 server that serves `bodies[call_index]` (the
/// last one repeats if called more times than scripted) as a 200 JSON
/// response, and captures each request's full request line (so a test
/// can assert the second call's `since=...` checkpoint query param was
/// actually sent).
fn spawn_json_script_server(bodies: Vec<&'static str>) -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = captured.clone();
    let counter = Arc::new(AtomicUsize::new(0));
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
            captured_clone.lock().unwrap().push(request_line.trim().to_string());
            let idx = counter.fetch_add(1, Ordering::SeqCst);
            let body = bodies.get(idx).or_else(|| bodies.last()).copied().unwrap_or("[]");
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let mut stream = stream;
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, captured)
}

fn setup_job(conn: &rusqlite::Connection, workspace_id: &str, admin_id: &str, port: u16, cursor_field: Option<&str>) -> String {
    let connection = connection_service::create(
        conn, workspace_id, &master_key(),
        &ConnectionInput { name: "External CRM".into(), connection_type: "rest".into(), base_url: Some(format!("http://127.0.0.1:{port}")), auth_mode: "none".into(), secret_value: None, config_json: "{}".into(), owner_user_id: None },
        Some(admin_id),
    ).unwrap();
    let external_object = external_object_service::create(
        conn, workspace_id,
        &ExternalObjectInput { object_key: "crm_companies".into(), display_name: "CRM Companies".into(), connection_id: connection.id, resource_path: "/companies".into(), field_map: vec![], cache_ttl_seconds: None },
        Some(admin_id),
    ).unwrap();
    let job = integration_job_service::create(
        conn, workspace_id,
        &IntegrationJobInput { name: "Sync CRM Companies".into(), external_object_id: external_object.id, target_object_key: "Company".into(), match_key: "name".into(), cursor_field: cursor_field.map(String::from), interval_minutes: 30 },
        Some(admin_id),
    ).unwrap();
    job.id
}

const FIRST_BATCH: &str = r#"[
    {"name": "Acme Corp", "status": "Prospect", "updated_at": "2024-01-01T00:00:00Z"},
    {"name": "Widgets Inc", "status": "Active Customer", "updated_at": "2024-01-02T00:00:00Z"}
]"#;

#[test]
fn create_rejects_a_non_admin_and_an_unknown_external_object() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let connection = connection_service::create(
        &conn, &workspace_id, &master_key(),
        &ConnectionInput { name: "X".into(), connection_type: "rest".into(), base_url: Some("http://127.0.0.1:1".into()), auth_mode: "none".into(), secret_value: None, config_json: "{}".into(), owner_user_id: None },
        Some(&admin_id),
    ).unwrap();
    let external_object = external_object_service::create(
        &conn, &workspace_id,
        &ExternalObjectInput { object_key: "x".into(), display_name: "X".into(), connection_id: connection.id, resource_path: "/x".into(), field_map: vec![], cache_ttl_seconds: None },
        Some(&admin_id),
    ).unwrap();

    let good_input = IntegrationJobInput { name: "J".into(), external_object_id: external_object.id.clone(), target_object_key: "Company".into(), match_key: "name".into(), cursor_field: None, interval_minutes: 15 };
    assert!(integration_job_service::create(&conn, &workspace_id, &good_input, None).is_err(), "non-admins must be rejected");

    let bad_input = IntegrationJobInput { name: "J".into(), external_object_id: "not-a-real-id".into(), target_object_key: "Company".into(), match_key: "name".into(), cursor_field: None, interval_minutes: 15 };
    assert!(integration_job_service::create(&conn, &workspace_id, &bad_input, Some(&admin_id)).is_err(), "an unknown External Object must be rejected");
}

#[test]
fn create_rejects_a_non_positive_interval_and_an_empty_match_key() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let connection = connection_service::create(
        &conn, &workspace_id, &master_key(),
        &ConnectionInput { name: "X".into(), connection_type: "rest".into(), base_url: Some("http://127.0.0.1:1".into()), auth_mode: "none".into(), secret_value: None, config_json: "{}".into(), owner_user_id: None },
        Some(&admin_id),
    ).unwrap();
    let external_object = external_object_service::create(
        &conn, &workspace_id,
        &ExternalObjectInput { object_key: "x".into(), display_name: "X".into(), connection_id: connection.id, resource_path: "/x".into(), field_map: vec![], cache_ttl_seconds: None },
        Some(&admin_id),
    ).unwrap();

    let zero_interval = IntegrationJobInput { name: "J".into(), external_object_id: external_object.id.clone(), target_object_key: "Company".into(), match_key: "name".into(), cursor_field: None, interval_minutes: 0 };
    assert!(integration_job_service::create(&conn, &workspace_id, &zero_interval, Some(&admin_id)).is_err());

    let empty_match_key = IntegrationJobInput { name: "J".into(), external_object_id: external_object.id, target_object_key: "Company".into(), match_key: "  ".into(), cursor_field: None, interval_minutes: 15 };
    assert!(integration_job_service::create(&conn, &workspace_id, &empty_match_key, Some(&admin_id)).is_err());
}

#[tokio::test]
async fn run_now_creates_records_advances_the_cursor_and_writes_run_history() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let (port, captured) = spawn_json_script_server(vec![FIRST_BATCH]);
    let job_id = setup_job(&conn, &workspace_id, &admin_id, port, Some("updated_at"));

    let run = integration_job_service::run_now(&conn, &workspace_id, &master_key(), &job_id, Some(&admin_id)).await.unwrap();
    assert_eq!(run.status, "success");
    assert_eq!(run.records_processed, 2);
    assert_eq!(run.records_failed, 0);
    assert_eq!(run.cursor_before, None);
    assert_eq!(run.cursor_after.as_deref(), Some("2024-01-02T00:00:00Z"), "cursor should advance to the max updated_at seen");

    let companies = company_service::list(&conn, &workspace_id).unwrap();
    assert!(companies.iter().any(|c| c.name == "Acme Corp" && c.status == "Prospect"));
    assert!(companies.iter().any(|c| c.name == "Widgets Inc" && c.status == "Active Customer"));

    let job = integration_job_service::get(&conn, &workspace_id, &job_id).unwrap();
    assert_eq!(job.cursor_value.as_deref(), Some("2024-01-02T00:00:00Z"), "the job's own cursor must persist for the next run");
    assert_eq!(job.last_run_status.as_deref(), Some("success"));
    assert!(job.last_run_at.is_some());

    let runs = integration_job_service::list_runs(&conn, &workspace_id, &job_id, 10).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].id, run.id);

    // Only one call was made and it carried no `since` (no prior cursor).
    let calls = captured.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert!(!calls[0].contains("since="), "first run has no cursor yet: {}", calls[0]);
}

#[tokio::test]
async fn a_second_run_sends_the_checkpoint_and_upserts_rather_than_duplicates() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let second_batch = r#"[
        {"name": "Acme Corp", "status": "Active Customer", "updated_at": "2024-01-03T00:00:00Z"},
        {"name": "Gizmo LLC", "status": "Prospect", "updated_at": "2024-01-04T00:00:00Z"}
    ]"#;
    let (port, captured) = spawn_json_script_server(vec![FIRST_BATCH, second_batch]);
    let job_id = setup_job(&conn, &workspace_id, &admin_id, port, Some("updated_at"));

    integration_job_service::run_now(&conn, &workspace_id, &master_key(), &job_id, Some(&admin_id)).await.unwrap();
    let second_run = integration_job_service::run_now(&conn, &workspace_id, &master_key(), &job_id, Some(&admin_id)).await.unwrap();

    assert_eq!(second_run.status, "success");
    assert_eq!(second_run.records_processed, 2);
    assert_eq!(second_run.cursor_before.as_deref(), Some("2024-01-02T00:00:00Z"));
    assert_eq!(second_run.cursor_after.as_deref(), Some("2024-01-04T00:00:00Z"));

    let companies = company_service::list(&conn, &workspace_id).unwrap();
    // Still 3 companies, not 4 - "Acme Corp" was updated in place (matched
    // by name), not duplicated.
    assert_eq!(companies.len(), 3, "upsert-by-match_key must update, not duplicate: {companies:?}");
    let acme = companies.iter().find(|c| c.name == "Acme Corp").expect("Acme Corp still present");
    assert_eq!(acme.status, "Active Customer", "the second run's status update should have applied");
    assert!(companies.iter().any(|c| c.name == "Gizmo LLC"));

    let calls = captured.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert!(calls[1].contains("since=2024-01-02T00%3A00%3A00Z") || calls[1].contains("since=2024-01-02T00:00:00Z"), "the second call should carry the checkpoint from the first run: {}", calls[1]);
}

#[tokio::test]
async fn run_now_without_a_cursor_field_re_upserts_every_record_every_run() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let (port, _captured) = spawn_json_script_server(vec![FIRST_BATCH]);
    let job_id = setup_job(&conn, &workspace_id, &admin_id, port, None);

    let run = integration_job_service::run_now(&conn, &workspace_id, &master_key(), &job_id, Some(&admin_id)).await.unwrap();
    assert_eq!(run.cursor_after, None, "no cursor_field configured means no checkpoint is tracked");
    let job = integration_job_service::get(&conn, &workspace_id, &job_id).unwrap();
    assert_eq!(job.cursor_value, None);
}

#[tokio::test]
async fn run_now_marks_a_failure_when_the_external_system_is_unreachable() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    // Port 1 is not a listening server in this sandbox - a real
    // connection failure, not a scripted one.
    let job_id = setup_job(&conn, &workspace_id, &admin_id, 1, None);

    let run = integration_job_service::run_now(&conn, &workspace_id, &master_key(), &job_id, Some(&admin_id)).await.unwrap();
    assert_eq!(run.status, "failed");
    assert!(run.error_message.is_some());

    let job = integration_job_service::get(&conn, &workspace_id, &job_id).unwrap();
    assert_eq!(job.last_run_status.as_deref(), Some("failed"));
}

#[tokio::test]
async fn run_due_only_runs_jobs_whose_interval_has_elapsed_and_run_now_ignores_due_ness() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let (port, _captured) = spawn_json_script_server(vec![FIRST_BATCH, FIRST_BATCH, FIRST_BATCH]);
    let job_id = setup_job(&conn, &workspace_id, &admin_id, port, None);

    // Never run before -> due immediately.
    let ran = integration_job_service::run_due(&conn, &workspace_id, &master_key()).await.unwrap();
    assert_eq!(ran, 1);

    // Just ran, with a 30-minute interval -> not due again yet.
    let ran_again = integration_job_service::run_due(&conn, &workspace_id, &master_key()).await.unwrap();
    assert_eq!(ran_again, 0, "a job that just ran with a 30-minute interval should not be due again immediately");

    // Manual Run Now is unaffected by due-ness - it always runs.
    let run = integration_job_service::run_now(&conn, &workspace_id, &master_key(), &job_id, Some(&admin_id)).await.unwrap();
    assert_eq!(run.status, "success");
}

#[test]
fn update_and_delete_round_trip() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let job_id = setup_job(&conn, &workspace_id, &admin_id, 1, None);

    let job = integration_job_service::get(&conn, &workspace_id, &job_id).unwrap();
    let update_input = IntegrationJobInput { name: "Renamed Job".into(), external_object_id: job.external_object_id.clone(), target_object_key: job.target_object_key.clone(), match_key: job.match_key.clone(), cursor_field: job.cursor_field.clone(), interval_minutes: 5 };
    let updated = integration_job_service::update(&conn, &workspace_id, &job_id, &update_input, "paused", Some(&admin_id)).unwrap();
    assert_eq!(updated.name, "Renamed Job");
    assert_eq!(updated.status, "paused");
    assert_eq!(updated.interval_minutes, 5);

    assert!(integration_job_service::update(&conn, &workspace_id, &job_id, &update_input, "bogus-status", Some(&admin_id)).is_err());
    assert!(integration_job_service::update(&conn, &workspace_id, &job_id, &update_input, "active", None).is_err(), "non-admins must be rejected");

    integration_job_service::delete(&conn, &workspace_id, &job_id, Some(&admin_id)).unwrap();
    assert!(integration_job_service::list_for_workspace(&conn, &workspace_id).unwrap().is_empty());
}
