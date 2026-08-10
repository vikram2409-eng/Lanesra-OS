use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::custom_object::{CustomObjectDefinition, CustomObjectDefinitionInput, CustomObjectDefinitionUpdate};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<CustomObjectDefinition> {
    Ok(CustomObjectDefinition {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        key: row.get("key")?,
        singular_label: row.get("singular_label")?,
        plural_label: row.get("plural_label")?,
        icon: row.get("icon")?,
        prefix: row.get("prefix")?,
        digits: row.get("digits")?,
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    key: &str,
    input: &CustomObjectDefinitionInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<CustomObjectDefinition> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO custom_object_definitions
            (id, workspace_id, key, singular_label, plural_label, icon, prefix, digits, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?10, ?9, ?10)",
        rusqlite::params![
            id, workspace_id, key, input.singular_label, input.plural_label, input.icon,
            input.prefix, input.digits, now, actor_user_id,
        ],
    )?;
    get(conn, id).map(|d| d.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<CustomObjectDefinition>> {
    conn.query_row("SELECT * FROM custom_object_definitions WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn get_by_key(conn: &Connection, workspace_id: &str, key: &str) -> rusqlite::Result<Option<CustomObjectDefinition>> {
    conn.query_row(
        "SELECT * FROM custom_object_definitions WHERE workspace_id = ?1 AND key = ?2",
        (workspace_id, key),
        map_row,
    )
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// All definitions for the workspace, active and inactive - the service
/// layer decides which to expose to which caller (nav/creation only wants
/// active ones; the admin screen wants everything).
pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<CustomObjectDefinition>> {
    let mut stmt = conn.prepare("SELECT * FROM custom_object_definitions WHERE workspace_id = ?1 ORDER BY plural_label")?;
    let rows = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}

pub fn update(conn: &Connection, id: &str, input: &CustomObjectDefinitionUpdate, actor_user_id: Option<&str>) -> rusqlite::Result<CustomObjectDefinition> {
    conn.execute(
        "UPDATE custom_object_definitions
         SET singular_label = ?1, plural_label = ?2, icon = ?3, prefix = ?4, digits = ?5, is_active = ?6, updated_at = ?7, updated_by = ?8
         WHERE id = ?9",
        rusqlite::params![
            input.singular_label, input.plural_label, input.icon, input.prefix, input.digits,
            input.is_active, now_iso(), actor_user_id, id,
        ],
    )?;
    get(conn, id).map(|d| d.expect("just updated"))
}

/// Hard-deletes a definition. Callers must first verify there are no
/// `custom_records` for this key - the service layer enforces that (the
/// same "block delete when dependents exist" rule every other entity in
/// the product follows), not a SQL foreign key, since custom_records.
/// object_key is joined by value rather than a formal FK.
pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM custom_object_definitions WHERE id = ?1", [id])?;
    Ok(())
}
