//! Integration Hub: proves the `/api/admin/...` async action routes
//! end-to-end - a real spawned server, a real HTTP client with a real
//! session cookie (not a Bearer API key - these are admin-UI actions),
//! and a real local HTTP listener standing in for "the external
//! system" so `test_connection` genuinely exercises
//! `connection_service::test_connection`'s full async path through the
//! route.
//!
//! Uses a real on-disk SQLite file (not the in-memory DB other server
//! tests use), since these handlers deliberately open their own
//! connection to the same file rather than sharing `ServerState.conn` -
//! see `admin_actions`'s own doc comment.

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener as StdTcpListener};

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::integration::ConnectionInput;
use lanesra_core::services::connection_service;
use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};

async fn spawn_file_backed_server() -> (SocketAddr, SharedState, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("lanesra.sqlite3");
    let conn = open_workspace_db(&db_path).unwrap();
    let state = ServerState::new(conn, db_path, SecurityConfig::default());
    let frontend_dir = dir.path().join("frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state.clone(), frontend_dir);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, state, dir)
}

fn client_with_cookies() -> reqwest::Client {
    reqwest::Client::builder().cookie_store(true).build().unwrap()
}

async fn invoke(client: &reqwest::Client, addr: SocketAddr, command: &str, args: Value) -> Value {
    client.post(format!("http://{addr}/api/invoke/{command}")).json(&args).send().await.unwrap().json().await.unwrap()
}

/// A minimal raw-socket HTTP/1.1 server that always answers 200 - real
/// enough to prove `test_connection` reached it and got a real response.
fn spawn_echo_server() -> u16 {
    let listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
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
            let body = "{}";
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let mut stream = stream;
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

#[tokio::test]
async fn test_connection_route_requires_a_session_and_reaches_the_real_endpoint() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();

    // No session yet - rejected before ever touching the connection.
    let unauthenticated = client.post(format!("http://{addr}/api/admin/connections/does-not-matter/test")).send().await.unwrap();
    assert_eq!(unauthenticated.status(), reqwest::StatusCode::UNAUTHORIZED);

    let setup = invoke(
        &client, addr, "first_run_setup",
        json!({
            "setup": {
                "business_name": "Admin Actions Co", "legal_name": null, "currency_code": "USD", "locale": "en-US",
                "timezone": "UTC", "default_tax_rate_bp": 0, "admin_username": "admin",
                "admin_display_name": "Admin", "admin_password": "supersecretpw", "load_sample_data": false
            }
        }),
    )
    .await;
    assert_eq!(setup["ok"], true);
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    let port = spawn_echo_server();
    let connection_id = {
        let conn = state.conn.lock().unwrap();
        connection_service::create(
            &conn, &workspace_id, &[7u8; 32],
            &ConnectionInput { name: "Local test API".into(), connection_type: "rest".into(), base_url: Some(format!("http://127.0.0.1:{port}")), auth_mode: "none".into(), secret_value: None, config_json: "{}".into(), owner_user_id: None },
            Some(&admin_id),
        )
        .unwrap()
        .id
    };

    // The `first_run_setup` call above already left a session cookie on
    // `client` (mirroring how the admin UI stays logged in after setup).
    let response = client.post(format!("http://{addr}/api/admin/connections/{connection_id}/test")).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["ok"], true, "{body:?}");
    assert_eq!(body["data"]["ok"], true, "the real echo server's 200 should read back as a successful test: {body:?}");
    assert_eq!(body["data"]["status_code"], 200);
}

#[tokio::test]
async fn run_integration_job_route_requires_a_session() {
    let (addr, _state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let response = client.post(format!("http://{addr}/api/admin/jobs/does-not-matter/run")).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}
