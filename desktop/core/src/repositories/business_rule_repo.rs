use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::business_rule::{
    BusinessRule, BusinessRuleAction, BusinessRuleActionInput, BusinessRuleCondition, BusinessRuleConditionInput,
    BusinessRuleInput, BusinessRuleUpdate,
};

fn map_condition(row: &rusqlite::Row) -> rusqlite::Result<BusinessRuleCondition> {
    Ok(BusinessRuleCondition {
        id: row.get("id")?,
        field_source: row.get("field_source")?,
        field_key: row.get("field_key")?,
        operator: row.get("operator")?,
        value: row.get("value")?,
        sort_order: row.get("sort_order")?,
    })
}

fn map_action(row: &rusqlite::Row) -> rusqlite::Result<BusinessRuleAction> {
    Ok(BusinessRuleAction {
        id: row.get("id")?,
        action_type: row.get("action_type")?,
        target_field_key: row.get("target_field_key")?,
        target_field_source: row.get("target_field_source")?,
        action_value: row.get("action_value")?,
        message: row.get("message")?,
        sort_order: row.get("sort_order")?,
    })
}

fn conditions_for(conn: &Connection, rule_id: &str) -> rusqlite::Result<Vec<BusinessRuleCondition>> {
    let mut stmt = conn.prepare("SELECT * FROM business_rule_conditions WHERE rule_id = ?1 ORDER BY sort_order")?;
    let rows = stmt.query_map([rule_id], map_condition)?.collect();
    rows
}

fn actions_for(conn: &Connection, rule_id: &str) -> rusqlite::Result<Vec<BusinessRuleAction>> {
    let mut stmt = conn.prepare("SELECT * FROM business_rule_actions WHERE rule_id = ?1 ORDER BY sort_order")?;
    let rows = stmt.query_map([rule_id], map_action)?.collect();
    rows
}

fn map_rule_header(row: &rusqlite::Row) -> rusqlite::Result<BusinessRule> {
    Ok(BusinessRule {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        entity_type: row.get("entity_type")?,
        name: row.get("name")?,
        description: row.get("description")?,
        match_type: row.get("match_type")?,
        priority: row.get("priority")?,
        is_active: row.get("is_active")?,
        effective_start_date: row.get("effective_start_date")?,
        effective_end_date: row.get("effective_end_date")?,
        is_protected: row.get("is_protected")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        conditions: Vec::new(),
        actions: Vec::new(),
    })
}

fn write_conditions(conn: &Connection, rule_id: &str, conditions: &[BusinessRuleConditionInput]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM business_rule_conditions WHERE rule_id = ?1", [rule_id])?;
    for (i, c) in conditions.iter().enumerate() {
        conn.execute(
            "INSERT INTO business_rule_conditions (id, rule_id, field_source, field_key, operator, value, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![new_uuid(), rule_id, c.field_source, c.field_key, c.operator, c.value, i as i64],
        )?;
    }
    Ok(())
}

fn write_actions(conn: &Connection, rule_id: &str, actions: &[BusinessRuleActionInput]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM business_rule_actions WHERE rule_id = ?1", [rule_id])?;
    for (i, a) in actions.iter().enumerate() {
        conn.execute(
            "INSERT INTO business_rule_actions (id, rule_id, action_type, target_field_key, target_field_source, action_value, message, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![new_uuid(), rule_id, a.action_type, a.target_field_key, a.target_field_source, a.action_value, a.message, i as i64],
        )?;
    }
    Ok(())
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &BusinessRuleInput, actor_user_id: Option<&str>) -> rusqlite::Result<BusinessRule> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO business_rules
            (id, workspace_id, entity_type, name, description, match_type, priority, is_active,
             effective_start_date, effective_end_date, is_protected, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9, 0, ?10, ?11, ?10, ?11)",
        rusqlite::params![
            id, workspace_id, input.entity_type, input.name, input.description, input.match_type, input.priority,
            input.effective_start_date, input.effective_end_date, now, actor_user_id,
        ],
    )?;
    write_conditions(conn, id, &input.conditions)?;
    write_actions(conn, id, &input.actions)?;
    get(conn, id).map(|r| r.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<BusinessRule>> {
    let header = conn
        .query_row("SELECT * FROM business_rules WHERE id = ?1", [id], map_rule_header)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })?;
    match header {
        Some(mut rule) => {
            rule.conditions = conditions_for(conn, &rule.id)?;
            rule.actions = actions_for(conn, &rule.id)?;
            Ok(Some(rule))
        }
        None => Ok(None),
    }
}

pub fn list(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<Vec<BusinessRule>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM business_rules WHERE workspace_id = ?1 AND entity_type = ?2 ORDER BY priority, name",
    )?;
    let headers: Vec<BusinessRule> = stmt.query_map((workspace_id, entity_type), map_rule_header)?.collect::<rusqlite::Result<_>>()?;
    let mut out = Vec::with_capacity(headers.len());
    for mut rule in headers {
        rule.conditions = conditions_for(conn, &rule.id)?;
        rule.actions = actions_for(conn, &rule.id)?;
        out.push(rule);
    }
    Ok(out)
}

pub fn update(conn: &Connection, id: &str, input: &BusinessRuleUpdate, actor_user_id: Option<&str>) -> rusqlite::Result<BusinessRule> {
    conn.execute(
        "UPDATE business_rules
         SET name = ?1, description = ?2, match_type = ?3, priority = ?4, is_active = ?5,
             effective_start_date = ?6, effective_end_date = ?7, updated_at = ?8, updated_by = ?9
         WHERE id = ?10",
        rusqlite::params![
            input.name, input.description, input.match_type, input.priority, input.is_active,
            input.effective_start_date, input.effective_end_date, now_iso(), actor_user_id, id,
        ],
    )?;
    write_conditions(conn, id, &input.conditions)?;
    write_actions(conn, id, &input.actions)?;
    get(conn, id).map(|r| r.expect("just updated"))
}
