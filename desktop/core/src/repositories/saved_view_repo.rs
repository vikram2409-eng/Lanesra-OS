//! Raw CRUD for `saved_views` (migration 0034) - see
//! `services::saved_view_service` for visibility rules and the
//! one-default-per-object_key invariant.

use rusqlite::Connection;
use std::collections::HashMap;

use crate::domain::ids::now_iso;
use crate::models::saved_view::SavedView;

fn parse_filters(raw: &str) -> HashMap<String, String> {
    serde_json::from_str(raw).unwrap_or_default()
}

fn parse_columns(raw: Option<String>) -> Option<Vec<String>> {
    raw.and_then(|s| serde_json::from_str(&s).ok())
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<SavedView> {
    let filters_raw: String = row.get("filters")?;
    let columns_raw: Option<String> = row.get("columns")?;
    Ok(SavedView {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        object_key: row.get("object_key")?,
        name: row.get("name")?,
        owner_user_id: row.get("owner_user_id")?,
        owner_name: None,
        visibility: row.get("visibility")?,
        filters: parse_filters(&filters_raw),
        sort_field: row.get("sort_field")?,
        sort_direction: row.get("sort_direction")?,
        columns: parse_columns(columns_raw),
        group_by_field: row.get("group_by_field")?,
        is_object_default: row.get("is_object_default")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    object_key: &str,
    name: &str,
    owner_user_id: &str,
    visibility: &str,
    filters: &HashMap<String, String>,
    sort_field: Option<&str>,
    sort_direction: &str,
    columns: Option<&[String]>,
    group_by_field: Option<&str>,
) -> rusqlite::Result<SavedView> {
    let now = now_iso();
    let filters_json = serde_json::to_string(filters).unwrap_or_else(|_| "{}".to_string());
    let columns_json = columns.map(|c| serde_json::to_string(c).unwrap_or_default());
    conn.execute(
        "INSERT INTO saved_views (id, workspace_id, object_key, name, owner_user_id, visibility, filters, sort_field, sort_direction, columns, group_by_field, is_object_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, ?12, ?12)",
        rusqlite::params![id, workspace_id, object_key, name, owner_user_id, visibility, filters_json, sort_field, sort_direction, columns_json, group_by_field, now],
    )?;
    get_by_id(conn, id).map(|v| v.expect("just inserted"))
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    conn: &Connection,
    id: &str,
    name: &str,
    visibility: &str,
    filters: &HashMap<String, String>,
    sort_field: Option<&str>,
    sort_direction: &str,
    columns: Option<&[String]>,
    group_by_field: Option<&str>,
) -> rusqlite::Result<SavedView> {
    let now = now_iso();
    let filters_json = serde_json::to_string(filters).unwrap_or_else(|_| "{}".to_string());
    let columns_json = columns.map(|c| serde_json::to_string(c).unwrap_or_default());
    conn.execute(
        "UPDATE saved_views SET name = ?2, visibility = ?3, filters = ?4, sort_field = ?5, sort_direction = ?6, columns = ?7, group_by_field = ?8, updated_at = ?9 WHERE id = ?1",
        rusqlite::params![id, name, visibility, filters_json, sort_field, sort_direction, columns_json, group_by_field, now],
    )?;
    get_by_id(conn, id).map(|v| v.expect("just updated"))
}

pub fn get_by_id(conn: &Connection, id: &str) -> rusqlite::Result<Option<SavedView>> {
    conn.query_row("SELECT * FROM saved_views WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// Every view a `user_id` may use for `object_key`: their own (private or
/// shared) plus every other user's shared views - private views owned by
/// someone else never come back here, enforced by this query rather than
/// filtered after the fact.
pub fn list_usable(conn: &Connection, workspace_id: &str, object_key: &str, user_id: &str) -> rusqlite::Result<Vec<SavedView>> {
    let mut stmt = conn.prepare(
        "SELECT sv.*, u.display_name AS owner_name FROM saved_views sv
         LEFT JOIN users u ON u.id = sv.owner_user_id
         WHERE sv.workspace_id = ?1 AND sv.object_key = ?2
           AND (sv.owner_user_id = ?3 OR sv.visibility = 'shared')
         ORDER BY sv.is_object_default DESC, sv.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(rusqlite::params![workspace_id, object_key, user_id], |row| {
        let mut view = map_row(row)?;
        view.owner_name = row.get("owner_name")?;
        Ok(view)
    })?.collect();
    rows
}

pub fn find_default(conn: &Connection, workspace_id: &str, object_key: &str) -> rusqlite::Result<Option<SavedView>> {
    conn.query_row(
        "SELECT * FROM saved_views WHERE workspace_id = ?1 AND object_key = ?2 AND is_object_default = 1",
        (workspace_id, object_key),
        map_row,
    )
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn clear_default_for_object(conn: &Connection, workspace_id: &str, object_key: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE saved_views SET is_object_default = 0 WHERE workspace_id = ?1 AND object_key = ?2 AND is_object_default = 1",
        (workspace_id, object_key),
    )?;
    Ok(())
}

pub fn set_default(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE saved_views SET is_object_default = 1 WHERE id = ?1", [id])?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM saved_views WHERE id = ?1", [id])?;
    Ok(())
}
