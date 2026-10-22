//! WP-0187: bulk export (`bulk.read`) and bulk write (`bulk.write`) on the
//! inbound REST API, over real HTTP.
//!
//! Covers: REQ-INT-006, REQ-INT-005 (WP-0187)

use std::net::SocketAddr;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::integration::ApiClientInput;
use lanesra_core::services::api_client_service;
use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};

async fn spawn_server_with_state() -> (SocketAddr, SharedState) {
    let conn = open_in_memory_db().unwrap();
    let state = ServerState::new(
        conn,
        std::env::temp_dir().join("lanesra-api-v1-test-unused.sqlite3"),
        SecurityConfig::default(),
    );
    let frontend_dir = std::env::temp_dir().join("lanesra-server-test-frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state.clone(), frontend_dir);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, state)
}

fn client_with_cookies() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
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

async fn first_run(client: &reqwest::Client, addr: SocketAddr) -> Value {
    invoke(
        client,
        addr,
        "first_run_setup",
        json!({
            "setup": {
                "business_name": "Test Co", "legal_name": null, "currency_code": "USD", "locale": "en-US",
                "timezone": "UTC", "default_tax_rate_bp": 0, "admin_username": "admin",
                "admin_display_name": "Admin", "admin_password": "supersecretpw", "load_sample_data": false
            }
        }),
    )
    .await
}

/// Issues a real `{client_id}.{secret}` API key with the given scopes,
/// through the exact same `api_client_service::create` an admin UI would
/// call - not a hand-rolled test-only shortcut.
fn issue_api_key(
    state: &SharedState,
    workspace_id: &str,
    admin_user_id: &str,
    scopes: &[&str],
) -> String {
    let conn = state.conn.lock().unwrap();
    let issued = api_client_service::create(
        &conn,
        workspace_id,
        &ApiClientInput {
            name: "Test Integration".into(),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
            allowed_cidr: None,
            owner_user_id: None,
        },
        Some(admin_user_id),
    )
    .unwrap();
    issued.api_key
}

async fn setup_workspace_and_key(scopes: &[&str]) -> (SocketAddr, SharedState, String) {
    let (addr, state, key, _admin_id) = setup_workspace_and_key_with_admin(scopes).await;
    (addr, state, key)
}

async fn setup_workspace_and_key_with_admin(
    scopes: &[&str],
) -> (SocketAddr, SharedState, String, String) {
    let (addr, state) = spawn_server_with_state().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    assert_eq!(setup["ok"], true);
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();
    let api_key = issue_api_key(&state, &workspace_id, &admin_id, scopes);
    (addr, state, api_key, admin_id)
}

async fn bulk(
    addr: SocketAddr,
    api_key: &str,
    object: &str,
    operations: Value,
) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/objects/{object}/bulk"))
        .bearer_auth(api_key)
        .json(&json!({"operations": operations}))
        .send()
        .await
        .unwrap()
}

async fn bulk_ok(addr: SocketAddr, api_key: &str, object: &str, operations: Value) -> Value {
    let response = bulk(addr, api_key, object, operations).await;
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["ok"], true, "{body:?}");
    body["data"].clone()
}

async fn export(addr: SocketAddr, api_key: &str, object: &str, format: &str) -> reqwest::Response {
    reqwest::Client::new()
        .get(format!(
            "http://{addr}/api/v1/objects/{object}/export?format={format}"
        ))
        .bearer_auth(api_key)
        .send()
        .await
        .unwrap()
}

async fn company_count(addr: SocketAddr, api_key: &str) -> i64 {
    let body: Value = reqwest::Client::new()
        .get(format!(
            "http://{addr}/api/v1/objects/Company/records?page_size=1"
        ))
        .bearer_auth(api_key)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    body["data"]["total"].as_i64().unwrap()
}

fn create_company(name: &str) -> Value {
    json!({"op": "create", "data": {"name": name, "status": "Prospect"}})
}

const ALL: &[&str] = &["bulk.read", "bulk.write", "objects.read", "objects.write"];

