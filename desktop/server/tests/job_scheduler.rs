//! Integration Hub (spec §15): proves the server's real background
//! scheduler thread (`lanesra_server::job_scheduler`) actually fires an
//! Integration Job on its own, with nobody calling `run_now` - a real
//! local HTTP listener standing in for "the external system", a real
//! on-disk SQLite file (the scheduler opens its own connection to it -
//! see that module's doc comment for why), and a short tick interval so
//! the test doesn't need to wait a real 60s.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::Duration;

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::integration::{ConnectionInput, ExternalObjectInput, IntegrationJobInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, connection_service, external_object_service, integration_job_service, workspace_service};

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
            business_name: "Scheduler Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
            timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
            admin_password: "supersecretpassword".into(), load_sample_data: false,
        };
        let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();

        let port = spawn_json_server(r#"[{"name":"Scheduled Co","status":"Prospect"}]"#);
        let connection = connection_service::create(
            &conn, &workspace.id, &master_key(),
            &ConnectionInput { name: "External CRM".into(), connection_type: "rest".into(), base_url: Some(format!("http://127.0.0.1:{port}")), auth_mode: "none".into(), secret_value: None, config_json: "{}".into(), owner_user_id: None },
            Some(&admin.id),
        ).unwrap();
        let external_object = external_object_service::create(
            &conn, &workspace.id,
            &ExternalObjectInput { object_key: "crm_companies".into(), display_name: "CRM Companies".into(), connection_id: connection.id, resource_path: "/companies".into(), field_map: vec![], cache_ttl_seconds: None },
            Some(&admin.id),
        ).unwrap();
        let job = integration_job_service::create(
            &conn, &workspace.id,
            &IntegrationJobInput { name: "Sync CRM Companies".into(), external_object_id: external_object.id, target_object_key: "Company".into(), match_key: "name".into(), cursor_field: None, interval_minutes: 1 },
            Some(&admin.id),
        ).unwrap();

        // Never run before -> due immediately once the scheduler ticks.
        assert!(job.last_run_at.is_none());
        (workspace.id, admin.id, job.id)
    };
    // The setup connection above is dropped here - the scheduler opens
    // its own, exactly as it will against a real workspace database.

    lanesra_server::job_scheduler::spawn(db_path.clone(), key_file_path, Duration::from_millis(150));

    // Poll (from a third, independent connection) for the job run to
    // land - proves the scheduler thread did real work on its own,
    // without the test ever calling run_now.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let conn = open_workspace_db(&db_path).unwrap();
        let job = integration_job_service::get(&conn, &workspace_id, &job_id).unwrap();
        if job.last_run_status.is_some() {
            assert_eq!(job.last_run_status.as_deref(), Some("success"), "the scheduled run should have succeeded");
            let companies = company_service::list(&conn, &workspace_id).unwrap();
            assert!(companies.iter().any(|c| c.name == "Scheduled Co"), "the scheduler's own run should have created the record: {companies:?}");
            break;
        }
        if std::time::Instant::now() > deadline {
            panic!("the background scheduler never ran the due job within the deadline");
        }
    }
    let _ = admin_id; // kept for clarity/documentation of setup, not asserted on further
}
