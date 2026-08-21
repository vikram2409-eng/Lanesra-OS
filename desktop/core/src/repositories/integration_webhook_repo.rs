//! Raw CRUD for `integration_webhooks`/`integration_webhook_deliveries`
//! (migration 0032) - see `services::webhook_service` for signing,
//! delivery and retry.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::integration::{Webhook, WebhookDelivery};

fn map_webhook(row: &rusqlite::Row) -> rusqlite::Result<Webhook> {
    let event_types_json: String = row.get("event_types_json")?;
    let secret_id: Option<String> = row.get("secret_id")?;
    Ok(Webhook {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        connection_id: row.get("connection_id")?,
        endpoint_url: row.get("endpoint_url")?,
        event_types: serde_json::from_str(&event_types_json).unwrap_or_default(),
        object_scope: row.get("object_scope")?,
        filter_json: row.get("filter_json")?,
        payload_version: row.get("payload_version")?,
        has_secret: secret_id.is_some(),
        retry_policy_json: row.get("retry_policy_json")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

const SELECT: &str = "SELECT w.*, c.base_url AS endpoint_url FROM integration_webhooks w JOIN integration_connections c ON c.id = w.connection_id";

#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    name: &str,
    connection_id: &str,
    event_types_json: &str,
    object_scope: Option<&str>,
    filter_json: Option<&str>,
    payload_version: &str,
    secret_id: Option<&str>,
    retry_policy_json: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Webhook> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO integration_webhooks (id, workspace_id, name, connection_id, event_types_json, object_scope, filter_json, payload_version, secret_id, retry_policy_json, status, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'active', ?11, ?12, ?11, ?12)",
        rusqlite::params![id, workspace_id, name, connection_id, event_types_json, object_scope, filter_json, payload_version, secret_id, retry_policy_json, now, actor_user_id],
    )?;
    get(conn, id).map(|w| w.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Webhook>> {
    conn.query_row(&format!("{SELECT} WHERE w.id = ?1"), [id], map_webhook)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Webhook>> {
    let mut stmt = conn.prepare(&format!("{SELECT} WHERE w.workspace_id = ?1 ORDER BY w.name"))?;
    let rows = stmt.query_map([workspace_id], map_webhook)?;
    rows.collect()
}

/// Every active webhook subscribed to `event_type` in this workspace -
/// what a fired internal event fans out to.
pub fn list_active_for_event(conn: &Connection, workspace_id: &str, event_type: &str) -> rusqlite::Result<Vec<Webhook>> {
    let all = list_for_workspace(conn, workspace_id)?;
    Ok(all.into_iter().filter(|w| w.status == "active" && w.event_types.iter().any(|e| e == event_type)).collect())
}

pub fn set_status(conn: &Connection, id: &str, status: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE integration_webhooks SET status = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![status, now_iso(), id])?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM integration_webhooks WHERE id = ?1", [id])?;
    Ok(())
}

fn map_delivery(row: &rusqlite::Row) -> rusqlite::Result<WebhookDelivery> {
    Ok(WebhookDelivery {
        id: row.get("id")?,
        webhook_id: row.get("webhook_id")?,
        event_id: row.get("event_id")?,
        event_type: row.get("event_type")?,
        attempt_number: row.get("attempt_number")?,
        status: row.get("status")?,
        http_status: row.get("http_status")?,
        duration_ms: row.get("duration_ms")?,
        response_snippet: row.get("response_snippet")?,
        created_at: row.get("created_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn insert_delivery(
    conn: &Connection,
    id: &str,
    webhook_id: &str,
    event_id: &str,
    event_type: &str,
    payload_json: &str,
    attempt_number: i64,
    status: &str,
    http_status: Option<i64>,
    duration_ms: Option<i64>,
    response_snippet: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO integration_webhook_deliveries (id, webhook_id, event_id, event_type, payload_json, attempt_number, status, http_status, duration_ms, response_snippet, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![id, webhook_id, event_id, event_type, payload_json, attempt_number, status, http_status, duration_ms, response_snippet, now_iso()],
    )?;
    Ok(())
}

pub fn list_deliveries(conn: &Connection, webhook_id: &str, limit: i64) -> rusqlite::Result<Vec<WebhookDelivery>> {
    let mut stmt = conn.prepare("SELECT * FROM integration_webhook_deliveries WHERE webhook_id = ?1 ORDER BY created_at DESC LIMIT ?2")?;
    let rows = stmt.query_map(rusqlite::params![webhook_id, limit], map_delivery)?;
    rows.collect()
}

/// Consecutive failures since the last success - used to decide when a
/// webhook should flip to Degraded/Paused (spec §10.3).
pub fn consecutive_failures(conn: &Connection, webhook_id: &str) -> rusqlite::Result<i64> {
    let mut stmt = conn.prepare("SELECT status FROM integration_webhook_deliveries WHERE webhook_id = ?1 ORDER BY created_at DESC LIMIT 20")?;
    let statuses: Vec<String> = stmt.query_map([webhook_id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    Ok(statuses.iter().take_while(|s| s.as_str() == "failed").count() as i64)
}

/// How many delivery attempts have failed workspace-wide since `since_iso`
/// - the Overview screen's "failed webhooks today" KPI (spec §3.1).
pub fn count_failed_deliveries_since(conn: &Connection, workspace_id: &str, since_iso: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM integration_webhook_deliveries d
         JOIN integration_webhooks w ON w.id = d.webhook_id
         WHERE w.workspace_id = ?1 AND d.status = 'failed' AND d.created_at >= ?2",
        rusqlite::params![workspace_id, since_iso],
        |r| r.get(0),
    )
}
