use std::collections::HashMap;

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::custom_field::{CustomFieldDefinition, CustomFieldDefinitionInput, CustomFieldDefinitionUpdate};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<CustomFieldDefinition> {
    let options_json: Option<String> = row.get("options_json")?;
    let options = options_json
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default();
    Ok(CustomFieldDefinition {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        entity_type: row.get("entity_type")?,
        key: row.get("key")?,
        label: row.get("label")?,
        field_type: row.get("field_type")?,
        options,
        required: row.get("required")?,
        show_in_list: row.get("show_in_list")?,
        sort_order: row.get("sort_order")?,
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create_definition(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    key: &str,
    input: &CustomFieldDefinitionInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<CustomFieldDefinition> {
    let now = now_iso();
    let options_json = serde_json::to_string(&input.options).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "INSERT INTO custom_field_definitions
            (id, workspace_id, entity_type, key, label, field_type, options_json, required, show_in_list, sort_order, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, ?11, ?12, ?11, ?12)",
        rusqlite::params![
            id, workspace_id, input.entity_type, key, input.label, input.field_type,
            options_json, input.required, input.show_in_list, input.sort_order, now, actor_user_id,
        ],
    )?;
    get_definition(conn, id).map(|d| d.expect("just inserted"))
}

pub fn get_definition(conn: &Connection, id: &str) -> rusqlite::Result<Option<CustomFieldDefinition>> {
    conn.query_row("SELECT * FROM custom_field_definitions WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// All definitions for the entity type, active and inactive - the service
/// layer decides which to expose to which caller (forms only want active
/// ones; the admin screen wants everything, ordered by sort_order).
pub fn list_definitions(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<Vec<CustomFieldDefinition>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM custom_field_definitions WHERE workspace_id = ?1 AND entity_type = ?2 ORDER BY sort_order, label",
    )?;
    let rows = stmt.query_map((workspace_id, entity_type), map_row)?.collect();
    rows
}

pub fn update_definition(conn: &Connection, id: &str, input: &CustomFieldDefinitionUpdate, actor_user_id: Option<&str>) -> rusqlite::Result<CustomFieldDefinition> {
    let options_json = serde_json::to_string(&input.options).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "UPDATE custom_field_definitions
         SET label = ?1, options_json = ?2, required = ?3, show_in_list = ?4, sort_order = ?5, is_active = ?6, updated_at = ?7, updated_by = ?8
         WHERE id = ?9",
        rusqlite::params![
            input.label, options_json, input.required, input.show_in_list, input.sort_order,
            input.is_active, now_iso(), actor_user_id, id,
        ],
    )?;
    get_definition(conn, id).map(|d| d.expect("just updated"))
}

/// Upserts one value row per (definition_id, entity_id); a definition
/// whose value is now empty has its row removed entirely rather than
/// stored as an empty string, keeping "no value" and "empty string"
/// indistinguishable the same way every other optional field in this
/// schema treats them.
pub fn set_value(conn: &Connection, definition_id: &str, entity_id: &str, value: &str) -> rusqlite::Result<()> {
    if value.trim().is_empty() {
        conn.execute(
            "DELETE FROM custom_field_values WHERE definition_id = ?1 AND entity_id = ?2",
            (definition_id, entity_id),
        )?;
        return Ok(());
    }
    let now = now_iso();
    conn.execute(
        "INSERT INTO custom_field_values (id, definition_id, entity_id, value_text, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)
         ON CONFLICT(definition_id, entity_id) DO UPDATE SET value_text = excluded.value_text, updated_at = excluded.updated_at",
        (crate::domain::ids::new_uuid(), definition_id, entity_id, value, now),
    )?;
    Ok(())
}

/// Values for one entity, keyed by field key (joined through the
/// definition so callers never need to know definition ids).
pub fn get_values(conn: &Connection, entity_id: &str) -> rusqlite::Result<HashMap<String, String>> {
    let mut stmt = conn.prepare(
        "SELECT d.key, v.value_text FROM custom_field_values v
         JOIN custom_field_definitions d ON d.id = v.definition_id
         WHERE v.entity_id = ?1",
    )?;
    let rows = stmt
        .query_map([entity_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows.into_iter().collect())
}
