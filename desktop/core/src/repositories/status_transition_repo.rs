use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::status_transition::{StatusTransition, StatusTransitionInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<StatusTransition> {
    Ok(StatusTransition {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        entity_type: row.get("entity_type")?,
        from_status: row.get("from_status")?,
        to_status: row.get("to_status")?,
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn list(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<Vec<StatusTransition>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM status_transitions WHERE workspace_id = ?1 AND entity_type = ?2 ORDER BY created_at",
    )?;
    let rows = stmt.query_map((workspace_id, entity_type), map_row)?.collect();
    rows
}

/// Every active rule for `entity_type`, regardless of workspace - callers
/// always pass a single workspace's `workspace_id` alongside `entity_type`
/// (this app is single-tenant per database), so this is equivalent to
/// `list` filtered to `is_active`, kept separate to make the enforcement
/// call site's intent explicit.
pub fn list_active(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<Vec<StatusTransition>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM status_transitions WHERE workspace_id = ?1 AND entity_type = ?2 AND is_active = 1",
    )?;
    let rows = stmt.query_map((workspace_id, entity_type), map_row)?.collect();
    rows
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<StatusTransition>> {
    conn.query_row("SELECT * FROM status_transitions WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    input: &StatusTransitionInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<StatusTransition> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO status_transitions (id, workspace_id, entity_type, from_status, to_status, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?6, ?7)",
        (id, workspace_id, &input.entity_type, &input.from_status, &input.to_status, &now, &actor_user_id),
    )?;
    Ok(get(conn, id)?.expect("just inserted"))
}

pub fn set_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE status_transitions SET is_active = ?2, updated_at = ?3, updated_by = ?4 WHERE id = ?1",
        (id, is_active, now_iso(), actor_user_id),
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM status_transitions WHERE id = ?1", [id])?;
    Ok(())
}
