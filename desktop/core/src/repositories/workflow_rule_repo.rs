use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::workflow_rule::{WorkflowRule, WorkflowRuleInput, WorkflowRuleUpdate};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<WorkflowRule> {
    Ok(WorkflowRule {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        entity_type: row.get("entity_type")?,
        trigger_status: row.get("trigger_status")?,
        task_title: row.get("task_title")?,
        task_description: row.get("task_description")?,
        due_in_days: row.get("due_in_days")?,
        assignee_user_id: row.get("assignee_user_id")?,
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    input: &WorkflowRuleInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<WorkflowRule> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO workflow_rules
            (id, workspace_id, entity_type, trigger_status, task_title, task_description, due_in_days, assignee_user_id, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?10, ?9, ?10)",
        rusqlite::params![
            id, workspace_id, input.entity_type, input.trigger_status, input.task_title,
            input.task_description, input.due_in_days, input.assignee_user_id, now, actor_user_id,
        ],
    )?;
    get(conn, id).map(|r| r.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<WorkflowRule>> {
    conn.query_row("SELECT * FROM workflow_rules WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// All rules for the entity type - active and inactive; the service layer
/// filters to active-only for evaluation, and returns everything for the
/// admin screen.
pub fn list(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<Vec<WorkflowRule>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM workflow_rules WHERE workspace_id = ?1 AND entity_type = ?2 ORDER BY created_at",
    )?;
    let rows = stmt.query_map((workspace_id, entity_type), map_row)?.collect();
    rows
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &WorkflowRuleUpdate,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<WorkflowRule> {
    conn.execute(
        "UPDATE workflow_rules
         SET trigger_status = ?1, task_title = ?2, task_description = ?3, due_in_days = ?4,
             assignee_user_id = ?5, is_active = ?6, updated_at = ?7, updated_by = ?8
         WHERE id = ?9",
        rusqlite::params![
            input.trigger_status, input.task_title, input.task_description, input.due_in_days,
            input.assignee_user_id, input.is_active, now_iso(), actor_user_id, id,
        ],
    )?;
    get(conn, id).map(|r| r.expect("just updated"))
}
