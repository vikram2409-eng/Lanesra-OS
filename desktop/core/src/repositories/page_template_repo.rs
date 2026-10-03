use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::page_template::PageTemplate;

/// Deserializes `definition_json` - same JSON-blind convention as
/// `page_layout_repo::map_row`.
fn map_row(row: &rusqlite::Row) -> rusqlite::Result<(PageTemplate, String)> {
    let template = PageTemplate {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        entity_type: row.get("entity_type")?,
        name: row.get("name")?,
        description: row.get("description")?,
        definition: Default::default(), // filled in by the caller from definition_json
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
    };
    let definition_json: String = row.get("definition_json")?;
    Ok((template, definition_json))
}

pub fn list(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<Vec<(PageTemplate, String)>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM page_templates WHERE workspace_id = ?1 AND entity_type = ?2 ORDER BY name",
    )?;
    let rows = stmt.query_map((workspace_id, entity_type), map_row)?.collect();
    rows
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<(PageTemplate, String)>> {
    conn.query_row("SELECT * FROM page_templates WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    entity_type: &str,
    name: &str,
    description: Option<&str>,
    definition_json: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO page_templates (id, workspace_id, entity_type, name, description, definition_json, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, workspace_id, entity_type, name, description, definition_json, now_iso(), actor_user_id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM page_templates WHERE id = ?1", [id])?;
    Ok(())
}

pub fn new_id() -> String {
    new_uuid()
}
