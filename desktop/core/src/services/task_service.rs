use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::numbering::{self, TASK};
use crate::domain::{AppError, AppResult};
use crate::models::task::{Task, TaskInput, TASK_PRIORITIES, TASK_STATUSES};
use crate::repositories::{audit_repo, task_repo};
use crate::services::{entity_registry, workflow_service};

/// A task's related record can be any built-in entity type task_links has
/// always supported, or (Phase D, ADM-WF-06) any active custom object -
/// entity_registry::exists already knows how to check both without this
/// module needing its own custom-object lookup, and task_links.related_type
/// has no CHECK constraint since migration 0014 for exactly this reason.
fn validate_relation(conn: &Connection, input: &TaskInput) -> AppResult<()> {
    match (&input.related_type, &input.related_id) {
        (None, None) => Ok(()),
        (Some(related_type), Some(related_id)) => {
            if !entity_registry::exists(conn, related_type, related_id)? {
                return Err(AppError::Validation(format!(
                    "The selected {related_type} record does not exist"
                )));
            }
            Ok(())
        }
        _ => Err(AppError::Validation(
            "A task must have both a relationship type and a related record, or neither".into(),
        )),
    }
}

fn validate(conn: &Connection, input: &TaskInput) -> AppResult<()> {
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("Task title is required".into()));
    }
    if !TASK_PRIORITIES.contains(&input.priority.as_str()) {
        return Err(AppError::Validation(format!("Invalid task priority '{}'", input.priority)));
    }
    if !TASK_STATUSES.contains(&input.status.as_str()) {
        return Err(AppError::Validation(format!("Invalid task status '{}'", input.status)));
    }
    validate_relation(conn, input)
}

pub fn create(
    conn: &Connection,
    workspace_id: &str,
    input: &TaskInput,
    actor_user_id: Option<&str>,
) -> AppResult<Task> {
    validate(conn, input)?;
    let id = new_uuid();
    let task_number = numbering::allocate_number(conn, workspace_id, &TASK)?;
    let task = task_repo::create(conn, &id, workspace_id, &task_number, input, actor_user_id)?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "create",
        Some("task"),
        Some(&task.id),
        &format!("Created task {}", task.task_number),
        None,
    )?;
    workflow_service::fire_event(conn, workspace_id, "Task", &task.id, None, &task.status, task.owner_user_id.as_deref(), actor_user_id)?;
    Ok(task)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Task> {
    task_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Task".into()))
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Task>> {
    Ok(task_repo::list(conn, workspace_id)?)
}

pub fn list_by_related(conn: &Connection, related_type: &str, related_id: &str) -> AppResult<Vec<Task>> {
    Ok(task_repo::list_by_related(conn, related_type, related_id)?)
}

pub fn update(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    input: &TaskInput,
    actor_user_id: Option<&str>,
) -> AppResult<Task> {
    validate(conn, input)?;
    let before = get(conn, id)?;
    let task = task_repo::update(conn, id, input, actor_user_id)?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "update",
        Some("task"),
        Some(id),
        &format!("Updated task {} (status: {})", task.task_number, task.status),
        None,
    )?;
    workflow_service::fire_event(
        conn, workspace_id, "Task", id, Some(&before.status), &task.status, task.owner_user_id.as_deref(), actor_user_id,
    )?;
    Ok(task)
}

pub fn archive(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let existing = get(conn, id)?;
    task_repo::archive(conn, id, actor_user_id)?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "archive",
        Some("task"),
        Some(id),
        &format!("Archived task {}", existing.task_number),
        None,
    )?;
    Ok(())
}
