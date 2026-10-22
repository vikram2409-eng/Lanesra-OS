//! WP-0183: over real HTTP, as an admin session, creating a webhook shows
//! its signing secret exactly once, no read ever returns it, rotating
//! returns a new one once, and only an administrator can rotate.

use std::net::SocketAddr;

use lanesra_core::db::open_in_memory_db;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};

async fn spawn_server() -> SocketAddr {
    let conn = open_in_memory_db().unwrap();
    let state = ServerState::new(
        conn,
        std::env::temp_dir().join("lanesra-webhook-secret-test-unused.sqlite3"),
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

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn invoke(c: &reqwest::Client, addr: SocketAddr, command: &str, args: Value) -> Value {
    c.post(format!("http://{addr}/api/invoke/{command}"))
        .json(&args)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

// Covers: REQ-INT-002 (WP-0183 AC-1, AC-2, AC-3)
#[tokio::test]
async fn the_secret_is_shown_once_rotated_over_http_and_only_an_admin_can_rotate() {
    let addr = spawn_server().await;
    let admin = client();
    let setup = invoke(
        &admin,
        addr,
        "first_run_setup",
        json!({"setup": {
            "business_name": "Secret Co", "legal_name": null, "currency_code": "USD",
            "locale": "en-US", "timezone": "UTC", "default_tax_rate_bp": 0,
            "admin_username": "admin", "admin_display_name": "Admin",
            "admin_password": "supersecretpw", "load_sample_data": false
        }}),
    )
    .await;
    assert_eq!(setup["ok"], true, "{setup}");

    let connection = invoke(
        &admin,
        addr,
        "create_connection",
        json!({"input": {
            "name": "Receiver", "connection_type": "webhook",
            "base_url": "http://127.0.0.1:9", "auth_mode": "none",
            "secret_value": null, "config_json": "{}", "owner_user_id": null
        }}),
    )
    .await;
    assert_eq!(connection["ok"], true, "{connection}");
    let connection_id = connection["data"]["id"].as_str().unwrap().to_string();

    // Create: the secret is in the response, once.
    let created = invoke(
        &admin,
        addr,
        "create_webhook",
        json!({"input": {
            "name": "Signed", "connection_id": connection_id,
            "event_types": ["record.created"], "object_scope": null,
            "filter_json": null, "payload_version": null, "retry_policy_json": null
        }}),
    )
    .await;
    assert_eq!(created["ok"], true, "{created}");
    let secret = created["data"]["signing_secret"]
        .as_str()
        .unwrap()
        .to_string();
    let id = created["data"]["id"].as_str().unwrap().to_string();
    assert!(secret.len() >= 32);
    assert_eq!(created["data"]["has_secret"], true);

    // No read returns it.
    let listed = invoke(&admin, addr, "list_webhooks", json!({})).await;
    assert_eq!(listed["ok"], true);
    assert!(
        !listed.to_string().contains(&secret),
        "list must not contain the secret"
    );

    // Rotate: a different secret, once; reads still do not return it.
    let rotated = invoke(&admin, addr, "rotate_webhook_secret", json!({"id": id})).await;
    assert_eq!(rotated["ok"], true, "{rotated}");
    let new_secret = rotated["data"].as_str().unwrap().to_string();
    assert_ne!(new_secret, secret);
    assert!(new_secret.len() >= 32);
    let listed = invoke(&admin, addr, "list_webhooks", json!({}))
        .await
        .to_string();
    assert!(!listed.contains(&new_secret) && !listed.contains(&secret));

    // A non-admin cannot rotate.
    invoke(
        &admin,
        addr,
        "create_user",
        json!({"input": {"username": "sam", "display_name": "Sam", "password": "anothersecretpw", "roles": ["Sales"]}}),
    )
    .await;
    let sam = client();
    invoke(
        &sam,
        addr,
        "login",
        json!({"credentials": {"username": "sam", "password": "anothersecretpw"}}),
    )
    .await;
    let denied = invoke(&sam, addr, "rotate_webhook_secret", json!({"id": id})).await;
    assert_eq!(denied["ok"], false, "{denied}");

    // An unknown id is an error, not a panic or a new secret.
    let unknown = invoke(&admin, addr, "rotate_webhook_secret", json!({"id": "nope"})).await;
    assert_eq!(unknown["ok"], false, "{unknown}");
}
