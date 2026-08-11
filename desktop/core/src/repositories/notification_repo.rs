use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::workflow::Notification;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Notification> {
    Ok(Notification {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        recipient_user_id: row.get("recipient_user_id")?,
        message: row.get("message")?,
        entity_type: row.get("entity_type")?,
        entity_id: row.get("entity_id")?,
        created_at: row.get("created_at")?,
        read_at: row.get("read_at")?,
    })
}

pub fn create(conn: &Connection, workspace_id: &str, recipient_user_id: Option<&str>, message: &str, entity_type: Option<&str>, entity_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO notifications (id, workspace_id, recipient_user_id, message, entity_type, entity_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![new_uuid(), workspace_id, recipient_user_id, message, entity_type, entity_id, now_iso()],
    )?;
    Ok(())
}

/// Notifications addressed to `user_id` directly, or broadcast to
/// everyone (recipient_user_id IS NULL) - the latter is only ever written
/// by the `all_admins` audience today, but is deliberately not filtered by
/// role here so a future broadcast to non-admins doesn't need a schema
/// change.
pub fn list_for_user(conn: &Connection, workspace_id: &str, user_id: &str, unread_only: bool) -> rusqlite::Result<Vec<Notification>> {
    let sql = format!(
        "SELECT * FROM notifications WHERE workspace_id = ?1 AND (recipient_user_id = ?2 OR recipient_user_id IS NULL){}
         ORDER BY created_at DESC LIMIT 100",
        if unread_only { " AND read_at IS NULL" } else { "" }
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map((workspace_id, user_id), map_row)?.collect();
    rows
}

pub fn mark_read(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE notifications SET read_at = ?1 WHERE id = ?2 AND read_at IS NULL", (now_iso(), id))?;
    Ok(())
}

pub fn mark_all_read(conn: &Connection, workspace_id: &str, user_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE notifications SET read_at = ?1 WHERE workspace_id = ?2 AND (recipient_user_id = ?3 OR recipient_user_id IS NULL) AND read_at IS NULL",
        (now_iso(), workspace_id, user_id),
    )?;
    Ok(())
}
