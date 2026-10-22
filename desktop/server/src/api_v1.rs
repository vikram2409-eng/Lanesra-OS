//! Integration Hub (spec §7/§8/§9): the inbound generic REST API - the
//! one surface where the axum Team Workspace server is a genuinely
//! external-facing process. A pure single-user desktop install has no
//! listening socket at all, so this router (and everything under it)
//! only ever serves external callers when Team Workspace mode is the one
//! actually running - stated plainly here and in the admin UI copy
//! (Integration Hub -> API Access) rather than glossed over.
//!
//! Every route dispatches through `api_object_service`, the exact same
//! generic list/get/create/update/archive functions the desktop admin UI
//! itself calls (via Tauri commands / `/api/invoke`) - so every
//! permission/validation/business-rule check already enforced there fires
//! identically for an external caller, by construction (spec §7.2/§9).
//!
//! Auth is `Authorization: Bearer {client_id}.{secret}`
//! (`api_client_service::authenticate`), entirely separate from the
//! cookie-session auth `/api/invoke/:command` uses - an external caller
//! never gets or needs a session cookie. Every call is rate-limited per
//! client (`RateLimiter`, spec §19) and logged to `integration_executions`
//! (spec §23) whether it succeeds or is rejected before ever reaching
//! `api_object_service`, so a locked-out or misconfigured integration is
//! visible in the same unified log as everything else.

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use lanesra_core::domain::{AppError, ErrorClass};
use lanesra_core::models::integration::{ApiBulkOperation, ApiClient, ApiListQuery};
use lanesra_core::services::data_exchange_service::{self, ExportFormat};
use lanesra_core::services::integration_log_service::{self, FinishOutcome};
use lanesra_core::services::{api_client_service, api_object_service, network_trust};

use crate::caller::Caller;
use crate::dispatch::require_workspace_id;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/api/v1/limits", get(limits))
        .route("/api/v1/objects", get(list_object_keys))
        .route("/api/v1/search", get(search))
        .route("/api/v1/objects/:key/metadata", get(get_metadata))
        .route(
            "/api/v1/objects/:key/records",
            get(list_records).post(create_record),
        )
        .route(
            "/api/v1/objects/:key/records/:id",
            get(get_record).patch(update_record).delete(archive_record),
        )
        .route("/api/v1/objects/:key/export", get(export_records))
        .route(
            "/api/v1/objects/:key/bulk",
            // A full 1000-operation batch is larger than axum's 2 MB default.
            post(bulk_write).layer(DefaultBodyLimit::max(BULK_BODY_LIMIT)),
        )
}

/// The largest bulk-write request body accepted (WP-0187).
const BULK_BODY_LIMIT: usize = 32 * 1024 * 1024;
/// Records encoded per chunk of a streamed export.
const EXPORT_CHUNK: usize = 500;

pub(crate) fn error_status(e: &AppError) -> StatusCode {
    match e {
        AppError::NotFound(_) => StatusCode::NOT_FOUND,
        AppError::Validation(_) => StatusCode::BAD_REQUEST,
        AppError::Conflict(_) => StatusCode::CONFLICT,
        AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AppError::PolicyBlocked(_) => StatusCode::FORBIDDEN,
        // ADR-0009: a missing setup (configuration) is the server's
        // dependency being unavailable, like a transient failure.
        AppError::Classified { class, .. } => match class {
            ErrorClass::Transient | ErrorClass::Configuration => StatusCode::SERVICE_UNAVAILABLE,
            ErrorClass::Policy => StatusCode::FORBIDDEN,
            ErrorClass::Validation => StatusCode::BAD_REQUEST,
            ErrorClass::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        },
    }
}

pub(crate) fn err_json(status: StatusCode, message: &str) -> (StatusCode, Json<Value>) {
    (status, Json(json!({"ok": false, "error": message})))
}

pub(crate) fn app_err(e: AppError) -> (StatusCode, Json<Value>) {
    let status = error_status(&e);
    err_json(status, &e.to_string())
}

/// Verifies the presented API key, its scope, and this client's rate
/// limit - the one gate every route below goes through before touching
/// `api_object_service`. Returns the authenticated client and this
/// server's current workspace id.
pub(crate) fn authorize(
    state: &SharedState,
    headers: &HeaderMap,
    caller: Caller,
    required_scope: &str,
) -> Result<(ApiClient, String), (StatusCode, Json<Value>)> {
    let presented = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            err_json(
                StatusCode::UNAUTHORIZED,
                "Missing 'Authorization: Bearer <client_id>.<secret>' header",
            )
        })?;

    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn).map_err(app_err)?;
    let client =
        api_client_service::authenticate(&conn, &workspace_id, presented).map_err(|_| {
            err_json(
                StatusCode::UNAUTHORIZED,
                "Invalid or revoked API credential",
            )
        })?;
    // ADR-0018: the client's allowed network, checked only after the key
    // verifies so a refusal never tells a guesser that a client id exists.
    if let Some(allowed) = client.allowed_cidr.as_deref() {
        if network_trust::is_restricted(allowed)
            && !caller
                .0
                .is_some_and(|ip| network_trust::client_admits(allowed, ip))
        {
            log_network_refusal(&conn, &workspace_id, &client, caller);
            return Err(err_json(
                StatusCode::FORBIDDEN,
                "This API client is not allowed from this network address",
            ));
        }
    }
    if !api_client_service::has_scope(&client, required_scope) {
        return Err(err_json(
            StatusCode::FORBIDDEN,
            &format!("This API client lacks the '{required_scope}' scope"),
        ));
    }
    let settings = integration_log_service::get_settings(&conn, &workspace_id).map_err(app_err)?;
    if let Err(retry_after) = state
        .rate_limiter
        .check(&client.id, settings.api_rate_limit_per_minute)
    {
        let mut resp = err_json(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded for this API client",
        );
        resp.1 .0["retry_after_seconds"] = json!(retry_after);
        return Err(resp);
    }
    Ok((client, workspace_id))
}

