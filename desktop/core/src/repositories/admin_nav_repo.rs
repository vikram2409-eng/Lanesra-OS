use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::admin_nav::AdminNavItem;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<AdminNavItem> {
    Ok(AdminNavItem {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        user_id: row.get("user_id")?,
        admin_tab: row.get("admin_tab")?,
        label: row.get("label")?,
        is_pinned: row.get::<_, i64>("is_pinned")? != 0,
        last_viewed_at: row.get("last_viewed_at")?,
    })
}

pub fn record_visit(conn: &Connection, workspace_id: &str, user_id: &str, admin_tab: &str, label: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO admin_nav_history (id, workspace_id, user_id, admin_tab, label, is_pinned, last_viewed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)
         ON CONFLICT(workspace_id, user_id, admin_tab) DO UPDATE SET label = excluded.label, last_viewed_at = excluded.last_viewed_at",
        rusqlite::params![new_uuid(), workspace_id, user_id, admin_tab, label, now_iso()],
    )?;
    Ok(())
}

pub fn set_pinned(conn: &Connection, workspace_id: &str, user_id: &str, admin_tab: &str, pinned: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE admin_nav_history SET is_pinned = ?1 WHERE workspace_id = ?2 AND user_id = ?3 AND admin_tab = ?4",
        rusqlite::params![pinned as i64, workspace_id, user_id, admin_tab],
    )?;
    Ok(())
}

pub fn list_recent(conn: &Connection, workspace_id: &str, user_id: &str, limit: i64) -> rusqlite::Result<Vec<AdminNavItem>> {
    let mut stmt = conn.prepare("SELECT * FROM admin_nav_history WHERE workspace_id = ?1 AND user_id = ?2 AND is_pinned = 0 ORDER BY last_viewed_at DESC LIMIT ?3")?;
    let rows = stmt.query_map(rusqlite::params![workspace_id, user_id, limit], map_row)?;
    rows.collect()
}

pub fn list_pinned(conn: &Connection, workspace_id: &str, user_id: &str) -> rusqlite::Result<Vec<AdminNavItem>> {
    let mut stmt = conn.prepare("SELECT * FROM admin_nav_history WHERE workspace_id = ?1 AND user_id = ?2 AND is_pinned = 1 ORDER BY last_viewed_at DESC")?;
    let rows = stmt.query_map(rusqlite::params![workspace_id, user_id], map_row)?;
    rows.collect()
}
