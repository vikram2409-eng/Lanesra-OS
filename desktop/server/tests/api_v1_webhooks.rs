//! WP-0188: webhook management on the REST API behind `webhooks.manage`,
//! over real HTTP, with a local receiver that checks each delivery's
//! signature against the secret the API handed out.
//!
//! Covers: REQ-INT-006, REQ-INT-005 (WP-0188)

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};

use hmac::{Hmac, Mac};
use lanesra_core::db::open_workspace_db;
use lanesra_core::models::integration::ApiClientInput;
use lanesra_core::services::api_client_service;
use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};
use sha2::Sha256;

struct Ws {
    addr: SocketAddr,
    state: SharedState,
    admin_session: reqwest::Client,
    workspace_id: String,
    admin_id: String,
    _dir: tempfile::TempDir,
}

async fn workspace() -> Ws {
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
    let admin_session = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let setup = invoke(
        &admin_session,
        addr,
        "first_run_setup",
        json!({"setup": {
            "business_name": "Hook Co", "legal_name": null, "currency_code": "USD",
            "locale": "en-US", "timezone": "UTC", "default_tax_rate_bp": 0,
            "admin_username": "admin", "admin_display_name": "Admin",
            "admin_password": "supersecretpw", "load_sample_data": false
        }}),
    )
    .await;
    assert_eq!(setup["ok"], true, "{setup}");
    Ws {
        addr,
        state,
        admin_session,
        workspace_id: setup["data"][0]["id"].as_str().unwrap().to_string(),
        admin_id: setup["data"][1]["id"].as_str().unwrap().to_string(),
        _dir: dir,
    }
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

fn api_key(ws: &Ws, scopes: &[&str]) -> (String, String) {
    let conn = ws.state.conn.lock().unwrap();
    let issued = api_client_service::create(
        &conn,
        &ws.workspace_id,
        &ApiClientInput {
            name: "Provisioner".into(),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
            allowed_cidr: None,
            owner_user_id: None,
        },
        Some(&ws.admin_id),
    )
    .unwrap();
    (issued.api_key, issued.client.id)
}

/// A connection made by the admin in the UI; the API cannot create one.
async fn connection(ws: &Ws, connection_type: &str, base_url: &str) -> String {
    let created = invoke(
        &ws.admin_session,
        ws.addr,
        "create_connection",
        json!({"input": {
            "name": format!("{connection_type} receiver"), "connection_type": connection_type,
            "base_url": base_url, "auth_mode": "none",
            "secret_value": null, "config_json": "{}", "owner_user_id": null
        }}),
    )
    .await;
    assert_eq!(created["ok"], true, "{created}");
    created["data"]["id"].as_str().unwrap().to_string()
}

/// Receives deliveries and records each `(signature, body)`.
fn receiver() -> (u16, Arc<Mutex<Vec<(String, String)>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen_by_server = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let (mut length, mut signature) = (0usize, String::new());
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line.trim().is_empty() {
                    break;
                }
                let lower = line.to_ascii_lowercase();
                if let Some(v) = lower.strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap_or(0);
                }
                if lower.starts_with("x-lanesra-signature:") {
                    signature = line.split_once(':').unwrap().1.trim().to_string();
                }
            }
            let mut body = vec![0u8; length];
            let _ = reader.read_exact(&mut body);
            seen_by_server
                .lock()
                .unwrap()
                .push((signature, String::from_utf8_lossy(&body).to_string()));
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        }
    });
    (port, seen)
}

fn signs(secret: &str, signature: &str, body: &str) -> bool {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body.as_bytes());
    signature == format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

async fn call(
    addr: SocketAddr,
    key: &str,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
) -> (u16, Value) {
    let mut request = reqwest::Client::new()
        .request(method, format!("http://{addr}{path}"))
        .bearer_auth(key);
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.unwrap();
    let status = response.status().as_u16();
    let text = response.text().await.unwrap();
    (
        status,
        serde_json::from_str(&text).unwrap_or(json!({"raw": text})),
    )
}

fn webhook_input(name: &str, connection_id: &str) -> Value {
    json!({"name": name, "connection_id": connection_id, "event_types": ["record.created"],
           "object_scope": null, "filter_json": null, "payload_version": null,
           "retry_policy_json": null})
}

use reqwest::Method;