#[tokio::test]
async fn export_streams_every_record_as_ndjson_and_csv_without_archived_ones() {
    // Covers: REQ-INT-006 (WP-0187 AC-1)
    let (addr, _state, api_key) = setup_workspace_and_key(ALL).await;
    for batch in 0..3 {
        let ops: Vec<Value> = (0..1000)
            .map(|i| create_company(&format!("Company {batch}-{i:04}")))
            .collect();
        let data = bulk_ok(addr, &api_key, "Company", json!(ops)).await;
        assert_eq!(data["succeeded"], 1000, "{}", data["results"][0]);
    }
    let gone = bulk_ok(
        addr,
        &api_key,
        "Company",
        json!([create_company("Archived Co")]),
    )
    .await;
    let gone_id = gone["results"][0]["id"].as_str().unwrap().to_string();
    bulk_ok(
        addr,
        &api_key,
        "Company",
        json!([{"op": "archive", "id": gone_id}]),
    )
    .await;

    let response = export(addr, &api_key, "Company", "ndjson").await;
    assert_eq!(response.status(), 200);
    assert!(response.headers()["content-type"]
        .to_str()
        .unwrap()
        .contains("ndjson"));
    let text = response.text().await.unwrap();
    let lines: Vec<Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 3000);
    assert!(lines.iter().all(|r| r["id"] != gone_id.as_str()));

    let csv = export(addr, &api_key, "Company", "csv")
        .await
        .text()
        .await
        .unwrap();
    let mut rows = csv.lines();
    let header = rows.next().unwrap();
    assert!(header.split(',').any(|c| c == "name"), "{header}");
    assert_eq!(rows.count(), 3000);
    assert!(!csv.contains("Archived Co"));

    assert_eq!(export(addr, &api_key, "Company", "xml").await.status(), 400);
    assert_eq!(
        export(addr, &api_key, "NoSuchObject", "ndjson")
            .await
            .status(),
        404
    );
}

#[tokio::test]
async fn export_needs_the_bulk_read_scope() {
    // Covers: REQ-INT-006 (WP-0187 AC-1), REQ-INT-005
    let (addr, _state, api_key) = setup_workspace_and_key(&["objects.read"]).await;
    assert_eq!(
        export(addr, &api_key, "Company", "ndjson").await.status(),
        403
    );
}

