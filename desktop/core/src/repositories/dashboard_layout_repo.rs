use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::dashboard_layout::DashboardLayout;

/// Deserializes `roles_json`/`draft_json`/`published_json` - the service
/// layer owns `DashboardWidgets`/`Vec<String>` shapes, this stays
/// JSON-blind beyond routing the raw text in and out (see
/// `screen_layout_repo::map_row`'s identical reasoning).
fn map_row(row: &rusqlite::Row) -> rusqlite::Result<(DashboardLayout, String, String, Option<String>)> {
    let published_json: Option<String> = row.get("published_json")?;
    let layout = DashboardLayout {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        is_default: row.get("is_default")?,
        roles: Vec::new(),         // filled in by the caller from roles_json
        draft: Default::default(), // filled in by the caller from draft_json
        published: None,           // filled in by the caller from published_json
        app_id: row.get("app_id")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    };
    let roles_json: String = row.get("roles_json")?;
    let draft_json: String = row.get("draft_json")?;
    Ok((layout, roles_json, draft_json, published_json))
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<(DashboardLayout, String, String, Option<String>)>> {
    let mut stmt = conn.prepare("SELECT * FROM dashboard_layouts WHERE workspace_id = ?1 ORDER BY is_default DESC, name")?;
    let rows = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<(DashboardLayout, String, String, Option<String>)>> {
    conn.query_row("SELECT * FROM dashboard_layouts WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    name: &str,
    is_default: bool,
    roles_json: &str,
    draft_json: &str,
    app_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO dashboard_layouts (id, workspace_id, name, is_default, roles_json, draft_json, published_json, app_id, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8, ?9, ?8, ?9)",
        rusqlite::params![id, workspace_id, name, is_default, roles_json, draft_json, app_id, now, actor_user_id],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn update_meta_and_draft(
    conn: &Connection,
    id: &str,
    name: &str,
    roles_json: &str,
    draft_json: &str,
    app_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE dashboard_layouts SET name = ?1, roles_json = ?2, draft_json = ?3, app_id = ?4, updated_at = ?5, updated_by = ?6 WHERE id = ?7",
        rusqlite::params![name, roles_json, draft_json, app_id, now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn publish(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE dashboard_layouts SET published_json = draft_json, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        rusqlite::params![now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn unpublish(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE dashboard_layouts SET published_json = NULL, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        rusqlite::params![now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn revert_draft_to_published(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE dashboard_layouts SET draft_json = published_json, updated_at = ?1, updated_by = ?2 WHERE id = ?3 AND published_json IS NOT NULL",
        rusqlite::params![now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

/// Clears `is_default` on every OTHER layout for this workspace before the
/// caller sets it on the new one - two sequential UPDATEs so the partial
/// unique index (`idx_dashboard_layouts_one_default`) never sees two rows
/// with `is_default = 1` at once, same reasoning as
/// `screen_layout_repo::clear_default`.
pub fn clear_default(conn: &Connection, workspace_id: &str, except_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE dashboard_layouts SET is_default = 0 WHERE workspace_id = ?1 AND id != ?2 AND is_default = 1",
        rusqlite::params![workspace_id, except_id],
    )?;
    Ok(())
}

pub fn set_default(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE dashboard_layouts SET is_default = 1, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        rusqlite::params![now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM dashboard_layouts WHERE id = ?1", [id])?;
    Ok(())
}

pub fn count_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM dashboard_layouts WHERE workspace_id = ?1", [workspace_id], |row| row.get(0))
}

/// A fresh id, reserved so the service layer can build the first widget
/// ids consistently before the row exists.
pub fn new_id() -> String {
    new_uuid()
}
