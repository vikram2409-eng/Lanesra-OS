//! WP-0189 AC-4: every scope an API client can be granted is enforced by a
//! real route. For each scope in `api_client_service::VALID_SCOPES` this
//! test names a route, proves a key holding only that scope gets through
//! (any answer but 401 or 403) and a key holding every other scope is
//! refused (403). It fails as soon as a scope is added without an entry
//! here, so a scope can never again be grantable while doing nothing
//! (RAID I-31).
//!
//! Covers: REQ-INT-005 (WP-0189 AC-4), REQ-INT-006

use std::net::SocketAddr;

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::integration::ApiClientInput;
use lanesra_core::services::api_client_service::{self, VALID_SCOPES};
use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use reqwest::Method;
use serde_json::{json, Value};

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

/// The route that enforces each scope: method, path and body.
fn enforcing_route(scope: &str, agent_id: &str) -> Option<(Method, String, Option<Value>)> {
    let company = json!({"name": "Scope Co", "status": "Prospect"});
    Some(match scope {
        "objects.read" => (Method::GET, "/api/v1/objects/Company/records".into(), None),
        "objects.write" => (
            Method::POST,
            "/api/v1/objects/Company/records".into(),
            Some(company),
        ),
        "metadata.read" => (Method::GET, "/api/v1/objects".into(), None),
        "search.read" => (Method::GET, "/api/v1/search?q=scope".into(), None),
        "bulk.read" => (Method::GET, "/api/v1/objects/Company/export".into(), None),
        "bulk.write" => (
            Method::POST,
            "/api/v1/objects/Company/bulk".into(),
            Some(json!({"operations": []})),
        ),
        "webhooks.manage" => (Method::GET, "/api/v1/webhooks".into(), None),
        "events.read" => (Method::GET, "/api/v1/events/stream".into(), None),
        "admin.integration.read" => (Method::GET, "/api/v1/integration/connections".into(), None),
        "admin.integration.manage" => (
            Method::POST,
            "/api/v1/integration/jobs/no-such-job/pause".into(),
            None,
        ),
        "agents.trigger" => (
            Method::POST,
            format!("/api/v1/agents/{agent_id}/trigger"),
            Some(json!({"input": "hello"})),
        ),
        _ => return None,
    })
}

fn key(state: &SharedState, workspace_id: &str, admin_id: &str, scopes: &[&str]) -> String {
    let conn = state.conn.lock().unwrap();
    api_client_service::create(
        &conn,
        workspace_id,
        &ApiClientInput {
            name: format!("Scope test {}", scopes.len()),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
            allowed_cidr: None,
            owner_user_id: None,
        },
        Some(admin_id),
    )
    .unwrap()
    .api_key
}

/// Sends the request and returns only the status; a streaming body (the
/// event stream) is dropped as soon as the headers arrive.
async fn status(addr: SocketAddr, key: &str, route: &(Method, String, Option<Value>)) -> u16 {
    let mut request = reqwest::Client::new()
        .request(route.0.clone(), format!("http://{addr}{}", route.1))
        .bearer_auth(key);
    if let Some(body) = &route.2 {
        request = request.json(body);
    }
    request.send().await.unwrap().status().as_u16()
}

#[tokio::test]
async fn every_grantable_scope_has_an_enforcing_route_that_allows_and_denies() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("lanesra.sqlite3");
    let state = ServerState::new(
        open_workspace_db(&db_path).unwrap(),
        db_path,
        SecurityConfig::default(),
    );
    let frontend_dir = dir.path().join("frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state.clone(), frontend_dir);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let admin = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let setup = invoke(
        &admin,
        addr,
        "first_run_setup",
        json!({"setup": {
            "business_name": "Scope Co", "legal_name": null, "currency_code": "USD",
            "locale": "en-US", "timezone": "UTC", "default_tax_rate_bp": 0,
            "admin_username": "admin", "admin_display_name": "Admin",
            "admin_password": "supersecretpw", "load_sample_data": false
        }}),
    )
    .await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();
    // agents.trigger needs an agent with an active webhook trigger (WP-0109).
    let agent = invoke(
        &admin,
        addr,
        "create_ai_agent",
        json!({"input": {"name": "Scoped", "description": null, "icon": "🤖",
                          "system_prompt": "Hi.", "action_names": [],
                          "delegate_agent_ids": [], "skill_ids": []}}),
    )
    .await;
    let agent_id = agent["data"]["id"].as_str().unwrap().to_string();
    let trigger = invoke(
        &admin,
        addr,
        "create_ai_agent_trigger",
        json!({"input": {"target_type": "agent", "target_id": agent_id,
                          "trigger_type": "webhook", "interval_minutes": null}}),
    )
    .await;
    assert_eq!(trigger["ok"], true, "{trigger}");

    let unmapped: Vec<&&str> = VALID_SCOPES
        .iter()
        .filter(|s| enforcing_route(s, &agent_id).is_none())
        .collect();
    assert!(
        unmapped.is_empty(),
        "scopes with no enforcing route: {unmapped:?}; add the route and an entry here"
    );

    for scope in VALID_SCOPES {
        let route = enforcing_route(scope, &agent_id).unwrap();
        let only = key(&state, &workspace_id, &admin_id, &[scope]);
        let allowed = status(addr, &only, &route).await;
        assert!(
            allowed != 401 && allowed != 403,
            "'{scope}' alone was refused by {} {} ({allowed})",
            route.0,
            route.1
        );
        let others: Vec<&str> = VALID_SCOPES
            .iter()
            .copied()
            .filter(|s| s != scope)
            .collect();
        let without = key(&state, &workspace_id, &admin_id, &others);
        assert_eq!(
            status(addr, &without, &route).await,
            403,
            "{} {} let through a key without '{scope}'",
            route.0,
            route.1
        );
    }
}
