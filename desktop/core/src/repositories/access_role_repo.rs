use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::access_role::{AccessRole, AccessRoleGrant, AccessRoleGrantInput, AccessRoleInput, AccessRoleUpdate, RecordScope};

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

fn map_grant(row: &rusqlite::Row) -> rusqlite::Result<AccessRoleGrant> {
    let scope_str: String = row.get("record_scope")?;
    Ok(AccessRoleGrant {
        id: row.get("id")?,
        access_role_id: row.get("access_role_id")?,
        object_key: row.get("object_key")?,
        can_create: row.get("can_create")?,
        can_read: row.get("can_read")?,
        can_update: row.get("can_update")?,
        can_delete: row.get("can_delete")?,
        can_assign: row.get("can_assign")?,
        record_scope: RecordScope::from_str(&scope_str).unwrap_or(RecordScope::Owner),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &AccessRoleInput, is_system: bool, actor_user_id: Option<&str>) -> rusqlite::Result<AccessRole> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO access_roles (id, workspace_id, name, description, is_system, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?6, ?7)",
        rusqlite::params![id, workspace_id, input.name, input.description, is_system, now, actor_user_id],
    )?;
    get(conn, id).map(|d| d.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<AccessRole>> {
    conn.query_row("SELECT * FROM access_roles WHERE id = ?1", [id], map_role)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<AccessRole>> {
    let mut stmt = conn.prepare("SELECT * FROM access_roles WHERE workspace_id = ?1 ORDER BY is_system DESC, name")?;
    let rows = stmt.query_map([workspace_id], map_role)?.collect();
    rows
}

/// Looks up one of the two system roles ("Full Access"/"Standard User") by
/// name - used to assign a starting Access Role to a newly created user,
/// the same bridge the 0056 migration's own backfill applies to users that
/// already existed when it ran.
pub fn get_by_name(conn: &Connection, workspace_id: &str, name: &str) -> rusqlite::Result<Option<AccessRole>> {
    conn.query_row("SELECT * FROM access_roles WHERE workspace_id = ?1 AND name = ?2", (workspace_id, name), map_role)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn update(conn: &Connection, id: &str, input: &AccessRoleUpdate, actor_user_id: Option<&str>) -> rusqlite::Result<AccessRole> {
    conn.execute(
        "UPDATE access_roles SET name = ?1, description = ?2, updated_at = ?3, updated_by = ?4 WHERE id = ?5",
        rusqlite::params![input.name, input.description, now_iso(), actor_user_id, id],
    )?;
    get(conn, id).map(|d| d.expect("just updated"))
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM access_role_grants WHERE access_role_id = ?1", [id])?;
    conn.execute("DELETE FROM access_roles WHERE id = ?1", [id])?;
    Ok(())
}

/// Every grant row this role carries - one per object_key it customizes,
/// plus a `'*'` row if it has a default. `access_service::capability_grant_for`
/// is the one place that resolves "the right row for this object_key,
/// falling back to `'*'`" - this just returns everything, unfiltered.
pub fn list_grants(conn: &Connection, access_role_id: &str) -> rusqlite::Result<Vec<AccessRoleGrant>> {
    let mut stmt = conn.prepare("SELECT * FROM access_role_grants WHERE access_role_id = ?1 ORDER BY (object_key = '*'), object_key")?;
    let rows = stmt.query_map([access_role_id], map_grant)?.collect();
    rows
}

pub fn get_grant_for_object_key(conn: &Connection, access_role_id: &str, object_key: &str) -> rusqlite::Result<Option<AccessRoleGrant>> {
    conn.query_row(
        "SELECT * FROM access_role_grants WHERE access_role_id = ?1 AND object_key = ?2",
        (access_role_id, object_key),
        map_grant,
    )
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// Insert-or-replace a role's grant for one object_key (including `'*'`) -
/// the admin UI's per-row "save" action. Keeps the existing row's `id` on
/// an update (only the touched columns change) rather than generating a new
/// one, so nothing else referencing this grant's id is disturbed.
pub fn upsert_grant(conn: &Connection, id: &str, access_role_id: &str, input: &AccessRoleGrantInput) -> rusqlite::Result<AccessRoleGrant> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO access_role_grants (id, access_role_id, object_key, can_create, can_read, can_update, can_delete, can_assign, record_scope, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)
         ON CONFLICT(access_role_id, object_key) DO UPDATE SET
            can_create = excluded.can_create, can_read = excluded.can_read, can_update = excluded.can_update,
            can_delete = excluded.can_delete, can_assign = excluded.can_assign, record_scope = excluded.record_scope,
            updated_at = excluded.updated_at",
        rusqlite::params![
            id, access_role_id, input.object_key, input.can_create, input.can_read, input.can_update,
            input.can_delete, input.can_assign, input.record_scope, now,
        ],
    )?;
    get_grant_for_object_key(conn, access_role_id, &input.object_key).map(|d| d.expect("just upserted"))
}

pub fn delete_grant(conn: &Connection, access_role_id: &str, object_key: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM access_role_grants WHERE access_role_id = ?1 AND object_key = ?2", (access_role_id, object_key))?;
    Ok(())
}
