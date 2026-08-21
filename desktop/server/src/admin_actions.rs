//! Integration Hub: the handful of genuinely-async admin actions
//! (Test Connection, Test Action, Test Delivery, Run Now, and a live
//! External Object preview) that can't go through the plain-sync
//! `/api/invoke` dispatcher - see `dispatch.rs`'s own note on why. These
//! routes reuse the *session cookie* the admin UI already holds (not the
//! Bearer API-key scheme `/api/v1` uses for external callers). Any
//! admin-only gating happens inside the core service functions
//! themselves, exactly like every other dispatch/Tauri command - see
//! `authorize`'s own doc comment.
//!
//! Every handler runs its actual work through `run_with_own_connection`
//! rather than a plain `.await` against a directly-opened connection -
//! a real bug caught only by compiling this (not by inspection): each
//! core function here makes a real outbound call while holding
//! `&rusqlite::Connection` across that `.await`, and `Connection` isn't
//! `Sync`, so `&Connection` isn't `Send` - which axum's `Handler` bound
//! requires of the whole future a route handler produces (the same
//! problem `job_scheduler` and the Tauri integration commands hit, fixed
//! the same way: hand the real work to a blocking-pool thread running
//! its own throwaway `current_thread` Tokio runtime via `block_on`, which
//! has no such requirement since nothing there is ever moved to another
//! thread mid-await).

use std::future::Future;
use std::path::PathBuf;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use serde_json::{json, Value};

use lanesra_core::domain::AppError;
use lanesra_core::services::{connection_service, connector_execution_service, external_object_service, integration_job_service, webhook_service};

use crate::dispatch::{require_workspace_id, resolve_master_key, to_value};
use crate::session::current_actor;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/api/admin/connections/:id/test", post(test_connection))
        .route("/api/admin/connectors/:connector_id/actions/:action_key/test", post(test_connector_action))
        .route("/api/admin/webhooks/:id/test", post(test_webhook_delivery))
        .route("/api/admin/jobs/:id/run", post(run_integration_job_now))
        .route("/api/admin/external-objects/:object_key/preview", get(preview_external_object_records))
}

fn err_json(status: StatusCode, message: &str) -> (StatusCode, Json<Value>) {
    (status, Json(json!({"ok": false, "error": message})))
}

fn error_status(e: &AppError) -> StatusCode {
    match e {
        AppError::NotFound(_) => StatusCode::NOT_FOUND,
        AppError::Validation(_) => StatusCode::BAD_REQUEST,
        AppError::Conflict(_) => StatusCode::CONFLICT,
        AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn app_err(e: AppError) -> (StatusCode, Json<Value>) {
    let status = error_status(&e);
    err_json(status, &e.to_string())
}

/// Verifies the session cookie belongs to an authenticated user and
/// resolves this workspace's id + secret master key - the one gate every
/// route below goes through before handing off to `run_with_own_connection`.
/// Admin-only enforcement itself happens inside each core service
/// function (the same `require_admin` check every other dispatch/Tauri
/// command relies on), not here - `external_object_service::list_records`
/// and `connector_execution_service::execute` are deliberately *not*
/// admin-gated at the service layer (read-only preview; Workflow
/// Automation's own runtime calls `execute` with no actor at all), so
/// this transport-level check only ever asks "is this a real, logged-in
/// session," matching `dispatch()`'s own catch-all arm.
fn authorize(state: &SharedState, jar: &CookieJar) -> Result<(String, String, [u8; 32]), (StatusCode, Json<Value>)> {
    let conn = state.conn.lock().unwrap();
    let actor = current_actor(&conn, jar).ok_or_else(|| err_json(StatusCode::UNAUTHORIZED, "Not authenticated - please log in"))?;
    let workspace_id = require_workspace_id(&conn).map_err(app_err)?;
    drop(conn);
    let master_key = resolve_master_key(&state.db_path).map_err(app_err)?;
    Ok((workspace_id, actor, master_key))
}

/// Runs `f` - an async closure handed a fresh, exclusively-owned
/// `rusqlite::Connection` opened from `db_path` - to completion on a
/// blocking-pool thread with its own single-threaded Tokio runtime. See
/// this module's own doc comment for why a plain `.await` doesn't work
/// here.
async fn run_with_own_connection<T, F, Fut>(db_path: PathBuf, f: F) -> Result<T, (StatusCode, Json<Value>)>
where
    T: Send + 'static,
    F: FnOnce(rusqlite::Connection) -> Fut + Send + 'static,
    Fut: Future<Output = Result<T, AppError>>,
{
    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| AppError::Validation(format!("could not start a background runtime: {e}")))?;
        let conn = lanesra_core::db::open_workspace_db(&db_path).map_err(AppError::from)?;
        rt.block_on(f(conn))
    })
    .await
    .map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, &format!("background task panicked: {e}")))?
    .map_err(app_err)
}

async fn test_connection(State(state): State<SharedState>, jar: CookieJar, Path(id): Path<String>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (workspace_id, actor, master_key) = authorize(&state, &jar)?;
    let db_path = state.db_path.clone();
    let data = run_with_own_connection(db_path, move |conn| async move {
        connection_service::test_connection(&conn, &workspace_id, &master_key, &id, Some(&actor)).await
    })
    .await?;
    Ok(Json(json!({"ok": true, "data": to_value(data).map_err(app_err)?})))
}

#[derive(Debug, Deserialize)]
struct TestActionBody {
    reference_key: String,
    params: Value,
}

async fn test_connector_action(
    State(state): State<SharedState>,
    jar: CookieJar,
    Path((connector_id, action_key)): Path<(String, String)>,
    Json(body): Json<TestActionBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (workspace_id, actor, master_key) = authorize(&state, &jar)?;
    let db_path = state.db_path.clone();
    let data = run_with_own_connection(db_path, move |conn| async move {
        connector_execution_service::execute(&conn, &workspace_id, &master_key, &connector_id, &action_key, &body.reference_key, &body.params, Some(&actor)).await
    })
    .await?;
    Ok(Json(json!({"ok": true, "data": to_value(data).map_err(app_err)?})))
}

async fn test_webhook_delivery(State(state): State<SharedState>, jar: CookieJar, Path(id): Path<String>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (workspace_id, actor, master_key) = authorize(&state, &jar)?;
    let db_path = state.db_path.clone();
    run_with_own_connection(db_path, move |conn| async move { webhook_service::test_delivery(&conn, &workspace_id, &master_key, &id, Some(&actor)).await }).await?;
    Ok(Json(json!({"ok": true})))
}

async fn run_integration_job_now(State(state): State<SharedState>, jar: CookieJar, Path(id): Path<String>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (workspace_id, actor, master_key) = authorize(&state, &jar)?;
    let db_path = state.db_path.clone();
    let data = run_with_own_connection(db_path, move |conn| async move {
        integration_job_service::run_now(&conn, &workspace_id, &master_key, &id, Some(&actor)).await
    })
    .await?;
    Ok(Json(json!({"ok": true, "data": to_value(data).map_err(app_err)?})))
}

async fn preview_external_object_records(State(state): State<SharedState>, jar: CookieJar, Path(object_key): Path<String>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (workspace_id, _actor, master_key) = authorize(&state, &jar)?;
    let db_path = state.db_path.clone();
    let data = run_with_own_connection(db_path, move |conn| async move { external_object_service::list_records(&conn, &workspace_id, &master_key, &object_key).await }).await?;
    Ok(Json(json!({"ok": true, "data": data})))
}
