use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::workflow::{
    WorkflowAction, WorkflowActionInput, WorkflowCondition, WorkflowConditionInput, WorkflowDefinition,
    WorkflowDefinitionInput, WorkflowDefinitionUpdate, WorkflowRun,
};

fn map_condition(row: &rusqlite::Row) -> rusqlite::Result<WorkflowCondition> {
    Ok(WorkflowCondition {
        id: row.get("id")?,
        field_source: row.get("field_source")?,
        field_key: row.get("field_key")?,
        operator: row.get("operator")?,
        value: row.get("value")?,
        compare_field_source: row.get("compare_field_source")?,
        compare_field_key: row.get("compare_field_key")?,
        group_id: row.get("group_id")?,
        sort_order: row.get("sort_order")?,
    })
}

fn map_action(row: &rusqlite::Row) -> rusqlite::Result<WorkflowAction> {
    Ok(WorkflowAction {
        id: row.get("id")?,
        action_type: row.get("action_type")?,
        params_json: row.get("params_json")?,
        sort_order: row.get("sort_order")?,
    })
}

fn map_header(row: &rusqlite::Row) -> rusqlite::Result<WorkflowDefinition> {
    Ok(WorkflowDefinition {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        entity_type: row.get("entity_type")?,
        name: row.get("name")?,
        description: row.get("description")?,
        trigger_type: row.get("trigger_type")?,
        trigger_status: row.get("trigger_status")?,
        trigger_field_key: row.get("trigger_field_key")?,
        trigger_field_source: row.get("trigger_field_source")?,
        trigger_offset_days: row.get("trigger_offset_days")?,
        match_type: row.get("match_type")?,
        priority: row.get("priority")?,
        is_active: row.get("is_active")?,
        is_protected: row.get("is_protected")?,
        last_scheduled_run_at: row.get("last_scheduled_run_at")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
        conditions: Vec::new(),
        actions: Vec::new(),
    })
}

fn conditions_for(conn: &Connection, workflow_id: &str) -> rusqlite::Result<Vec<WorkflowCondition>> {
    let mut stmt = conn.prepare("SELECT * FROM workflow_conditions WHERE workflow_id = ?1 ORDER BY sort_order")?;
    let rows = stmt.query_map([workflow_id], map_condition)?.collect();
    rows
}

fn actions_for(conn: &Connection, workflow_id: &str) -> rusqlite::Result<Vec<WorkflowAction>> {
    let mut stmt = conn.prepare("SELECT * FROM workflow_actions WHERE workflow_id = ?1 ORDER BY sort_order")?;
    let rows = stmt.query_map([workflow_id], map_action)?.collect();
    rows
}

fn write_conditions(conn: &Connection, workflow_id: &str, conditions: &[WorkflowConditionInput]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM workflow_conditions WHERE workflow_id = ?1", [workflow_id])?;
    for (i, c) in conditions.iter().enumerate() {
        conn.execute(
            "INSERT INTO workflow_conditions (id, workflow_id, field_source, field_key, operator, value, compare_field_source, compare_field_key, group_id, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                new_uuid(), workflow_id, c.field_source, c.field_key, c.operator, c.value,
                c.compare_field_source, c.compare_field_key, c.group_id, i as i64
            ],
        )?;
    }
    Ok(())
}

fn write_actions(conn: &Connection, workflow_id: &str, actions: &[WorkflowActionInput]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM workflow_actions WHERE workflow_id = ?1", [workflow_id])?;
    for (i, a) in actions.iter().enumerate() {
        conn.execute(
            "INSERT INTO workflow_actions (id, workflow_id, action_type, params_json, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![new_uuid(), workflow_id, a.action_type, a.params_json, i as i64],
        )?;
    }
    Ok(())
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &WorkflowDefinitionInput, actor_user_id: Option<&str>) -> rusqlite::Result<WorkflowDefinition> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO workflow_definitions
            (id, workspace_id, entity_type, name, description, trigger_type, trigger_status, trigger_field_key,
             trigger_field_source, trigger_offset_days, match_type, priority, is_active, is_protected, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 1, 0, ?13, ?14, ?13, ?14)",
        rusqlite::params![
            id, workspace_id, input.entity_type, input.name, input.description, input.trigger_type, input.trigger_status,
            input.trigger_field_key, input.trigger_field_source, input.trigger_offset_days, input.match_type, input.priority, now, actor_user_id,
        ],
    )?;
    write_conditions(conn, id, &input.conditions)?;
    write_actions(conn, id, &input.actions)?;
    get(conn, id).map(|w| w.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<WorkflowDefinition>> {
    let header = conn
        .query_row("SELECT * FROM workflow_definitions WHERE id = ?1", [id], map_header)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })?;
    match header {
        Some(mut wf) => {
            wf.conditions = conditions_for(conn, &wf.id)?;
            wf.actions = actions_for(conn, &wf.id)?;
            Ok(Some(wf))
        }
        None => Ok(None),
    }
}

pub fn list(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<Vec<WorkflowDefinition>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM workflow_definitions WHERE workspace_id = ?1 AND entity_type = ?2 ORDER BY priority, name",
    )?;
    let headers: Vec<WorkflowDefinition> = stmt.query_map((workspace_id, entity_type), map_header)?.collect::<rusqlite::Result<_>>()?;
    let mut out = Vec::with_capacity(headers.len());
    for mut wf in headers {
        wf.conditions = conditions_for(conn, &wf.id)?;
        wf.actions = actions_for(conn, &wf.id)?;
        out.push(wf);
    }
    Ok(out)
}

