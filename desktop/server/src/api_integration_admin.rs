//! WP-0189: integration administration on the inbound REST API.
//!
//! `admin.integration.read` lists connections, connectors, jobs and API
//! clients, metadata only. `admin.integration.manage` pauses and
//! reactivates connections and jobs and runs a job now. Neither scope can
//! read a secret, create anything or change a scope; each route goes
//! through `api_v1::authorize`, and changes are audited under the client.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};

use lanesra_core::models::integration::ApiClient;
use lanesra_core::services::integration_admin_service::{self, ApiCaller};

use crate::admin_actions::run_with_own_connection;
use crate::api_v1::{app_err, authorize, logged};
use crate::caller::Caller;
use crate::dispatch::resolve_master_key;
use crate::state::SharedState;

const READ: &str = "admin.integration.read";
const MANAGE: &str = "admin.integration.manage";

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/api/v1/integration/connections", get(connections))
        .route("/api/v1/integration/connectors", get(connectors))
        .route("/api/v1/integration/jobs", get(jobs))
        .route("/api/v1/integration/api-clients", get(api_clients))
        .route(
            "/api/v1/integration/connections/:id/pause",
            post(pause_connection),
        )
        .route(
            "/api/v1/integration/connections/:id/reactivate",
            post(reactivate_connection),
        )
        .route("/api/v1/integration/jobs/:id/pause", post(pause_job))
        .route(
            "/api/v1/integration/jobs/:id/reactivate",
            post(reactivate_job),
        )
        .route("/api/v1/integration/jobs/:id/run", post(run_job))
}

type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;

fn caller_of(client: &ApiClient) -> ApiCaller<'_> {
    ApiCaller {
        id: &client.id,
        name: &client.name,
    }
}

fn ok(data: impl serde::Serialize) -> ApiResult {
    Ok(Json(json!({"ok": true, "data": data})))
}

macro_rules! listing {
    ($name:ident, $service:path) => {
        async fn $name(
            State(state): State<SharedState>,
            headers: HeaderMap,
            caller: Caller,
        ) -> ApiResult {
            let (_client, workspace_id) = authorize(&state, &headers, caller, READ)?;
            let conn = state.conn.lock().unwrap();
            let data =
                logged(&conn, &workspace_id, $service(&conn, &workspace_id)).map_err(app_err)?;
            ok(data)
        }
    };
}

listing!(connections, integration_admin_service::list_connections);
listing!(connectors, integration_admin_service::list_connectors);
listing!(jobs, integration_admin_service::list_jobs);
listing!(api_clients, integration_admin_service::list_api_clients);

async fn connection_status(
    state: SharedState,
    headers: HeaderMap,
    caller: Caller,
    id: String,
    paused: bool,
) -> ApiResult {
    let (client, workspace_id) = authorize(&state, &headers, caller, MANAGE)?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        integration_admin_service::set_connection_paused(
            &conn,
            &workspace_id,
            &id,
            paused,
            caller_of(&client),
        ),
    )
    .map_err(app_err)?;
    ok(data)
}

async fn pause_connection(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
) -> ApiResult {
    connection_status(state, headers, caller, id, true).await
}

async fn reactivate_connection(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
) -> ApiResult {
    connection_status(state, headers, caller, id, false).await
}

async fn job_status(
    state: SharedState,
    headers: HeaderMap,
    caller: Caller,
    id: String,
    paused: bool,
) -> ApiResult {
    let (client, workspace_id) = authorize(&state, &headers, caller, MANAGE)?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        integration_admin_service::set_job_paused(
            &conn,
            &workspace_id,
            &id,
            paused,
            caller_of(&client),
        ),
    )
    .map_err(app_err)?;
    ok(data)
}

async fn pause_job(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
) -> ApiResult {
    job_status(state, headers, caller, id, true).await
}

async fn reactivate_job(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
) -> ApiResult {
    job_status(state, headers, caller, id, false).await
}

/// Runs a job now. It calls out to the external system, so it runs on its
/// own connection instead of holding the shared one across the request.
async fn run_job(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(id): Path<String>,
) -> ApiResult {
    let (client, workspace_id) = authorize(&state, &headers, caller, MANAGE)?;
    let master_key = resolve_master_key(&state.db_path).map_err(app_err)?;
    let db_path = state.db_path.clone();
    let run = run_with_own_connection(db_path, move |conn| async move {
        integration_admin_service::run_job_now(
            &conn,
            &workspace_id,
            &master_key,
            &id,
            caller_of(&client),
        )
        .await
    })
    .await?;
    ok(run)
}
