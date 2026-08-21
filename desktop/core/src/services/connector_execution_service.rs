//! Integration Hub (spec §6.3/§17): invokes one Connector Action against
//! the physical Connection a Connection Reference currently resolves to -
//! the one function both the future "Call Connector Action" Workflow step
//! (task #310) and an admin's manual "Test this Action" button call.
//!
//! Deliberately thin: no per-action retry/backoff (that's a Workflow-level
//! concern if the caller wants it, same as every other Workflow action),
//! no response-schema validation (the Connector Action's schema is
//! metadata for the UI/workflow field picker, not a runtime contract this
//! layer enforces) - just resolve reference -> connection -> secret, build
//! one HTTP request from the action's template + supplied params, send it,
//! and log the outcome the same way every other outbound call in this
//! subsystem does (`integration_log_service`).

use std::time::{Duration, Instant};

use rusqlite::Connection;
use serde_json::Value;

use crate::domain::{AppError, AppResult};
use crate::models::integration::ConnectorExecutionResult;
use crate::repositories::integration_pending_action_repo;
use crate::services::integration_log_service::{self, FinishOutcome};

fn substitute_path(template: &str, params: &Value) -> AppResult<String> {
    let mut path = template.to_string();
    while let Some(start) = path.find('{') {
        let end = path[start..].find('}').map(|i| start + i).ok_or_else(|| AppError::Validation(format!("Path template '{template}' has an unmatched '{{'")))?;
        let name = &path[start + 1..end];
        let value = params
            .get(name)
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| Some(v.to_string())))
            .ok_or_else(|| AppError::Validation(format!("Missing required path parameter '{name}'")))?;
        path.replace_range(start..=end, &value);
    }
    Ok(path)
}

/// Invokes `action_key` on `connector_id`, against whatever physical
/// Connection `reference_key` currently resolves to in this workspace.
/// `params` is a flat JSON object keyed by each `ConnectorActionParam`'s
/// `name` - the `"body"` param (if the action has one) supplies the
/// request body value as-is.
///
/// The log entry is opened *before* connector/reference/connection
/// resolution, not just before the HTTP call - a bad connector id, an
/// unbound Connection Reference, or a malformed path template is exactly
/// the kind of failure an admin needs to see in the unified log, not one
/// that silently vanishes because it happened "too early" to log.
pub async fn execute(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    connector_id: &str,
    action_key: &str,
    reference_key: &str,
    params: &Value,
    actor_user_id: Option<&str>,
) -> AppResult<ConnectorExecutionResult> {
    let execution_id = integration_log_service::start(conn, workspace_id, "connector_action", None, Some(connector_id), "outbound", actor_user_id);
    let started = Instant::now();

    let result = build_and_send(conn, workspace_id, master_key, connector_id, action_key, reference_key, params).await;
    let duration_ms = started.elapsed().as_millis() as u64;

    match result {
        Ok((status_code, response_body, ok, message)) => {
            integration_log_service::finish(
                conn,
                &execution_id,
                &FinishOutcome {
                    status: if ok { "success".into() } else { "failed".into() },
                    http_status: status_code.map(|s| s as i64),
                    records_written: if ok { 1 } else { 0 },
                    records_failed: if ok { 0 } else { 1 },
                    error_category: if ok { None } else { Some("http_error".into()) },
                    error_message: if ok { None } else { Some(message.clone()) },
                    ..Default::default()
                },
            );
            Ok(ConnectorExecutionResult { ok, status_code, duration_ms, response_body, message })
        }
        Err(e) => {
            let message = e.to_string();
            integration_log_service::finish(
                conn,
                &execution_id,
                &FinishOutcome { status: "failed".into(), records_failed: 1, error_category: Some("resolution_error".into()), error_message: Some(message.clone()), ..Default::default() },
            );
            Ok(ConnectorExecutionResult { ok: false, status_code: None, duration_ms, response_body: Value::Null, message })
        }
    }
}

