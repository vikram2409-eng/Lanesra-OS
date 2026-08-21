//! Raw CRUD for `integration_external_objects` (migration 0032) - see
//! `services::external_object_service`.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::integration::{ExternalObject, FieldMapEntry};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ExternalObject> {
    let field_map_json: String = row.get("field_map_json")?;
    let field_map: Vec<FieldMapEntry> = serde_json::from_str(&field_map_json).unwrap_or_default();
    Ok(ExternalObject {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        object_key: row.get("object_key")?,
        display_name: row.get("display_name")?,
        connection_id: row.get("connection_id")?,
        resource_path: row.get("resource_path")?,
        field_map,
        cache_ttl_seconds: row.get("cache_ttl_seconds")?,
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
    object_key: &str,
    display_name: &str,
    connection_id: &str,
    resource_path: &str,
    field_map_json: &str,
    cache_ttl_seconds: Option<i64>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<ExternalObject> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO integration_external_objects
            (id, workspace_id, object_key, display_name, connection_id, resource_path, field_map_json, cache_ttl_seconds, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?9, ?10)",
        rusqlite::params![id, workspace_id, object_key, display_name, connection_id, resource_path, field_map_json, cache_ttl_seconds, now, actor_user_id],
    )?;
    get(conn, id).map(|o| o.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<ExternalObject>> {
    conn.query_row("SELECT * FROM integration_external_objects WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn get_by_object_key(conn: &Connection, workspace_id: &str, object_key: &str) -> rusqlite::Result<Option<ExternalObject>> {
    conn.query_row("SELECT * FROM integration_external_objects WHERE workspace_id = ?1 AND object_key = ?2", rusqlite::params![workspace_id, object_key], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<ExternalObject>> {
    let mut stmt = conn.prepare("SELECT * FROM integration_external_objects WHERE workspace_id = ?1 ORDER BY display_name")?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM integration_external_objects WHERE id = ?1", [id])?;
    Ok(())
}
