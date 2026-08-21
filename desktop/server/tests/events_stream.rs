//! Integration Hub (spec §11/§23): proves the `/api/v1/events/stream`
//! SSE endpoint end-to-end - a real spawned server, a real HTTP client
//! reading the response body as it arrives (not buffered to completion),
//! and a real `integration_executions` row (produced by an ordinary
//! `/api/v1` REST call) showing up as an `event: execution` line within
//! the endpoint's 1s poll interval.
//!
//! Uses a real on-disk SQLite file (not the in-memory DB the other
//! server tests use) because the SSE handler deliberately opens its
//! *own* connection to the same file rather than sharing `ServerState.conn`
//! (see `events_stream`'s own doc comment) - an in-memory database isn't
//! reachable from a second, independent connection.

use std::net::SocketAddr;
use std::time::Duration;

use futures_util::StreamExt;
use lanesra_core::db::open_workspace_db;
use lanesra_core::models::integration::ApiClientInput;
use lanesra_core::services::{api_client_service, workspace_service};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};

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

fn setup_workspace_and_key(state: &SharedState, scopes: &[&str]) -> String {
    let conn = state.conn.lock().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Events Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    let issued = api_client_service::create(
        &conn,
        &workspace.id,
        &ApiClientInput { name: "Events Test Client".into(), scopes: scopes.iter().map(|s| s.to_string()).collect(), allowed_cidr: None, owner_user_id: None },
        Some(&admin.id),
    )
    .unwrap();
    issued.api_key
}

#[tokio::test]
async fn stream_requires_auth_and_the_events_read_scope() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let _key_without_scope = setup_workspace_and_key(&state, &["objects.read"]);

    let client = reqwest::Client::new();
    let unauthenticated = client.get(format!("http://{addr}/api/v1/events/stream")).send().await.unwrap();
    assert_eq!(unauthenticated.status(), reqwest::StatusCode::UNAUTHORIZED);

    let wrong_scope = client
        .get(format!("http://{addr}/api/v1/events/stream"))
        .header("Authorization", format!("Bearer {_key_without_scope}"))
        .send()
        .await
        .unwrap();
    assert_eq!(wrong_scope.status(), reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_new_execution_arrives_as_a_real_sse_event() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let key = setup_workspace_and_key(&state, &["events.read", "metadata.read"]);

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{addr}/api/v1/events/stream"))
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.headers().get("content-type").and_then(|v| v.to_str().ok()), Some("text/event-stream"));

    let mut byte_stream = response.bytes_stream();

    // Give the stream a moment to establish and take its "start from now"
    // baseline read, then produce a real execution row via an ordinary
    // authenticated REST call - the exact same log write any /api/v1
    // caller triggers, not a test-only shortcut.
    tokio::time::sleep(Duration::from_millis(200)).await;
    let list_response = client
        .get(format!("http://{addr}/api/v1/objects"))
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .unwrap();
    assert_eq!(list_response.status(), reqwest::StatusCode::OK);

    // Read stream chunks until the "execution" event shows up, or time out.
    let mut collected = String::new();
    let found = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let Some(chunk) = byte_stream.next().await else { return false };
            let Ok(chunk) = chunk else { return false };
            collected.push_str(&String::from_utf8_lossy(&chunk));
            if collected.contains("event: execution") && collected.contains("\"execution_type\":\"api_call\"") {
                return true;
            }
        }
    })
    .await
    .unwrap_or(false);

    assert!(found, "expected an 'execution' SSE event for the api_call, got: {collected}");
}
