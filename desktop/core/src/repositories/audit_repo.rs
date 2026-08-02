use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::audit::AuditEvent;

#[allow(clippy::too_many_arguments)]
pub fn record(
    conn: &Connection,
    workspace_id: &str,
    user_id: Option<&str>,
    event_type: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    summary: &str,
    details_json: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO audit_events (id, workspace_id, occurred_at, user_id, event_type, entity_type, entity_id, summary, details_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        (
            new_uuid(),
            workspace_id,
            now_iso(),
            user_id,
            event_type,
            entity_type,
            entity_id,
            summary,
            details_json,
        ),
    )?;
    Ok(())
}

pub fn list_for_entity(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> rusqlite::Result<Vec<AuditEvent>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM audit_events WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY occurred_at DESC",
    )?;
    let rows = stmt.query_map((entity_type, entity_id), |row| {
        Ok(AuditEvent {
            id: row.get("id")?,
            workspace_id: row.get("workspace_id")?,
            occurred_at: row.get("occurred_at")?,
            user_id: row.get("user_id")?,
            event_type: row.get("event_type")?,
            entity_type: row.get("entity_type")?,
            entity_id: row.get("entity_id")?,
            summary: row.get("summary")?,
            details_json: row.get("details_json")?,
        })
    })?;
    rows.collect()
}
