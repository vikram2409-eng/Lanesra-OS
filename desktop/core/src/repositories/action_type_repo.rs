//! Raw CRUD for `action_type_definitions` (migration 0076). See
//! `services::action_type_service` for validation, capability
//! enforcement and invocation.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::action_type::{ActionParameterSpec, ActionTypeDefinition, ActionTypeDefinitionInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ActionTypeDefinition> {
    let parameters_json: String = row.get("parameters_json")?;
    let parameters: Vec<ActionParameterSpec> = serde_json::from_str(&parameters_json).unwrap_or_default();
    Ok(ActionTypeDefinition {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        target_object_type: row.get("target_object_type")?,
        required_capability: row.get("required_capability")?,
        parameters,
        effect: row.get("effect")?,
        effect_config_json: row.get("effect_config_json")?,
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &ActionTypeDefinitionInput, actor_user_id: Option<&str>) -> rusqlite::Result<ActionTypeDefinition> {
    let now = now_iso();
    let parameters_json = serde_json::to_string(&input.parameters).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO action_type_definitions (id, workspace_id, name, description, target_object_type, required_capability, parameters_json, effect, effect_config_json, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11, ?10, ?11)",
        params![
            id, workspace_id, input.name, input.description, input.target_object_type, input.required_capability,
            parameters_json, input.effect, input.effect_config_json, now, actor_user_id,
        ],
    )?;
    get(conn, id).map(|m| m.expect("just inserted"))
}

pub fn update(conn: &Connection, id: &str, input: &ActionTypeDefinitionInput, actor_user_id: Option<&str>) -> rusqlite::Result<ActionTypeDefinition> {
    let now = now_iso();
    let parameters_json = serde_json::to_string(&input.parameters).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "UPDATE action_type_definitions SET name = ?1, description = ?2, target_object_type = ?3, required_capability = ?4,
            parameters_json = ?5, effect = ?6, effect_config_json = ?7, updated_at = ?8, updated_by = ?9
         WHERE id = ?10",
        params![input.name, input.description, input.target_object_type, input.required_capability, parameters_json, input.effect, input.effect_config_json, now, actor_user_id, id],
    )?;
    get(conn, id).map(|m| m.expect("just updated"))
}

pub fn set_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute("UPDATE action_type_definitions SET is_active = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4", params![is_active, now_iso(), actor_user_id, id])?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<ActionTypeDefinition>> {
    conn.query_row("SELECT * FROM action_type_definitions WHERE id = ?1", [id], map_row).optional()
}

pub fn get_by_name(conn: &Connection, workspace_id: &str, target_object_type: &str, name: &str) -> rusqlite::Result<Option<ActionTypeDefinition>> {
    conn.query_row(
        "SELECT * FROM action_type_definitions WHERE workspace_id = ?1 AND target_object_type = ?2 AND name = ?3",
        params![workspace_id, target_object_type, name],
        map_row,
    )
    .optional()
}

pub fn list(conn: &Connection, workspace_id: &str, target_object_type: Option<&str>) -> rusqlite::Result<Vec<ActionTypeDefinition>> {
    match target_object_type {
        Some(object_type) => {
            let mut stmt = conn.prepare("SELECT * FROM action_type_definitions WHERE workspace_id = ?1 AND target_object_type = ?2 ORDER BY name")?;
            let rows: rusqlite::Result<Vec<ActionTypeDefinition>> = stmt.query_map(params![workspace_id, object_type], map_row)?.collect();
            rows
        }
        None => {
            let mut stmt = conn.prepare("SELECT * FROM action_type_definitions WHERE workspace_id = ?1 ORDER BY target_object_type, name")?;
            let rows: rusqlite::Result<Vec<ActionTypeDefinition>> = stmt.query_map(params![workspace_id], map_row)?.collect();
            rows
        }
    }
}
