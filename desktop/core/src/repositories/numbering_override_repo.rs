use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::numbering_override::NumberingOverride;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<NumberingOverride> {
    Ok(NumberingOverride {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        entity_type: row.get("entity_type")?,
        prefix: row.get("prefix")?,
        digits: row.get("digits")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn get_for_entity(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<Option<NumberingOverride>> {
    conn.query_row(
        "SELECT * FROM numbering_configs WHERE workspace_id = ?1 AND entity_type = ?2",
        (workspace_id, entity_type),
        map_row,
    )
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// On a fresh insert, `actor_user_id` becomes both `created_by` and
/// `updated_by`. On conflict (an override already exists for this entity
/// type), only `updated_by` changes - `created_by` stays whoever set the
/// override up originally, standard upsert audit semantics.
pub fn upsert(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
    prefix: &str,
    digits: i64,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<NumberingOverride> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO numbering_configs (id, workspace_id, entity_type, prefix, digits, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?6, ?7)
         ON CONFLICT (workspace_id, entity_type)
         DO UPDATE SET prefix = ?4, digits = ?5, updated_at = ?6, updated_by = ?7",
        (new_uuid(), workspace_id, entity_type, prefix, digits, now, actor_user_id),
    )?;
    get_for_entity(conn, workspace_id, entity_type).map(|r| r.expect("just upserted"))
}

pub fn delete(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM numbering_configs WHERE workspace_id = ?1 AND entity_type = ?2",
        (workspace_id, entity_type),
    )?;
    Ok(())
}
