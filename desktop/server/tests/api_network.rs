//! WP-0184 / ADR-0018: an API client's allowed network is enforced on every
//! API surface, and the caller address comes from the connection unless the
//! peer is a configured trusted proxy.
//!
//! The test server listens on loopback, so the peer is always 127.0.0.1:
//! "outside" means a client whose allowed list excludes loopback, and the
//! forwarded path is exercised by listing loopback as a trusted proxy.

use std::net::SocketAddr;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::integration::ApiClientInput;
use lanesra_core::services::api_client_service;
use lanesra_core::services::network_trust::parse_list;
use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};

const REFUSAL: &str = "This API client is not allowed from this network address";

struct Server {
    addr: SocketAddr,
    state: SharedState,
    workspace_id: String,
    admin_id: String,
}

/// `with_peer_info = false` serves the router the plain way, so the server
/// cannot see a peer address at all.
async fn spawn(trusted_proxies: &str, with_peer_info: bool) -> Server {
    let conn = open_in_memory_db().unwrap();
    let security = SecurityConfig {
        trusted_proxies: parse_list(trusted_proxies).unwrap(),
        ..SecurityConfig::default()
    };
    let state = ServerState::new(
        conn,
        std::env::temp_dir().join("lanesra-api-network-test-unused.sqlite3"),
        security,
    );
    let frontend_dir = std::env::temp_dir().join("lanesra-server-test-frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state.clone(), frontend_dir);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        if with_peer_info {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .unwrap();
        } else {
            axum::serve(listener, app).await.unwrap();
        }
    });

    let http = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let setup: Value = http
        .post(format!("http://{addr}/api/invoke/first_run_setup"))
        .json(&json!({"setup": {
            "business_name": "Test Co", "legal_name": null, "currency_code": "USD", "locale": "en-US",
            "timezone": "UTC", "default_tax_rate_bp": 0, "admin_username": "admin",
            "admin_display_name": "Admin", "admin_password": "supersecretpw", "load_sample_data": false
        }}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(setup["ok"], true);
    Server {
        addr,
        state,
        workspace_id: setup["data"][0]["id"].as_str().unwrap().to_string(),
        admin_id: setup["data"][1]["id"].as_str().unwrap().to_string(),
    }
}

impl Server {
    fn key(&self, allowed_cidr: Option<&str>) -> String {
        let conn = self.state.conn.lock().unwrap();
        api_client_service::create(
            &conn,
            &self.workspace_id,
            &ApiClientInput {
                name: "Network test".into(),
                scopes: vec![
                    "metadata.read".into(),
                    "objects.read".into(),
                    "events.read".into(),
                ],
                allowed_cidr: allowed_cidr.map(String::from),
                owner_user_id: None,
            },
            Some(&self.admin_id),
        )
        .unwrap()
        .api_key
    }

    fn get(&self, path: &str, key: &str) -> reqwest::RequestBuilder {
        reqwest::Client::new()
            .get(format!("http://{}{path}", self.addr))
            .bearer_auth(key)
    }

    fn mcp(&self, key: &str) -> reqwest::RequestBuilder {
        reqwest::Client::new()
            .post(format!("http://{}/mcp", self.addr))
            .bearer_auth(key)
            .json(&json!({"jsonrpc": "2.0", "id": 1, "method": "ping", "params": {}}))
    }
}

async fn assert_refused(response: reqwest::Response) {
    assert_eq!(response.status(), 403);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"], REFUSAL);
}

// Covers: REQ-INT-004 (WP-0184 AC-1)
#[tokio::test]
async fn a_restricted_client_is_admitted_inside_its_range_on_records_mcp_and_the_stream() {
    let server = spawn("", true).await;
    let key = server.key(Some("10.0.0.0/8, 127.0.0.0/8"));
    for path in ["/api/v1/objects/Company/records", "/api/v1/limits"] {
        assert_eq!(
            server.get(path, &key).send().await.unwrap().status(),
            200,
            "{path}"
        );
    }
    assert_eq!(server.mcp(&key).send().await.unwrap().status(), 200);
    let stream = server
        .get("/api/v1/events/stream", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(stream.status(), 200);
}

// Covers: REQ-INT-004 (WP-0184 AC-1)
#[tokio::test]
async fn a_restricted_client_is_refused_outside_its_range_on_records_mcp_and_the_stream() {
    let server = spawn("", true).await;
    let key = server.key(Some("203.0.113.0/24, 2001:db8::/32"));
    assert_refused(
        server
            .get("/api/v1/objects/Company/records", &key)
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_refused(server.mcp(&key).send().await.unwrap()).await;
    assert_refused(
        server
            .get("/api/v1/events/stream", &key)
            .send()
            .await
            .unwrap(),
    )
    .await;
}

// Covers: REQ-INT-004 (WP-0184 AC-2)
#[tokio::test]
async fn a_client_with_no_allowed_network_works_from_anywhere_as_before() {
    let server = spawn("", true).await;
    for restriction in [None, Some(""), Some("   ")] {
        let key = server.key(restriction);
        assert_eq!(
            server
                .get("/api/v1/limits", &key)
                .send()
                .await
                .unwrap()
                .status(),
            200,
            "{restriction:?}"
        );
    }
}

// Covers: REQ-INT-004 (WP-0184 AC-3)
#[tokio::test]
async fn forged_forwarding_headers_do_not_admit_a_restricted_client() {
    // No trusted proxy configured: every forwarding header is ignored.
    let server = spawn("", true).await;
    let key = server.key(Some("198.51.100.7"));
    let response = server
        .get("/api/v1/limits", &key)
        .header("X-Forwarded-For", "198.51.100.7")
        .header("Forwarded", "for=198.51.100.7")
        .header("X-Real-IP", "198.51.100.7")
        .send()
        .await
        .unwrap();
    assert_refused(response).await;
}

// Covers: REQ-INT-004 (WP-0184 AC-4)
#[tokio::test]
async fn behind_a_trusted_proxy_the_right_most_untrusted_hop_is_the_caller() {
    let server = spawn("127.0.0.1", true).await;
    let key = server.key(Some("198.51.100.7"));

    // The proxy saw 198.51.100.7 and passed it on: admitted.
    let admitted = server
        .get("/api/v1/limits", &key)
        .header("X-Forwarded-For", "198.51.100.7")
        .send()
        .await
        .unwrap();
    assert_eq!(admitted.status(), 200);

    // The real client (6.6.6.6) prepended a spoof; the proxy appended the
    // address it actually saw. The spoof must not count.
    let spoofed = server
        .get("/api/v1/limits", &key)
        .header("X-Forwarded-For", "198.51.100.7, 6.6.6.6")
        .send()
        .await
        .unwrap();
    assert_refused(spoofed).await;

    // No header at all: the caller is the proxy itself, which is not in range.
    assert_refused(server.get("/api/v1/limits", &key).send().await.unwrap()).await;
}

// Covers: REQ-INT-004 (WP-0184 AC-7)
#[tokio::test]
async fn a_bad_key_gets_the_same_401_whatever_the_address_and_a_refusal_is_logged() {
    let server = spawn("127.0.0.1", true).await;
    let key = server.key(Some("198.51.100.7"));

    let (client_id, _secret) = key.split_once('.').unwrap();
    let wrong = format!("{client_id}.not-the-secret");
    for forwarded in ["198.51.100.7", "6.6.6.6"] {
        let response = server
            .get("/api/v1/limits", &wrong)
            .header("X-Forwarded-For", forwarded)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 401, "{forwarded}");
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["error"], "Invalid or revoked API credential");
    }

    // A valid key from the wrong address is refused and the resolved
    // address is written to the integration log.
    let refused = server
        .get("/api/v1/limits", &key)
        .header("X-Forwarded-For", "6.6.6.6")
        .send()
        .await
        .unwrap();
    assert_refused(refused).await;
    let message: String = server
        .state
        .conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT error_message FROM integration_executions WHERE error_category = 'network_blocked'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(message.contains("6.6.6.6"), "{message}");
}

// Covers: REQ-INT-004 (WP-0184 AC-1)
#[tokio::test]
async fn when_the_peer_address_is_unknown_a_restricted_client_is_refused_and_others_still_work() {
    let server = spawn("", false).await;
    let restricted = server.key(Some("127.0.0.0/8"));
    assert_refused(
        server
            .get("/api/v1/limits", &restricted)
            .send()
            .await
            .unwrap(),
    )
    .await;
    let open = server.key(None);
    assert_eq!(
        server
            .get("/api/v1/limits", &open)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
}
