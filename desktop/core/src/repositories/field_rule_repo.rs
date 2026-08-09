use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::field_rule::{FieldRule, FieldRuleInput, FieldRuleUpdate};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<FieldRule> {
    Ok(FieldRule {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        entity_type: row.get("entity_type")?,
        trigger_field_source: row.get("trigger_field_source")?,
        trigger_field_key: row.get("trigger_field_key")?,
        operator: row.get("operator")?,
        trigger_value: row.get("trigger_value")?,
        target_field_key: row.get("target_field_key")?,
        effect: row.get("effect")?,
        is_active: row.get("is_active")?,
        sort_order: row.get("sort_order")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    input: &FieldRuleInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<FieldRule> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO field_rules
            (id, workspace_id, entity_type, trigger_field_source, trigger_field_key, operator, trigger_value, target_field_key, effect, is_active, sort_order, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11, ?12, ?11, ?12)",
        rusqlite::params![
            id, workspace_id, input.entity_type, input.trigger_field_source, input.trigger_field_key,
            input.operator, input.trigger_value, input.target_field_key, input.effect, input.sort_order,
            now, actor_user_id,
        ],
    )?;
    get(conn, id).map(|r| r.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<FieldRule>> {
    conn.query_row("SELECT * FROM field_rules WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// All rules for the entity type - active and inactive; the service layer
/// filters to active-only for evaluation, and returns everything for the
/// admin screen.
pub fn list(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<Vec<FieldRule>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM field_rules WHERE workspace_id = ?1 AND entity_type = ?2 ORDER BY sort_order",
    )?;
    let rows = stmt.query_map((workspace_id, entity_type), map_row)?.collect();
    rows
}

pub fn update(conn: &Connection, id: &str, input: &FieldRuleUpdate, actor_user_id: Option<&str>) -> rusqlite::Result<FieldRule> {
    conn.execute(
        "UPDATE field_rules
         SET trigger_field_source = ?1, trigger_field_key = ?2, operator = ?3, trigger_value = ?4,
             target_field_key = ?5, effect = ?6, sort_order = ?7, is_active = ?8, updated_at = ?9, updated_by = ?10
         WHERE id = ?11",
        rusqlite::params![
            input.trigger_field_source, input.trigger_field_key, input.operator, input.trigger_value,
            input.target_field_key, input.effect, input.sort_order, input.is_active, now_iso(), actor_user_id, id,
        ],
    )?;
    get(conn, id).map(|r| r.expect("just updated"))
}
