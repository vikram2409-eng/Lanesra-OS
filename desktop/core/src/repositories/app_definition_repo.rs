use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::app_definition::{AppDefinition, AppPermission};

/// Deserializes `object_keys_json` - the service layer owns the
/// `Vec<String>` shape, this stays JSON-blind beyond routing the raw text
/// in and out (same reasoning as `dashboard_layout_repo::map_row`).
fn map_row(row: &rusqlite::Row) -> rusqlite::Result<(AppDefinition, String)> {
    let app = AppDefinition {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        icon: row.get("icon")?,
        description: row.get("description")?,
        object_keys: Vec::new(), // filled in by the caller from object_keys_json
        dashboard_id: row.get("dashboard_id")?,
        is_published: row.get("is_published")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    };
    let object_keys_json: String = row.get("object_keys_json")?;
    Ok((app, object_keys_json))
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<(AppDefinition, String)>> {
    let mut stmt = conn.prepare("SELECT * FROM app_definitions WHERE workspace_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<(AppDefinition, String)>> {
    conn.query_row("SELECT * FROM app_definitions WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    name: &str,
    icon: &str,
    description: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO app_definitions (id, workspace_id, name, icon, description, object_keys_json, dashboard_id, is_published, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, '[]', NULL, 0, ?6, ?7, ?6, ?7)",
        rusqlite::params![id, workspace_id, name, icon, description, now, actor_user_id],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    conn: &Connection,
    id: &str,
    name: &str,
    icon: &str,
    description: Option<&str>,
    object_keys_json: &str,
    dashboard_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE app_definitions SET name = ?1, icon = ?2, description = ?3, object_keys_json = ?4, dashboard_id = ?5, updated_at = ?6, updated_by = ?7 WHERE id = ?8",
        rusqlite::params![name, icon, description, object_keys_json, dashboard_id, now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn set_published(conn: &Connection, id: &str, published: bool, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE app_definitions SET is_published = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4",
        rusqlite::params![published, now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM app_definitions WHERE id = ?1", [id])?;
    Ok(())
}

pub fn new_id() -> String {
    new_uuid()
}

// ---- Permissions ----------------------------------------------------------

fn map_permission_row(row: &rusqlite::Row) -> rusqlite::Result<AppPermission> {
    Ok(AppPermission {
        id: row.get("id")?,
        app_id: row.get("app_id")?,
        principal_type: row.get("principal_type")?,
        principal_id: row.get("principal_id")?,
        level: row.get("level")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
    })
}

pub fn list_permissions(conn: &Connection, app_id: &str) -> rusqlite::Result<Vec<AppPermission>> {
    let mut stmt = conn.prepare("SELECT * FROM app_permissions WHERE app_id = ?1 ORDER BY principal_type, principal_id")?;
    let rows = stmt.query_map([app_id], map_permission_row)?.collect();
    rows
}

/// Every permission grant across every app in the workspace, in one query -
/// `app_service::list_accessible` needs this shape (grouped by app_id
/// after the fact) rather than N+1 `list_permissions` calls per app.
pub fn list_all_permissions_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<AppPermission>> {
    let mut stmt = conn.prepare(
        "SELECT ap.* FROM app_permissions ap
         JOIN app_definitions ad ON ad.id = ap.app_id
         WHERE ad.workspace_id = ?1",
    )?;
    let rows = stmt.query_map([workspace_id], map_permission_row)?.collect();
    rows
}

/// Grants (or re-grants, at a possibly different level) one principal
/// access to an app - the unique index on (app_id, principal_type,
/// principal_id) makes this an upsert rather than a plain insert, so
/// re-granting the same role/user updates the level in place instead of
/// piling up rows a resolver would have to pick the best of.
pub fn upsert_permission(
    conn: &Connection,
    id: &str,
    app_id: &str,
    principal_type: &str,
    principal_id: &str,
    level: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO app_permissions (id, app_id, principal_type, principal_id, level, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(app_id, principal_type, principal_id) DO UPDATE SET level = excluded.level",
        rusqlite::params![id, app_id, principal_type, principal_id, level, now_iso(), actor_user_id],
    )?;
    Ok(())
}

pub fn delete_permission(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM app_permissions WHERE id = ?1", [id])?;
    Ok(())
}
