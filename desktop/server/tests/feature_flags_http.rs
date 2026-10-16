//! Workspace feature flags over the Team Workspace HTTP API (WP-0010).

use std::net::SocketAddr;

use lanesra_core::db::open_in_memory_db;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};

async fn spawn_server() -> SocketAddr {
    let conn = open_in_memory_db().unwrap();
    let state = ServerState::new(
        conn,
        std::env::temp_dir().join("lanesra-flags-test-unused.sqlite3"),
        SecurityConfig::default(),
    );
    let frontend_dir = std::env::temp_dir().join("lanesra-server-test-frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state, frontend_dir);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

async fn invoke(client: &reqwest::Client, addr: SocketAddr, command: &str, args: Value) -> Value {
    client
        .post(format!("http://{addr}/api/invoke/{command}"))
        .json(&args)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn signed_in_admin(addr: SocketAddr) -> reqwest::Client {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let setup = invoke(
        &client,
        addr,
        "first_run_setup",
        json!({"setup": {
            "business_name": "Flag Co", "legal_name": null, "currency_code": "USD",
            "locale": "en-US", "timezone": "UTC", "default_tax_rate_bp": 0,
            "admin_username": "admin", "admin_display_name": "Admin",
            "admin_password": "supersecretpw", "load_sample_data": false
        }}),
    )
    .await;
    assert_eq!(setup["ok"], true, "{setup}");
    client
}

#[tokio::test]
async fn the_flag_list_is_available_to_a_signed_in_user_and_hides_building_flags() {
    // Covers: REQ-PRE-006 (WP-0010 AC-1, AC-2)
    let addr = spawn_server().await;
    let client = signed_in_admin(addr).await;
    let body = invoke(&client, addr, "list_feature_flags", json!({})).await;
    assert_eq!(body["ok"], true, "{body}");
    // Test binaries run without LANESRA_DEVELOPER_MODE, like a release.
    assert_eq!(body["data"]["developer_mode"], false);
    let flags = body["data"]["flags"].as_array().unwrap();
    assert!(flags.iter().all(|f| f["key"] != "playground.shell"));
}

#[tokio::test]
async fn a_building_flag_cannot_be_turned_on_outside_developer_mode() {
    // Covers: REQ-PRE-006 (WP-0010 AC-2)
    let addr = spawn_server().await;
    let client = signed_in_admin(addr).await;
    let body = invoke(
        &client,
        addr,
        "set_feature_flag_override",
        json!({"input": {"key": "playground.shell", "enabled": true}}),
    )
    .await;
    assert_eq!(body["ok"], false, "{body}");
    assert_eq!(body["error"]["kind"], "validation");
}

#[tokio::test]
async fn the_flag_list_needs_a_session() {
    // Covers: REQ-PRE-006 (WP-0010)
    let addr = spawn_server().await;
    let _admin = signed_in_admin(addr).await;
    let anon = reqwest::Client::new();
    let body = invoke(&anon, addr, "list_feature_flags", json!({})).await;
    assert_eq!(body["ok"], false);
}