/// The fallible core of `execute` - resolves connector/action/reference/
/// connection/secret, builds and sends one HTTP request. Returns
/// `(status_code, response_body, ok, message)` on any outcome that got as
/// far as building a request; an `Err` here means resolution failed
/// before a request could even be attempted.
async fn build_and_send(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    connector_id: &str,
    action_key: &str,
    reference_key: &str,
    params: &Value,
) -> AppResult<(Option<u16>, Value, bool, String)> {
    let connector = super::connector_service::get(conn, workspace_id, connector_id)?;
    let action = connector
        .actions
        .iter()
        .find(|a| a.action_key == action_key)
        .ok_or_else(|| AppError::NotFound(format!("Action '{action_key}' on connector '{}'", connector.name)))?
        .clone();

    let connection_id = super::connection_ref_service::resolve(conn, workspace_id, reference_key)?;
    let connection = super::connection_service::get(conn, workspace_id, &connection_id)?;
    let secret = super::connection_service::resolve_secret(conn, master_key, &connection_id)?;
    let base_url = connection.base_url.as_deref().ok_or_else(|| AppError::Validation("The bound connection has no base URL configured".into()))?;

    let path = substitute_path(&action.path_template, params)?;
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    let method = reqwest::Method::from_bytes(action.http_method.as_bytes()).map_err(|_| AppError::Validation(format!("Invalid HTTP method '{}'", action.http_method)))?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let mut builder = client.request(method, &url);

    for p in &action.params {
        match p.location.as_str() {
            "query" => {
                if let Some(v) = params.get(&p.name) {
                    builder = builder.query(&[(p.name.as_str(), value_as_query_string(v))]);
                }
            }
            "header" => {
                if let Some(v) = params.get(&p.name).and_then(|v| v.as_str()) {
                    builder = builder.header(p.name.as_str(), v);
                }
            }
            "body" => {
                if let Some(v) = params.get(&p.name) {
                    builder = builder.json(v);
                }
            }
            _ => {}
        }
    }
    builder = super::connection_service::apply_auth(builder, &connection.auth_mode, secret.as_deref());

    match builder.send().await {
        Ok(response) => {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            let response_body: Value = serde_json::from_str(&body_text).unwrap_or(Value::String(body_text));
            let ok = status.is_success();
            let message = if ok { format!("HTTP {status}") } else { format!("Action returned HTTP {status}") };
            Ok((Some(status.as_u16()), response_body, ok, message))
        }
        Err(e) => Ok((None, Value::Null, false, format!("Could not reach {url}: {e}"))),
    }
}

fn value_as_query_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Drains up to `limit` queued "call_connector_action" workflow actions for
/// one workspace, actually invoking each one and logging the outcome (via
/// `execute`, which logs internally) - meant to be called from whatever
/// cadence already polls for this workspace's async work (the server's
/// scheduler loop, or desktop's client-poll pattern for scheduled
/// workflows), not from inside the sync `apply_action` call site itself.
/// Each row is removed after one attempt regardless of outcome: a failed
/// call is already recorded in `integration_executions` for the admin to
/// see and re-trigger manually if needed - this queue is a hand-off, not a
/// retry mechanism (unlike `webhook_service`'s delivery retries, which
/// exist because a subscriber genuinely expects eventual delivery).
pub async fn drain_pending_actions(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], limit: i64) -> AppResult<usize> {
    let batch = integration_pending_action_repo::list_batch(conn, workspace_id, limit)?;
    let mut drained = 0;
    for item in &batch {
        let params: Value = serde_json::from_str(&item.params_json).unwrap_or(Value::Null);
        let _ = execute(conn, workspace_id, master_key, &item.connector_id, &item.action_key, &item.reference_key, &params, None).await;
        integration_pending_action_repo::delete(conn, &item.id)?;
        drained += 1;
    }
    Ok(drained)
}
