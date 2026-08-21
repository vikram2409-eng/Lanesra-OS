//! Raw CRUD for `integration_pending_events` (migration 0032) - see
//! `services::event_hooks` for what enqueues these and
//! `services::webhook_service::drain_pending_events` for what delivers
//! and clears them.

use rusqlite::Connection;

use crate::domain::ids::now_iso;

pub struct PendingEvent {
    pub id: String,
    pub workspace_id: String,
    pub event_type: String,
    pub object_key: String,
    pub record_id: String,
    pub payload_json: String,
    pub correlation_id: Option<String>,
}

pub fn insert(conn: &Connection, id: &str, workspace_id: &str, event_type: &str, object_key: &str, record_id: &str, payload_json: &str, correlation_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO integration_pending_events (id, workspace_id, event_type, object_key, record_id, payload_json, correlation_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, workspace_id, event_type, object_key, record_id, payload_json, correlation_id, now_iso()],
    )?;
    Ok(())
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<PendingEvent> {
    Ok(PendingEvent {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        event_type: row.get("event_type")?,
        object_key: row.get("object_key")?,
        record_id: row.get("record_id")?,
        payload_json: row.get("payload_json")?,
        correlation_id: row.get("correlation_id")?,
    })
}

/// Oldest-first, capped - a drain pass processes a bounded batch rather
/// than risking an unbounded backlog in one call.
pub fn list_batch(conn: &Connection, limit: i64) -> rusqlite::Result<Vec<PendingEvent>> {
    let mut stmt = conn.prepare("SELECT * FROM integration_pending_events ORDER BY created_at LIMIT ?1")?;
    let rows = stmt.query_map([limit], map_row)?;
    rows.collect()
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM integration_pending_events WHERE id = ?1", [id])?;
    Ok(())
}
