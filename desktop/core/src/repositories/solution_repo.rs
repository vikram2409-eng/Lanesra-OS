//! Raw CRUD for `solutions` and `solution_members` (migration 0031) - see
//! `services::solution_service` for validation, admin-gating, and the
//! export logic these back.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::solution::{Solution, SolutionMember};

fn map_solution(row: &rusqlite::Row) -> rusqlite::Result<Solution> {
    Ok(Solution {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        version: row.get("version")?,
        publisher_id: row.get("publisher_id")?,
        publisher_name: row.get("publisher_name")?,
        member_count: row.get("member_count")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

/// Every `Solution` read goes through this join so `publisher_name` and
/// `member_count` are always populated the same way, whether it's a
/// single-row `get` or the workspace-wide `list`.
const SELECT_SOLUTION: &str = "SELECT s.*, p.name AS publisher_name,
        (SELECT COUNT(*) FROM solution_members m WHERE m.solution_id = s.id) AS member_count
     FROM solutions s
     LEFT JOIN publishers p ON p.id = s.publisher_id";

#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    name: &str,
    description: Option<&str>,
    version: &str,
    publisher_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Solution> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO solutions (id, workspace_id, name, description, version, publisher_id, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?7, ?8)",
        rusqlite::params![id, workspace_id, name, description, version, publisher_id, now, actor_user_id],
    )?;
    get(conn, id).map(|s| s.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Solution>> {
    conn.query_row(&format!("{SELECT_SOLUTION} WHERE s.id = ?1"), [id], map_solution)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn get_by_name(conn: &Connection, workspace_id: &str, name: &str) -> rusqlite::Result<Option<Solution>> {
    conn.query_row(&format!("{SELECT_SOLUTION} WHERE s.workspace_id = ?1 AND s.name = ?2"), (workspace_id, name), map_solution)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Solution>> {
    let mut stmt = conn.prepare(&format!("{SELECT_SOLUTION} WHERE s.workspace_id = ?1 ORDER BY s.name"))?;
    let rows = stmt.query_map([workspace_id], map_solution)?;
    rows.collect()
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    conn: &Connection,
    id: &str,
    name: &str,
    description: Option<&str>,
    version: &str,
    publisher_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Solution> {
    let now = now_iso();
    conn.execute(
        "UPDATE solutions SET name = ?1, description = ?2, version = ?3, publisher_id = ?4, updated_at = ?5, updated_by = ?6 WHERE id = ?7",
        rusqlite::params![name, description, version, publisher_id, now, actor_user_id, id],
    )?;
    get(conn, id).map(|s| s.expect("just updated"))
}

/// `solution_members` cascades (migration 0031's `ON DELETE CASCADE`,
/// active because every real connection runs with
/// `PRAGMA foreign_keys = ON` - see `db::connection`) - nothing else to
/// clean up here.
pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM solutions WHERE id = ?1", [id])?;
    Ok(())
}

fn map_member(row: &rusqlite::Row) -> rusqlite::Result<SolutionMember> {
    Ok(SolutionMember {
        id: row.get("id")?,
        solution_id: row.get("solution_id")?,
        artifact_type: row.get("artifact_type")?,
        metadata_id: row.get("metadata_id")?,
        added_at: row.get("added_at")?,
        added_by: row.get("added_by")?,
    })
}

/// Idempotent add: re-adding an already-curated component is a silent
/// no-op rather than a conflict, the same forgiving convention
/// `solution_component_repo::upsert` uses for re-tagging.
pub fn add_member(
    conn: &Connection,
    id: &str,
    solution_id: &str,
    artifact_type: &str,
    metadata_id: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO solution_members (id, solution_id, artifact_type, metadata_id, added_at, added_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(solution_id, artifact_type, metadata_id) DO NOTHING",
        rusqlite::params![id, solution_id, artifact_type, metadata_id, now, actor_user_id],
    )?;
    Ok(())
}

/// Removing a component from a Solution's curated list only ever touches
/// this membership row - the component itself, and its
/// `solution_components` ownership tag, are untouched (matches every
/// other "remove from view, don't destroy" convention in this module).
pub fn remove_member(conn: &Connection, solution_id: &str, artifact_type: &str, metadata_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM solution_members WHERE solution_id = ?1 AND artifact_type = ?2 AND metadata_id = ?3",
        (solution_id, artifact_type, metadata_id),
    )?;
    Ok(())
}

pub fn list_members(conn: &Connection, solution_id: &str) -> rusqlite::Result<Vec<SolutionMember>> {
    let mut stmt = conn.prepare("SELECT * FROM solution_members WHERE solution_id = ?1 ORDER BY artifact_type, added_at")?;
    let rows = stmt.query_map([solution_id], map_member)?;
    rows.collect()
}