/// Every active workflow for the given trigger types, across every entity
/// type - used by the scheduled scan, which needs to consider every
/// entity_type at once rather than one at a time.
pub fn list_active_by_trigger_types(conn: &Connection, workspace_id: &str, trigger_types: &[&str]) -> rusqlite::Result<Vec<WorkflowDefinition>> {
    let placeholders: Vec<String> = trigger_types.iter().enumerate().map(|(i, _)| format!("?{}", i + 2)).collect();
    let sql = format!(
        "SELECT * FROM workflow_definitions WHERE workspace_id = ?1 AND is_active = 1 AND trigger_type IN ({}) ORDER BY priority, name",
        placeholders.join(", ")
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut params: Vec<&dyn rusqlite::ToSql> = vec![&workspace_id];
    for t in trigger_types {
        params.push(t);
    }
    let headers: Vec<WorkflowDefinition> = stmt.query_map(params.as_slice(), map_header)?.collect::<rusqlite::Result<_>>()?;
    let mut out = Vec::with_capacity(headers.len());
    for mut wf in headers {
        wf.conditions = conditions_for(conn, &wf.id)?;
        wf.actions = actions_for(conn, &wf.id)?;
        out.push(wf);
    }
    Ok(out)
}

pub fn update(conn: &Connection, id: &str, input: &WorkflowDefinitionUpdate, actor_user_id: Option<&str>) -> rusqlite::Result<WorkflowDefinition> {
    conn.execute(
        "UPDATE workflow_definitions
         SET name = ?1, description = ?2, trigger_status = ?3, trigger_field_key = ?4, trigger_field_source = ?5,
             trigger_offset_days = ?6, match_type = ?7, priority = ?8, is_active = ?9, updated_at = ?10, updated_by = ?11
         WHERE id = ?12",
        rusqlite::params![
            input.name, input.description, input.trigger_status, input.trigger_field_key, input.trigger_field_source,
            input.trigger_offset_days, input.match_type, input.priority, input.is_active, now_iso(), actor_user_id, id,
        ],
    )?;
    write_conditions(conn, id, &input.conditions)?;
    write_actions(conn, id, &input.actions)?;
    get(conn, id).map(|w| w.expect("just updated"))
}

/// Admin UX polish (spec §10) - see `business_rule_repo`'s version-history
/// functions for the full rationale; identical bounded-snapshot mechanism.
pub const VERSION_HISTORY_LIMIT: i64 = 10;

pub fn insert_version(conn: &Connection, workflow_id: &str, snapshot_json: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO workflow_rule_versions (id, workflow_id, snapshot_json, saved_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![new_uuid(), workflow_id, snapshot_json, now_iso()],
    )?;
    conn.execute(
        "DELETE FROM workflow_rule_versions WHERE workflow_id = ?1 AND id NOT IN (
            SELECT id FROM workflow_rule_versions WHERE workflow_id = ?1 ORDER BY saved_at DESC LIMIT ?2
        )",
        rusqlite::params![workflow_id, VERSION_HISTORY_LIMIT],
    )?;
    Ok(())
}

/// Raw `(id, snapshot_json, saved_at)` rows, newest first - the service
/// layer deserializes `snapshot_json` into a full `WorkflowDefinition`.
pub fn list_version_rows(conn: &Connection, workflow_id: &str) -> rusqlite::Result<Vec<(String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, snapshot_json, saved_at FROM workflow_rule_versions WHERE workflow_id = ?1 ORDER BY saved_at DESC",
    )?;
    let rows = stmt
        .query_map([workflow_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)))?
        .collect();
    rows
}

pub fn set_last_scheduled_run(conn: &Connection, id: &str, at: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE workflow_definitions SET last_scheduled_run_at = ?1 WHERE id = ?2", (at, id))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn record_run(
    conn: &Connection,
    workspace_id: &str,
    workflow_id: &str,
    entity_type: &str,
    entity_id: Option<&str>,
    trigger_type: &str,
    outcome: &str,
    actions_summary: Option<&str>,
    error_message: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO workflow_runs (id, workspace_id, workflow_id, entity_type, entity_id, trigger_type, triggered_at, outcome, actions_summary, error_message)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![new_uuid(), workspace_id, workflow_id, entity_type, entity_id, trigger_type, now_iso(), outcome, actions_summary, error_message],
    )?;
    Ok(())
}

pub fn has_run_for_entity(conn: &Connection, workflow_id: &str, entity_id: &str) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM workflow_runs WHERE workflow_id = ?1 AND entity_id = ?2",
        (workflow_id, entity_id),
        |r| r.get(0),
    )?;
    Ok(count > 0)
}

pub fn list_runs(conn: &Connection, workspace_id: &str, workflow_id: &str) -> rusqlite::Result<Vec<WorkflowRun>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM workflow_runs WHERE workspace_id = ?1 AND workflow_id = ?2 ORDER BY triggered_at DESC LIMIT 50",
    )?;
    let rows = stmt.query_map((workspace_id, workflow_id), |row| {
        Ok(WorkflowRun {
            id: row.get("id")?,
            workspace_id: row.get("workspace_id")?,
            workflow_id: row.get("workflow_id")?,
            entity_type: row.get("entity_type")?,
            entity_id: row.get("entity_id")?,
            trigger_type: row.get("trigger_type")?,
            triggered_at: row.get("triggered_at")?,
            outcome: row.get("outcome")?,
            actions_summary: row.get("actions_summary")?,
            error_message: row.get("error_message")?,
        })
    })?.collect();
    rows
}
