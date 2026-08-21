//! Raw CRUD for `integration_pending_actions` (migration 0033) - the
//! queue `workflow_service`'s "call_connector_action" action writes into;
//! `services::connector_execution_service::drain_pending_actions` is the
//! async drain that actually performs each call. See that module's own
//! doc comment for why this exists instead of an inline async call.

use rusqlite::Connection;

use crate::domain::ids::now_iso;

pub struct PendingAction {
    pub id: String,
    pub workspace_id: String,
    pub connector_id: String,
    pub action_key: String,
    pub reference_key: String,
    pub params_json: String,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<PendingAction> {
    Ok(PendingAction {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        connector_id: row.get("connector_id")?,
        action_key: row.get("action_key")?,
        reference_key: row.get("reference_key")?,
        params_json: row.get("params_json")?,
        source_entity_type: row.get("source_entity_type")?,
        source_entity_id: row.get("source_entity_id")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn enqueue(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    connector_id: &str,
    action_key: &str,
    reference_key: &str,
    params_json: &str,
    source_entity_type: Option<&str>,
    source_entity_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO integration_pending_actions (id, workspace_id, connector_id, action_key, reference_key, params_json, source_entity_type, source_entity_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, workspace_id, connector_id, action_key, reference_key, params_json, source_entity_type, source_entity_id, now_iso()],
    )?;
    Ok(())
}

pub fn list_batch(conn: &Connection, workspace_id: &str, limit: i64) -> rusqlite::Result<Vec<PendingAction>> {
    let mut stmt = conn.prepare("SELECT * FROM integration_pending_actions WHERE workspace_id = ?1 ORDER BY created_at LIMIT ?2")?;
    let rows = stmt.query_map(rusqlite::params![workspace_id, limit], map_row)?;
    rows.collect()
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM integration_pending_actions WHERE id = ?1", [id])?;
    Ok(())
}
