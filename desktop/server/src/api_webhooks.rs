//! WP-0188: webhook management on the inbound REST API, behind the
//! `webhooks.manage` scope (ADR-0018 section 3).
//!
//! Every route goes through `api_v1::authorize`, so the key, the client's
//! allowed network and the scope are checked exactly as for records. The
//! calls act as the API client, not as an Administrator: the webhook
//! service is told who is managing (`WebhookManager::ApiClient`) and audits
//! creation, update, rotation and deletion under the client's id. The
//! signing secret is returned only by create and rotate-secret; every read
//! returns `has_secret`. A webhook's URL always comes from an existing
//! `webhook` connection, which this API cannot create or edit, and must pass
//! the outbound guard.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use lanesra_core::models::integration::{ApiClient, WebhookInput};
use lanesra_core::services::webhook_service::{self, WebhookManager};

use crate::admin_actions::run_with_own_connection;
use crate::api_v1::{app_err, authorize, logged};
use crate::caller::Caller;
use crate::dispatch::resolve_master_key;
use crate::state::SharedState;

const SCOPE: &str = "webhooks.manage";

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/api/v1/webhooks", get(list).post(create))
        .route(
            "/api/v1/webhooks/:id",
            get(read).patch(update).delete(remove),
        )
        .route("/api/v1/webhooks/:id/pause", post(pause))
        .route("/api/v1/webhooks/:id/reactivate", post(reactivate))
        .route("/api/v1/webhooks/:id/rotate-secret", post(rotate_secret))
        .route("/api/v1/webhooks/:id/test", post(test))
        .route("/api/v1/webhooks/:id/deliveries", get(deliveries))
}

type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;

fn manager(client: &ApiClient) -> WebhookManager<'_> {
    WebhookManager::ApiClient {
        id: &client.id,
        name: &client.name,
    }
}

fn ok(data: impl serde::Serialize) -> ApiResult {
    Ok(Json(json!({"ok": true, "data": data})))
}

async fn list(State(state): State<SharedState>, headers: HeaderMap, caller: Caller) -> ApiResult {
    let (_client, workspace_id) = authorize(&state, &headers, caller, SCOPE)?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        webhook_service::list_for_workspace(&conn, &workspace_id),
    )
    .map_err(app_err)?;
    ok(data)
}

async fn create(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Json(input): Json<WebhookInput>,
) -> ApiResult {
    let (client, workspace_id) = authorize(&state, &headers, caller, SCOPE)?;
    let master_key = resolve_master_key(&state.db_path).map_err(app_err)?;
    let conn = state.conn.lock().unwrap();
    // The one response that carries the signing secret, besides rotation.
    let data = logged(
        &conn,
        &workspace_id,
        webhook_service::create_with_secret_by(
            &conn,
            &workspace_id,
            &master_key,
            &input,
            manager(&client),
        ),
    )
    .map_err(app_err)?;
    ok(data)
}

async fn read(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
) -> ApiResult {
    let (_client, workspace_id) = authorize(&state, &headers, caller, SCOPE)?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        webhook_service::get(&conn, &workspace_id, &id),
    )
    .map_err(app_err)?;
    ok(data)
}

async fn update(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
    Json(input): Json<WebhookInput>,
) -> ApiResult {
    let (client, workspace_id) = authorize(&state, &headers, caller, SCOPE)?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        webhook_service::update_by(&conn, &workspace_id, &id, &input, manager(&client)),
    )
    .map_err(app_err)?;
    ok(data)
}

async fn remove(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
) -> ApiResult {
    let (client, workspace_id) = authorize(&state, &headers, caller, SCOPE)?;
    let conn = state.conn.lock().unwrap();
    logged(
        &conn,
        &workspace_id,
        webhook_service::delete_by(&conn, &workspace_id, &id, manager(&client)),
    )
    .map_err(app_err)?;
    Ok(Json(json!({"ok": true})))
}

async fn set_status(
    state: SharedState,
    headers: HeaderMap,
    caller: Caller,
    id: String,
    status: &str,
) -> ApiResult {
    let (client, workspace_id) = authorize(&state, &headers, caller, SCOPE)?;
    let conn = state.conn.lock().unwrap();
    logged(
        &conn,
        &workspace_id,
        webhook_service::set_status_by(&conn, &workspace_id, &id, status, manager(&client))
            .and_then(|()| webhook_service::get(&conn, &workspace_id, &id)),
    )
    .map_err(app_err)
    .and_then(ok)
}

async fn pause(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
) -> ApiResult {
    set_status(state, headers, caller, id, "paused").await
}

async fn reactivate(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
) -> ApiResult {
    set_status(state, headers, caller, id, "active").await
}

async fn rotate_secret(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
) -> ApiResult {
    let (client, workspace_id) = authorize(&state, &headers, caller, SCOPE)?;
    let master_key = resolve_master_key(&state.db_path).map_err(app_err)?;
    let conn = state.conn.lock().unwrap();
    let secret = logged(
        &conn,
        &workspace_id,
        webhook_service::rotate_secret_by(&conn, &workspace_id, &master_key, &id, manager(&client)),
    )
    .map_err(app_err)?;
    ok(json!({"signing_secret": secret}))
}

/// Sends a synthetic delivery. It makes a network call, so it runs on its
/// own connection instead of holding the shared one across the request.
async fn test(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
) -> ApiResult {
    let (client, workspace_id) = authorize(&state, &headers, caller, SCOPE)?;
    let master_key = resolve_master_key(&state.db_path).map_err(app_err)?;
    let db_path = state.db_path.clone();
    run_with_own_connection(db_path, move |conn| async move {
        let result = webhook_service::test_delivery_by(
            &conn,
            &workspace_id,
            &master_key,
            &id,
            manager(&client),
        )
        .await;
        logged(&conn, &workspace_id, result)
    })
    .await?;
    Ok(Json(json!({"ok": true})))
}

#[derive(Debug, Deserialize)]
struct PageParams {
    page: Option<i64>,
    page_size: Option<i64>,
}

async fn deliveries(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
    Query(params): Query<PageParams>,
) -> ApiResult {
    let (_client, workspace_id) = authorize(&state, &headers, caller, SCOPE)?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        webhook_service::list_deliveries_page(
            &conn,
            &workspace_id,
            &id,
            params.page,
            params.page_size,
        ),
    )
    .map_err(app_err)?;
    ok(data)
}
