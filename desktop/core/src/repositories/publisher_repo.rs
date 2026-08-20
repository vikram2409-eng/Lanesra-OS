//! Raw CRUD for `publishers` (migration 0029) - see
//! `services::publisher_service` for key validation, reserved-keyword
//! rules, and the default-seeding this table depends on.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::publisher::Publisher;

fn map_publisher(row: &rusqlite::Row) -> rusqlite::Result<Publisher> {
    Ok(Publisher {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        key: row.get("key")?,
        name: row.get("name")?,
        description: row.get("description")?,
        is_official: row.get("is_official")?,
        is_local: row.get("is_local")?,
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
    key: &str,
    name: &str,
    description: Option<&str>,
    is_official: bool,
    is_local: bool,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Publisher> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO publishers (id, workspace_id, key, name, description, is_official, is_local, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?8, ?9)",
        rusqlite::params![id, workspace_id, key, name, description, is_official, is_local, now, actor_user_id],
    )?;
    get_by_id(conn, id).map(|p| p.expect("just inserted"))
}

pub fn get_by_id(conn: &Connection, id: &str) -> rusqlite::Result<Option<Publisher>> {
    conn.query_row("SELECT * FROM publishers WHERE id = ?1", [id], map_publisher)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn get_by_key(conn: &Connection, workspace_id: &str, key: &str) -> rusqlite::Result<Option<Publisher>> {
    conn.query_row(
        "SELECT * FROM publishers WHERE workspace_id = ?1 AND key = ?2",
        (workspace_id, key),
        map_publisher,
    )
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// Official and local publishers first, then the rest alphabetically by
/// name - the two auto-seeded publishers are the ones an admin most
/// needs to recognize at a glance.
pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Publisher>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM publishers WHERE workspace_id = ?1 ORDER BY is_official DESC, is_local DESC, name",
    )?;
    let rows = stmt.query_map([workspace_id], map_publisher)?.collect();
    rows
}