/// Records a network refusal with the address that was resolved, so an admin
/// can see where a leaked or misconfigured key was used from.
fn log_network_refusal(
    conn: &rusqlite::Connection,
    workspace_id: &str,
    client: &ApiClient,
    caller: Caller,
) {
    let from = caller
        .0
        .map_or_else(|| "an unknown address".to_string(), |ip| ip.to_string());
    let execution_id =
        integration_log_service::start(conn, workspace_id, "api_call", None, None, "inbound", None);
    integration_log_service::finish(
        conn,
        &execution_id,
        &FinishOutcome {
            status: "failed".into(),
            http_status: Some(403),
            records_failed: 1,
            error_category: Some("network_blocked".into()),
            error_message: Some(format!(
                "API client '{}' was refused from {from}: outside its allowed network",
                client.name
            )),
            ..Default::default()
        },
    );
}

/// Logs one REST call to `integration_executions` and returns the same
/// `Result` unchanged, so every handler can wrap its `api_object_service`
/// call in one line without duplicating the start/finish bookkeeping.
/// Takes the connection the caller already holds - `state.conn` is a
/// plain (non-reentrant) `std::sync::Mutex`, so this must never lock it
/// again itself, or every handler below would deadlock on its own lock.
///
/// `pub(crate)`, not private: `mcp.rs`'s `POST /mcp` handler reuses this
/// exact helper too, the same execution log either surface writes to -
/// same reasoning as `integration_commands.rs`'s `run_with_own_connection`
/// being loosened to `pub(crate)` in phase 1, rather than duplicating it.
pub(crate) fn logged<T>(
    conn: &rusqlite::Connection,
    workspace_id: &str,
    result: Result<T, AppError>,
) -> Result<T, AppError> {
    let execution_id =
        integration_log_service::start(conn, workspace_id, "api_call", None, None, "inbound", None);
    match &result {
        Ok(_) => integration_log_service::finish(
            conn,
            &execution_id,
            &FinishOutcome {
                status: "success".into(),
                records_written: 1,
                ..Default::default()
            },
        ),
        Err(e) => integration_log_service::finish(
            conn,
            &execution_id,
            &FinishOutcome {
                status: "failed".into(),
                records_failed: 1,
                error_category: Some("api_error".into()),
                error_message: Some(e.to_string()),
                ..Default::default()
            },
        ),
    }
    result
}

async fn limits(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (client, workspace_id) = authorize(&state, &headers, caller, "metadata.read")?;
    let conn = state.conn.lock().unwrap();
    let settings = integration_log_service::get_settings(&conn, &workspace_id).map_err(app_err)?;
    Ok(Json(json!({
        "ok": true,
        "data": {
            "api_rate_limit_per_minute": settings.api_rate_limit_per_minute,
            "scopes": client.scopes,
        }
    })))
}

async fn list_object_keys(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (_client, workspace_id) = authorize(&state, &headers, caller, "metadata.read")?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        api_object_service::list_object_keys(&conn, &workspace_id),
    )
    .map_err(app_err)?;
    Ok(Json(json!({"ok": true, "data": data})))
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    q: Option<String>,
    object: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

/// WP-0186: finds records by text across every object this API can read.
async fn search(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Query(params): Query<SearchParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (_client, workspace_id) = authorize(&state, &headers, caller, "search.read")?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        api_object_service::search_records(
            &conn,
            &workspace_id,
            params.q.as_deref().unwrap_or_default(),
            params.object.as_deref(),
            params.page,
            params.page_size,
        ),
    )
    .map_err(app_err)?;
    Ok(Json(json!({"ok": true, "data": data})))
}

async fn get_metadata(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(key): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (_client, workspace_id) = authorize(&state, &headers, caller, "metadata.read")?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        api_object_service::get_metadata(&conn, &workspace_id, &key),
    )
    .map_err(app_err)?;
    Ok(Json(json!({"ok": true, "data": data})))
}

#[derive(Debug, Deserialize)]
struct ListQueryParams {
    select: Option<String>,
    sort: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
    filter: Option<String>,
}