#[tokio::test]
async fn a_mixed_bulk_write_applies_each_operation_in_order() {
    // Covers: REQ-INT-006 (WP-0187 AC-2)
    let (addr, _state, api_key) = setup_workspace_and_key(ALL).await;
    let seeded = bulk_ok(
        addr,
        &api_key,
        "Company",
        json!([create_company("Keep Co"), create_company("Drop Co")]),
    )
    .await;
    let keep = seeded["results"][0]["id"].as_str().unwrap().to_string();
    let drop = seeded["results"][1]["id"].as_str().unwrap().to_string();

    let data = bulk_ok(
        addr,
        &api_key,
        "Company",
        json!([
            create_company("New Co"),
            {"op": "update", "id": keep, "data": {"name": "Kept Co", "status": "Active Customer"}},
            {"op": "archive", "id": drop},
        ]),
    )
    .await;
    assert_eq!(data["succeeded"], 3, "{data:?}");
    assert_eq!(data["failed"], 0);
    let indexes: Vec<i64> = data["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["index"].as_i64().unwrap())
        .collect();
    assert_eq!(indexes, vec![0, 1, 2]);
    assert_eq!(data["results"][1]["id"], keep.as_str());

    let kept: Value = reqwest::Client::new()
        .get(format!(
            "http://{addr}/api/v1/objects/Company/records/{keep}"
        ))
        .bearer_auth(&api_key)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(kept["data"]["name"], "Kept Co");
    assert_eq!(
        company_count(addr, &api_key).await,
        2,
        "Keep Co and New Co; Drop Co is archived"
    );
}

#[tokio::test]
async fn an_invalid_item_fails_alone_and_the_others_still_apply() {
    // Covers: REQ-INT-006 (WP-0187 AC-3)
    let (addr, _state, api_key) = setup_workspace_and_key(ALL).await;
    let data = bulk_ok(
        addr,
        &api_key,
        "Company",
        json!([
            create_company("First Co"),
            {"op": "create", "data": {"name": "No Status Co"}},
            {"op": "update", "id": "no-such-id", "data": {"name": "X", "status": "Prospect"}},
            {"op": "archive"},
            {"op": "merge", "id": "x"},
            create_company("Last Co"),
        ]),
    )
    .await;
    let oks: Vec<bool> = data["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["ok"].as_bool().unwrap())
        .collect();
    assert_eq!(
        oks,
        vec![true, false, false, false, false, true],
        "{data:?}"
    );
    assert_eq!(data["succeeded"], 2);
    assert_eq!(data["failed"], 4);
    assert!(data["results"][1]["error"].as_str().unwrap().len() > 3);
    assert_eq!(company_count(addr, &api_key).await, 2);

    // A read-only built-in fails per item with the single-record API's error.
    let read_only = bulk_ok(
        addr,
        &api_key,
        "Opportunity",
        json!([{"op": "create", "data": {"name": "Deal"}}]),
    )
    .await;
    assert_eq!(read_only["failed"], 1);
    assert!(read_only["results"][0]["error"]
        .as_str()
        .unwrap()
        .contains("read-only"));
    assert_eq!(
        bulk(addr, &api_key, "NoSuchObject", json!([]))
            .await
            .status(),
        404
    );
}

#[tokio::test]
async fn more_than_1000_operations_is_refused_and_nothing_is_applied() {
    // Covers: REQ-INT-006 (WP-0187 AC-4)
    let (addr, _state, api_key) = setup_workspace_and_key(ALL).await;
    let ops: Vec<Value> = (0..1001)
        .map(|i| create_company(&format!("Too Many {i}")))
        .collect();
    let response = bulk(addr, &api_key, "Company", json!(ops)).await;
    assert_eq!(response.status(), 413);
    assert_eq!(company_count(addr, &api_key).await, 0);
}

#[tokio::test]
async fn objects_write_and_bulk_write_are_separate_scopes() {
    // Covers: REQ-INT-006 (WP-0187 AC-5), REQ-INT-005
    let (addr, state, single_only) =
        setup_workspace_and_key(&["objects.write", "objects.read"]).await;
    assert_eq!(
        bulk(
            addr,
            &single_only,
            "Company",
            json!([create_company("A Co")])
        )
        .await
        .status(),
        403
    );
    let workspace_id = {
        let conn = state.conn.lock().unwrap();
        lanesra_core::repositories::workspace_repo::get_current(&conn)
            .unwrap()
            .unwrap()
            .id
    };
    let admin_id = {
        let conn = state.conn.lock().unwrap();
        conn.query_row("SELECT id FROM users LIMIT 1", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap()
    };
    let bulk_only = issue_api_key(&state, &workspace_id, &admin_id, &["bulk.write"]);
    let single = reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/objects/Company/records"))
        .bearer_auth(&bulk_only)
        .json(&json!({"name": "B Co", "status": "Prospect"}))
        .send()
        .await
        .unwrap();
    assert_eq!(single.status(), 403);
    let data = bulk_ok(addr, &bulk_only, "Company", json!([create_company("C Co")])).await;
    assert_eq!(data["succeeded"], 1);
}

#[tokio::test]
async fn a_business_rule_that_blocks_a_save_blocks_that_bulk_item() {
    // Covers: REQ-INT-006 (WP-0187 AC-6)
    let (addr, _state, api_key) = setup_workspace_and_key(ALL).await;
    let client = client_with_cookies();
    // The admin session from setup is on another client; sign in again.
    let signed_in = invoke(
        &client,
        addr,
        "login",
        json!({"credentials": {"username": "admin", "password": "supersecretpw"}}),
    )
    .await;
    assert_eq!(signed_in["ok"], true, "{signed_in:?}");
    let rule = invoke(
        &client,
        addr,
        "create_business_rule",
        json!({"input": {
            "entity_type": "Company", "name": "No blocked names", "description": null,
            "match_type": "all", "priority": 0, "effective_start_date": null, "effective_end_date": null,
            "conditions": [{"field_source": "builtin", "field_key": "name", "operator": "equals",
                "value": "Blocked Co", "compare_field_source": null, "compare_field_key": null,
                "group_id": null, "relationship_definition_id": null}],
            "actions": [{"action_type": "block_save", "target_field_key": null,
                "target_field_source": "builtin", "action_value": null,
                "message": "This name is not allowed"}]
        }}),
    )
    .await;
    assert_eq!(rule["ok"], true, "{rule:?}");

    let single = reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/objects/Company/records"))
        .bearer_auth(&api_key)
        .json(&json!({"name": "Blocked Co", "status": "Prospect"}))
        .send()
        .await
        .unwrap();
    assert!(
        !single.status().is_success(),
        "the single-record API refuses it too"
    );

    let data = bulk_ok(
        addr,
        &api_key,
        "Company",
        json!([create_company("Fine Co"), create_company("Blocked Co")]),
    )
    .await;
    assert_eq!(data["results"][0]["ok"], true, "{data:?}");
    assert_eq!(data["results"][1]["ok"], false, "{data:?}");
    assert_eq!(company_count(addr, &api_key).await, 1);
}
