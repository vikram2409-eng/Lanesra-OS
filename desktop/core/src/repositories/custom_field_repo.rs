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
        min_value: row.get("min_value")?,
        max_value: row.get("max_value")?,
        max_length: row.get("max_length")?,
        regex_pattern: row.get("regex_pattern")?,
        is_searchable: row.get("is_searchable")?,
        is_filterable: row.get("is_filterable")?,
        is_reportable: row.get("is_reportable")?,
        default_value: row.get("default_value")?,
        is_unique: row.get("is_unique")?,
        help_text: row.get("help_text")?,
        placeholder: row.get("placeholder")?,
        is_hidden_by_default: row.get("is_hidden_by_default")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
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
            (id, workspace_id, entity_type, key, label, field_type, options_json, required, show_in_list, sort_order, is_active,
             min_value, max_value, max_length, regex_pattern, is_searchable, is_filterable, is_reportable,
             default_value, is_unique, help_text, placeholder, is_hidden_by_default, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
                 ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?23, ?24)",
        rusqlite::params![
            id, workspace_id, input.entity_type, key, input.label, input.field_type,
            options_json, input.required, input.show_in_list, input.sort_order,
            input.min_value, input.max_value, input.max_length, input.regex_pattern,
            input.is_searchable, input.is_filterable, input.is_reportable,
            input.default_value, input.is_unique, input.help_text, input.placeholder, input.is_hidden_by_default, now, actor_user_id,
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
         SET label = ?1, options_json = ?2, required = ?3, show_in_list = ?4, sort_order = ?5, is_active = ?6,
             min_value = ?7, max_value = ?8, max_length = ?9, regex_pattern = ?10,
             is_searchable = ?11, is_filterable = ?12, is_reportable = ?13,
             default_value = ?14, is_unique = ?15, help_text = ?16, placeholder = ?17, is_hidden_by_default = ?18,
             updated_at = ?19, updated_by = ?20
         WHERE id = ?21",
        rusqlite::params![
            input.label, options_json, input.required, input.show_in_list, input.sort_order, input.is_active,
            input.min_value, input.max_value, input.max_length, input.regex_pattern,
            input.is_searchable, input.is_filterable, input.is_reportable,
            input.default_value, input.is_unique, input.help_text, input.placeholder, input.is_hidden_by_default,
            now_iso(), actor_user_id, id,
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

/// Addendum Phase 4: whether `value` is already stored for this
/// definition on some entity other than `entity_id` - the check
/// `is_unique` enforcement needs before writing.
pub fn value_exists_elsewhere(conn: &Connection, definition_id: &str, entity_id: &str, value: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM custom_field_values WHERE definition_id = ?1 AND value_text = ?2 AND entity_id != ?3)",
        (definition_id, value, entity_id),
        |row| row.get(0),
    )
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

/// Every `is_filterable` value for every active definition of one entity
/// type, across the whole workspace, keyed by entity id then field key -
/// powers desktop list-view filtering (roadmap "Global search &
/// list-view filtering"). One query per list screen load rather than one
/// `get_values` call per row, and scoped to filterable fields only so a
/// list screen never pulls values it has no control to show.
pub fn get_filterable_values(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
) -> rusqlite::Result<HashMap<String, HashMap<String, String>>> {
    let mut stmt = conn.prepare(
        "SELECT v.entity_id, d.key, v.value_text FROM custom_field_values v
         JOIN custom_field_definitions d ON d.id = v.definition_id
         WHERE d.workspace_id = ?1 AND d.entity_type = ?2 AND d.is_active = 1 AND d.is_filterable = 1",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![workspace_id, entity_type], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut out: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (entity_id, key, value) in rows {
        out.entry(entity_id).or_default().insert(key, value);
    }
    Ok(out)
}
