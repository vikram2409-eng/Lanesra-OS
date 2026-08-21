//! Raw CRUD for `integration_connection_refs` (migration 0032).

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::integration::ConnectionRef;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ConnectionRef> {
    Ok(ConnectionRef {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        reference_name: row.get("reference_name")?,
        reference_key: row.get("reference_key")?,
        expected_connection_type: row.get("expected_connection_type")?,
        connection_id: row.get("connection_id")?,
        connection_name: row.get("connection_name")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

const SELECT: &str = "SELECT r.*, c.name AS connection_name FROM integration_connection_refs r LEFT JOIN integration_connections c ON c.id = r.connection_id";

pub fn insert(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    reference_name: &str,
    reference_key: &str,
    expected_connection_type: &str,
    connection_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<ConnectionRef> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO integration_connection_refs (id, workspace_id, reference_name, reference_key, expected_connection_type, connection_id, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?7, ?8)",
        rusqlite::params![id, workspace_id, reference_name, reference_key, expected_connection_type, connection_id, now, actor_user_id],
    )?;
    get(conn, id).map(|r| r.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<ConnectionRef>> {
    conn.query_row(&format!("{SELECT} WHERE r.id = ?1"), [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn get_by_key(conn: &Connection, workspace_id: &str, reference_key: &str) -> rusqlite::Result<Option<ConnectionRef>> {
    conn.query_row(&format!("{SELECT} WHERE r.workspace_id = ?1 AND r.reference_key = ?2"), (workspace_id, reference_key), map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<ConnectionRef>> {
    let mut stmt = conn.prepare(&format!("{SELECT} WHERE r.workspace_id = ?1 ORDER BY r.reference_name"))?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn bind(conn: &Connection, id: &str, connection_id: Option<&str>, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE integration_connection_refs SET connection_id = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4",
        rusqlite::params![connection_id, now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM integration_connection_refs WHERE id = ?1", [id])?;
    Ok(())
}
