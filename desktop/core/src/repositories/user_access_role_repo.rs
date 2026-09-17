use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::access_role::AccessRole;

fn map_role(row: &rusqlite::Row) -> rusqlite::Result<AccessRole> {
    Ok(AccessRole {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        is_system: row.get("is_system")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// Every Access Role this user holds - `access_service::capability_grant_for`
/// folds all of them together (broadest scope wins) rather than picking one.
pub fn list_for_user(conn: &Connection, user_id: &str) -> rusqlite::Result<Vec<AccessRole>> {
    let mut stmt = conn.prepare(
        "SELECT ar.* FROM access_roles ar
         JOIN user_access_roles uar ON uar.access_role_id = ar.id
         WHERE uar.user_id = ?1
         ORDER BY ar.name",
    )?;
    let rows = stmt.query_map([user_id], map_role)?.collect();
    rows
}

pub fn list_assignee_user_ids(conn: &Connection, access_role_id: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT user_id FROM user_access_roles WHERE access_role_id = ?1")?;
    let rows = stmt.query_map([access_role_id], |r| r.get(0))?.collect();
    rows
}

pub fn count_assignees(conn: &Connection, access_role_id: &str) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM user_access_roles WHERE access_role_id = ?1", [access_role_id], |r| r.get(0))
}

pub fn assign(conn: &Connection, user_id: &str, access_role_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO user_access_roles (user_id, access_role_id, created_at) VALUES (?1, ?2, ?3)",
        (user_id, access_role_id, now_iso()),
    )?;
    Ok(())
}

pub fn remove(conn: &Connection, user_id: &str, access_role_id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM user_access_roles WHERE user_id = ?1 AND access_role_id = ?2", (user_id, access_role_id))?;
    Ok(())
}
