use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::org_unit::{OrgUnit, OrgUnitInput, OrgUnitUpdate};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<OrgUnit> {
    Ok(OrgUnit {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        unit_type: row.get("unit_type")?,
        parent_org_unit_id: row.get("parent_org_unit_id")?,
        manager_user_id: row.get("manager_user_id")?,
        status: row.get("status")?,
        effective_from: row.get("effective_from")?,
        effective_to: row.get("effective_to")?,
        default_team_id: row.get("default_team_id")?,
        path: row.get("path")?,
        depth: row.get("depth")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    input: &OrgUnitInput,
    path: &str,
    depth: i64,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<OrgUnit> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO org_units
            (id, workspace_id, name, unit_type, parent_org_unit_id, manager_user_id, status, effective_from, effective_to, default_team_id, path, depth, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'Active', ?7, ?8, NULL, ?9, ?10, ?11, ?12, ?11, ?12)",
        rusqlite::params![
            id, workspace_id, input.name, input.unit_type, input.parent_org_unit_id, input.manager_user_id,
            input.effective_from, input.effective_to, path, depth, now, actor_user_id,
        ],
    )?;
    get(conn, id).map(|d| d.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<OrgUnit>> {
    conn.query_row("SELECT * FROM org_units WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// Every unit in the workspace, ordered by path - a parent's row always
/// sorts before its own descendants (materialized path prefix ordering),
/// so the frontend can build a tree straight from this flat list without a
/// second pass.
pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<OrgUnit>> {
    let mut stmt = conn.prepare("SELECT * FROM org_units WHERE workspace_id = ?1 ORDER BY path")?;
    let rows = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}

pub fn list_children(conn: &Connection, parent_org_unit_id: &str) -> rusqlite::Result<Vec<OrgUnit>> {
    let mut stmt = conn.prepare("SELECT * FROM org_units WHERE parent_org_unit_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map([parent_org_unit_id], map_row)?.collect();
    rows
}

/// Every unit whose path starts with `path_prefix` (the unit's own subtree,
/// including itself) - the `LIKE` prefix scan the materialized-path design
/// exists for. `path_prefix` must already end in '/' and be escaped by the
/// caller if it can contain '%'/'_' (org unit ids are opaque hex/UUID
/// strings, never user-supplied text, so no escaping is needed here).
pub fn list_descendants_by_path(conn: &Connection, workspace_id: &str, path_prefix: &str) -> rusqlite::Result<Vec<OrgUnit>> {
    let mut stmt = conn.prepare("SELECT * FROM org_units WHERE workspace_id = ?1 AND path LIKE ?2 ORDER BY path")?;
    let pattern = format!("{path_prefix}%");
    let rows = stmt.query_map((workspace_id, pattern), map_row)?.collect();
    rows
}

pub fn update(conn: &Connection, id: &str, input: &OrgUnitUpdate, actor_user_id: Option<&str>) -> rusqlite::Result<OrgUnit> {
    conn.execute(
        "UPDATE org_units SET name = ?1, unit_type = ?2, manager_user_id = ?3, status = ?4, effective_from = ?5, effective_to = ?6, updated_at = ?7, updated_by = ?8
         WHERE id = ?9",
        rusqlite::params![
            input.name, input.unit_type, input.manager_user_id, input.status, input.effective_from, input.effective_to,
            now_iso(), actor_user_id, id,
        ],
    )?;
    get(conn, id).map(|d| d.expect("just updated"))
}

/// Moves `id` under `new_parent_id` (None = becomes a root unit) and
/// rewrites its own path/depth - the caller (org_unit_service::move_unit)
/// is responsible for then calling update_path_for_subtree to keep every
/// descendant's own path/depth consistent.
pub fn set_parent_and_path(
    conn: &Connection,
    id: &str,
    new_parent_id: Option<&str>,
    new_path: &str,
    new_depth: i64,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE org_units SET parent_org_unit_id = ?1, path = ?2, depth = ?3, updated_at = ?4, updated_by = ?5 WHERE id = ?6",
        (new_parent_id, new_path, new_depth, now_iso(), actor_user_id, id),
    )?;
    Ok(())
}

/// Rewrites every descendant's path (swap the `old_prefix` head for
/// `new_prefix`) and depth (shift by `depth_delta`) in one bounded
/// statement - the whole point of a materialized path over a closure
/// table: a move only ever touches the moved subtree's own rows, once.
pub fn update_path_for_subtree(conn: &Connection, old_prefix: &str, new_prefix: &str, depth_delta: i64, actor_user_id: Option<&str>) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE org_units
         SET path = ?1 || substr(path, ?2), depth = depth + ?3, updated_at = ?4, updated_by = ?5
         WHERE path LIKE ?6",
        rusqlite::params![
            new_prefix,
            (old_prefix.len() + 1) as i64,
            depth_delta,
            now_iso(),
            actor_user_id,
            format!("{old_prefix}%"),
        ],
    )
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM org_units WHERE id = ?1", [id])?;
    Ok(())
}