fn to_api_list_query(q: ListQueryParams) -> ApiListQuery {
    ApiListQuery {
        select: q
            .select
            .map(|s| s.split(',').map(str::trim).map(String::from).collect()),
        sort: q
            .sort
            .map(|s| s.split(',').map(str::trim).map(String::from).collect()),
        page: q.page,
        page_size: q.page_size,
        filter: q.filter.and_then(|f| serde_json::from_str(&f).ok()),
    }
}

async fn list_records(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(key): Path<String>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (_client, workspace_id) = authorize(&state, &headers, caller, "objects.read")?;
    let query = to_api_list_query(params);
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        api_object_service::list_records(&conn, &workspace_id, &key, &query),
    )
    .map_err(app_err)?;
    Ok(Json(json!({"ok": true, "data": data})))
}

async fn get_record(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path((key, id)): Path<(String, String)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (_client, workspace_id) = authorize(&state, &headers, caller, "objects.read")?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        api_object_service::get_record(&conn, &workspace_id, &key, &id),
    )
    .map_err(app_err)?;
    Ok(Json(json!({"ok": true, "data": data})))
}

async fn create_record(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(key): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (_client, workspace_id) = authorize(&state, &headers, caller, "objects.write")?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        api_object_service::create_record(&conn, &workspace_id, &key, &body, None),
    )
    .map_err(app_err)?;
    Ok(Json(json!({"ok": true, "data": data})))
}

async fn update_record(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path((key, id)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (_client, workspace_id) = authorize(&state, &headers, caller, "objects.write")?;
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        api_object_service::update_record(&conn, &workspace_id, &key, &id, &body, None),
    )
    .map_err(app_err)?;
    Ok(Json(json!({"ok": true, "data": data})))
}

async fn archive_record(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path((key, id)): Path<(String, String)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (_client, workspace_id) = authorize(&state, &headers, caller, "objects.write")?;
    let conn = state.conn.lock().unwrap();
    logged(
        &conn,
        &workspace_id,
        api_object_service::archive_record(&conn, &workspace_id, &key, &id, None),
    )
    .map_err(app_err)?;
    Ok(Json(json!({"ok": true})))
}

#[derive(Debug, Deserialize)]
struct ExportParams {
    format: Option<String>,
}

/// WP-0187: every non-archived record of an object as NDJSON (default) or
/// CSV. The body is encoded and sent in chunks, so the encoded export is
/// never held in memory whole.
async fn export_records(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(key): Path<String>,
    Query(params): Query<ExportParams>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let (_client, workspace_id) = authorize(&state, &headers, caller, "bulk.read")?;
    let format = ExportFormat::parse(params.format.as_deref()).map_err(app_err)?;
    let records = {
        let conn = state.conn.lock().unwrap();
        logged(
            &conn,
            &workspace_id,
            api_object_service::all_records(&conn, &workspace_id, &key),
        )
        .map_err(app_err)?
    };
    let columns = data_exchange_service::export_columns(&records);
    let stream = async_stream::stream! {
        let mut chunks = records.chunks(EXPORT_CHUNK).peekable();
        if chunks.peek().is_none() && format == ExportFormat::Csv {
            yield data_exchange_service::encode_export_chunk(format, &columns, &[], true)
                .map_err(|e| std::io::Error::other(e.to_string()));
        }
        let mut first = true;
        for chunk in chunks {
            yield data_exchange_service::encode_export_chunk(format, &columns, chunk, first)
                .map_err(|e| std::io::Error::other(e.to_string()));
            first = false;
        }
    };
    let extension = if format == ExportFormat::Csv {
        "csv"
    } else {
        "ndjson"
    };
    Response::builder()
        .header(header::CONTENT_TYPE, format.content_type())
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{key}.{extension}\""),
        )
        .body(Body::from_stream(stream))
        .map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))
}

#[derive(Debug, Deserialize)]
struct BulkBody {
    operations: Vec<ApiBulkOperation>,
}

/// WP-0187: up to 1000 create, update or archive operations on one object,
/// each applied on its own through the validated write path. 200 with a
/// result per item; 413 (nothing applied) above the operation cap.
async fn bulk_write(
    State(state): State<SharedState>,
    headers: HeaderMap,
    caller: Caller,
    Path(key): Path<String>,
    Json(body): Json<BulkBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (_client, workspace_id) = authorize(&state, &headers, caller, "bulk.write")?;
    if body.operations.len() > api_object_service::MAX_BULK_OPERATIONS {
        return Err(err_json(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!(
                "A bulk write carries at most {} operations; this one has {}",
                api_object_service::MAX_BULK_OPERATIONS,
                body.operations.len()
            ),
        ));
    }
    let conn = state.conn.lock().unwrap();
    let data = logged(
        &conn,
        &workspace_id,
        api_object_service::bulk_write(&conn, &workspace_id, &key, &body.operations, None),
    )
    .map_err(app_err)?;
    Ok(Json(json!({"ok": true, "data": data})))
}