#[tokio::test]
async fn a_client_with_webhooks_manage_runs_the_whole_lifecycle_and_sees_the_secret_twice() {
    // Covers: REQ-INT-006 (WP-0188 AC-1, AC-5), REQ-INT-005
    let ws = workspace().await;
    let (port, seen) = receiver();
    let connection_id = connection(&ws, "webhook", &format!("http://127.0.0.1:{port}")).await;
    let (key, _) = api_key(&ws, &["webhooks.manage"]);
    let addr = ws.addr;

    let (status, created) = call(
        addr,
        &key,
        Method::POST,
        "/api/v1/webhooks",
        Some(webhook_input("Orders feed", &connection_id)),
    )
    .await;
    assert_eq!(status, 200, "{created}");
    let id = created["data"]["id"].as_str().unwrap().to_string();
    let first_secret = created["data"]["signing_secret"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(first_secret.len() >= 32);

    // Reads never carry the secret.
    let (_, listed) = call(addr, &key, Method::GET, "/api/v1/webhooks", None).await;
    let (_, read) = call(
        addr,
        &key,
        Method::GET,
        &format!("/api/v1/webhooks/{id}"),
        None,
    )
    .await;
    for body in [&listed, &read] {
        assert!(!body.to_string().contains(&first_secret), "{body}");
    }
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);
    assert_eq!(read["data"]["has_secret"], true);

    let mut renamed = webhook_input("Orders feed v2", &connection_id);
    renamed["event_types"] = json!(["record.created", "record.updated"]);
    let (status, updated) = call(
        addr,
        &key,
        Method::PATCH,
        &format!("/api/v1/webhooks/{id}"),
        Some(renamed),
    )
    .await;
    assert_eq!(status, 200, "{updated}");
    assert_eq!(updated["data"]["name"], "Orders feed v2");
    assert_eq!(updated["data"]["event_types"].as_array().unwrap().len(), 2);

    let (_, paused) = call(
        addr,
        &key,
        Method::POST,
        &format!("/api/v1/webhooks/{id}/pause"),
        None,
    )
    .await;
    assert_eq!(paused["data"]["status"], "paused");
    let (_, active) = call(
        addr,
        &key,
        Method::POST,
        &format!("/api/v1/webhooks/{id}/reactivate"),
        None,
    )
    .await;
    assert_eq!(active["data"]["status"], "active");

    // A test delivery is signed with the secret create returned.
    let (status, tested) = call(
        addr,
        &key,
        Method::POST,
        &format!("/api/v1/webhooks/{id}/test"),
        None,
    )
    .await;
    assert_eq!(status, 200, "{tested}");
    let (signature, body) = seen.lock().unwrap().last().cloned().unwrap();
    assert!(signs(&first_secret, &signature, &body));

    // Rotation returns a new secret once; the next delivery verifies with
    // it and not with the old one.
    let (status, rotated) = call(
        addr,
        &key,
        Method::POST,
        &format!("/api/v1/webhooks/{id}/rotate-secret"),
        None,
    )
    .await;
    assert_eq!(status, 200, "{rotated}");
    let second_secret = rotated["data"]["signing_secret"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ne!(first_secret, second_secret);
    call(
        addr,
        &key,
        Method::POST,
        &format!("/api/v1/webhooks/{id}/test"),
        None,
    )
    .await;
    let (signature, body) = seen.lock().unwrap().last().cloned().unwrap();
    assert!(signs(&second_secret, &signature, &body));
    assert!(!signs(&first_secret, &signature, &body));

    // AC-5: the delivery history pages and never carries a secret.
    let (_, page_one) = call(
        addr,
        &key,
        Method::GET,
        &format!("/api/v1/webhooks/{id}/deliveries?page_size=1"),
        None,
    )
    .await;
    assert_eq!(page_one["data"]["total"], 2, "{page_one}");
    assert_eq!(page_one["data"]["deliveries"].as_array().unwrap().len(), 1);
    let (_, page_two) = call(
        addr,
        &key,
        Method::GET,
        &format!("/api/v1/webhooks/{id}/deliveries?page_size=1&page=2"),
        None,
    )
    .await;
    assert_ne!(
        page_one["data"]["deliveries"][0]["id"],
        page_two["data"]["deliveries"][0]["id"]
    );
    for page in [&page_one, &page_two] {
        let text = page.to_string();
        assert!(!text.contains(&first_secret) && !text.contains(&second_secret));
    }

    let (status, _) = call(
        addr,
        &key,
        Method::DELETE,
        &format!("/api/v1/webhooks/{id}"),
        None,
    )
    .await;
    assert_eq!(status, 200);
    let (status, _) = call(
        addr,
        &key,
        Method::GET,
        &format!("/api/v1/webhooks/{id}"),
        None,
    )
    .await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn without_webhooks_manage_every_webhook_endpoint_is_forbidden() {
    // Covers: REQ-INT-006 (WP-0188 AC-2), REQ-INT-005
    let ws = workspace().await;
    let connection_id = connection(&ws, "webhook", "http://127.0.0.1:9").await;
    let (key, _) = api_key(
        &ws,
        &["objects.write", "admin.integration.manage", "metadata.read"],
    );
    let addr = ws.addr;
    let id = "any-id";
    let attempts = [
        (Method::GET, "/api/v1/webhooks".to_string(), None),
        (
            Method::POST,
            "/api/v1/webhooks".to_string(),
            Some(webhook_input("X", &connection_id)),
        ),
        (Method::GET, format!("/api/v1/webhooks/{id}"), None),
        (
            Method::PATCH,
            format!("/api/v1/webhooks/{id}"),
            Some(webhook_input("X", &connection_id)),
        ),
        (Method::DELETE, format!("/api/v1/webhooks/{id}"), None),
        (Method::POST, format!("/api/v1/webhooks/{id}/pause"), None),
        (
            Method::POST,
            format!("/api/v1/webhooks/{id}/reactivate"),
            None,
        ),
        (
            Method::POST,
            format!("/api/v1/webhooks/{id}/rotate-secret"),
            None,
        ),
        (Method::POST, format!("/api/v1/webhooks/{id}/test"), None),
        (
            Method::GET,
            format!("/api/v1/webhooks/{id}/deliveries"),
            None,
        ),
    ];
    for (method, path, body) in attempts {
        let (status, response) = call(addr, &key, method.clone(), &path, body).await;
        assert_eq!(status, 403, "{method} {path}: {response}");
    }
}

#[tokio::test]
async fn a_webhook_needs_a_webhook_connection_whose_url_the_guard_allows() {
    // Covers: REQ-INT-006 (WP-0188 AC-3)
    let ws = workspace().await;
    let (key, _) = api_key(&ws, &["webhooks.manage"]);
    let rest = connection(&ws, "rest", "http://127.0.0.1:9").await;
    let (status, refused) = call(
        ws.addr,
        &key,
        Method::POST,
        "/api/v1/webhooks",
        Some(webhook_input("A", &rest)),
    )
    .await;
    assert_eq!(status, 400, "{refused}");

    // A webhook connection whose URL is a cloud metadata address. Saved
    // straight to the database, as an older or imported connection would be.
    let metadata = connection(&ws, "webhook", "http://127.0.0.1:9").await;
    {
        let conn = ws.state.conn.lock().unwrap();
        conn.execute(
            "UPDATE integration_connections SET base_url = 'http://169.254.169.254/latest' WHERE id = ?1",
            [&metadata],
        )
        .unwrap();
    }
    let (status, refused) = call(
        ws.addr,
        &key,
        Method::POST,
        "/api/v1/webhooks",
        Some(webhook_input("B", &metadata)),
    )
    .await;
    assert_eq!(status, 403, "{refused}");
    assert!(refused.to_string().contains("metadata"), "{refused}");
    let (_, listed) = call(ws.addr, &key, Method::GET, "/api/v1/webhooks", None).await;
    assert!(listed["data"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn creation_and_rotation_are_audited_with_the_client_and_without_the_secret() {
    // Covers: REQ-INT-006 (WP-0188 AC-4)
    let ws = workspace().await;
    let connection_id = connection(&ws, "webhook", "http://127.0.0.1:9").await;
    let (key, client_id) = api_key(&ws, &["webhooks.manage"]);
    let (_, created) = call(
        ws.addr,
        &key,
        Method::POST,
        "/api/v1/webhooks",
        Some(webhook_input("Audited", &connection_id)),
    )
    .await;
    let id = created["data"]["id"].as_str().unwrap().to_string();
    let first = created["data"]["signing_secret"]
        .as_str()
        .unwrap()
        .to_string();
    let (_, rotated) = call(
        ws.addr,
        &key,
        Method::POST,
        &format!("/api/v1/webhooks/{id}/rotate-secret"),
        None,
    )
    .await;
    let second = rotated["data"]["signing_secret"]
        .as_str()
        .unwrap()
        .to_string();

    let conn = ws.state.conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT event_type, user_id, summary, details_json FROM audit_events
              WHERE entity_type = 'Webhook' AND entity_id = ?1 ORDER BY rowid",
        )
        .unwrap();
    let rows: Vec<(String, Option<String>, String, Option<String>)> = stmt
        .query_map([&id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let kinds: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(kinds, vec!["webhook.created", "webhook.secret_rotated"]);
    for (_, user_id, summary, details) in &rows {
        assert!(user_id.is_none(), "an API client is not a user");
        let details: Value = serde_json::from_str(details.as_deref().unwrap()).unwrap();
        assert_eq!(details["api_client_id"], client_id.as_str());
        let all = format!("{summary}{details}");
        assert!(!all.contains(&first) && !all.contains(&second), "{all}");
    }
}
