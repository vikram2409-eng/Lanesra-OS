//! Raw CRUD for `integration_mappings` (migration 0032) - reusable field
//! mappings CSV import/export (and later, Integration Jobs) bind to. See
//! `services::mapping_service`.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::integration::{FieldMapEntry, Mapping};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Mapping> {
    let field_map_json: String = row.get("field_map_json")?;
    let field_map: Vec<FieldMapEntry> = serde_json::from_str(&field_map_json).unwrap_or_default();
    let needs_review: i64 = row.get("needs_review")?;
    Ok(Mapping {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        target_object_key: row.get("target_object_key")?,
        operation: row.get("operation")?,
        match_key: row.get("match_key")?,
        field_map,
        duplicate_policy: row.get("duplicate_policy")?,
        needs_review: needs_review != 0,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    name: &str,
    target_object_key: &str,
    operation: &str,
    match_key: Option<&str>,
    field_map_json: &str,
    duplicate_policy: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Mapping> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO integration_mappings
            (id, workspace_id, name, target_object_key, operation, match_key, field_map_json, duplicate_policy, needs_review, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, ?10, ?9, ?10)",
        rusqlite::params![id, workspace_id, name, target_object_key, operation, match_key, field_map_json, duplicate_policy, now, actor_user_id],
    )?;
    get(conn, id).map(|m| m.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Mapping>> {
    conn.query_row("SELECT * FROM integration_mappings WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Mapping>> {
    let mut stmt = conn.prepare("SELECT * FROM integration_mappings WHERE workspace_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM integration_mappings WHERE id = ?1", [id])?;
    Ok(())
}
