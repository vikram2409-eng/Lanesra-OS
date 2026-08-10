//! FR-WFL Phase 1: admin-defined workflow automation - "when an
//! Opportunity's stage (or an Invoice's status) transitions to X, create a
//! follow-up Task automatically." See the migration's header comment for
//! why this phase is scoped to just these two entities and just one
//! action (task creation), rather than the fuller workflow-automation
//! brainstorm in the product backlog.
//!
//! Unlike FR-RUL's rules, which can conflict (two rules targeting the same
//! field), workflow rules are purely additive - every active rule whose
//! trigger_status matches the entity's new value fires and creates its own
//! Task, so there is no "highest wins" ordering to worry about here.

use chrono::{Duration, Utc};
use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::company::COMPANY_STATUSES;
use crate::models::contact::CONTACT_STATUSES;
use crate::models::contract::CONTRACT_STATUSES;
use crate::models::invoice::INVOICE_STATUSES;
use crate::models::opportunity::OPPORTUNITY_STAGES;
use crate::models::order::ORDER_STATUSES;
use crate::models::quote::QUOTE_STATUSES;
use crate::models::task::{TaskInput, TASK_STATUSES};
use crate::models::workflow_rule::{
    transition_field_for, WorkflowRule, WorkflowRuleInput, WorkflowRuleUpdate, WORKFLOW_ENTITY_TYPES,
};
use crate::repositories::{user_repo, workflow_rule_repo};
use crate::services::task_service;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage workflow automation".into()));
    }
    Ok(())
}

fn valid_statuses_for(entity_type: &str) -> &'static [&'static str] {
    match entity_type {
        "Company" => COMPANY_STATUSES,
        "Contact" => CONTACT_STATUSES,
        "Opportunity" => OPPORTUNITY_STAGES,
        "Quote" => QUOTE_STATUSES,
        "Order" => ORDER_STATUSES,
        "Invoice" => INVOICE_STATUSES,
        "Contract" => CONTRACT_STATUSES,
        "Task" => TASK_STATUSES,
        _ => &[],
    }
}

fn validate_shape(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
    trigger_status: &str,
    task_title: &str,
    due_in_days: i64,
    assignee_user_id: Option<&str>,
) -> AppResult<()> {
    if !WORKFLOW_ENTITY_TYPES.contains(&entity_type) {
        return Err(AppError::Validation(format!("Invalid workflow entity type '{entity_type}'")));
    }
    if !valid_statuses_for(entity_type).contains(&trigger_status) {
        return Err(AppError::Validation(format!(
            "'{trigger_status}' is not a valid {entity_type} {}",
            transition_field_for(entity_type)
        )));
    }
    if task_title.trim().is_empty() {
        return Err(AppError::Validation("Task title is required".into()));
    }
    if due_in_days < 0 {
        return Err(AppError::Validation("Due date offset cannot be negative".into()));
    }
    if let Some(user_id) = assignee_user_id {
        let user = user_repo::find_by_id(conn, user_id)?
            .ok_or_else(|| AppError::Validation("Selected assignee does not exist".into()))?;
        if user.workspace_id != workspace_id {
            return Err(AppError::Validation("Selected assignee does not exist".into()));
        }
    }

    Ok(())
}

pub fn create_rule(
    conn: &Connection,
    workspace_id: &str,
    input: &WorkflowRuleInput,
    actor_user_id: Option<&str>,
) -> AppResult<WorkflowRule> {
    require_admin(conn, actor_user_id)?;
    validate_shape(
        conn, workspace_id, &input.entity_type, &input.trigger_status, &input.task_title,
        input.due_in_days, input.assignee_user_id.as_deref(),
    )?;
    let id = new_uuid();
    Ok(workflow_rule_repo::create(conn, &id, workspace_id, input, actor_user_id)?)
}

/// Only an Administrator can see (and manage) workflow rules - unlike
/// FR-RUL's field rules, nothing client-side needs to evaluate these live,
/// since they only take effect on the server at the moment of a status
/// transition.
pub fn list_rules(conn: &Connection, workspace_id: &str, entity_type: &str, actor_user_id: Option<&str>) -> AppResult<Vec<WorkflowRule>> {
    require_admin(conn, actor_user_id)?;
    Ok(workflow_rule_repo::list(conn, workspace_id, entity_type)?)
}

pub fn update_rule(conn: &Connection, id: &str, input: &WorkflowRuleUpdate, actor_user_id: Option<&str>) -> AppResult<WorkflowRule> {
    require_admin(conn, actor_user_id)?;
    let existing = workflow_rule_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Workflow rule".into()))?;
    validate_shape(
        conn, &existing.workspace_id, &existing.entity_type, &input.trigger_status, &input.task_title,
        input.due_in_days, input.assignee_user_id.as_deref(),
    )?;
    Ok(workflow_rule_repo::update(conn, id, input, actor_user_id)?)
}

/// Evaluates every active rule for `entity_type` against a stage/status
/// transition and creates the resulting Tasks. A no-op unless `old_value`
/// actually differs from `new_value` (re-saving a record without changing
/// its stage/status must never re-fire an already-fired rule). Returns the
/// number of tasks created.
pub fn fire_transition(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
    entity_id: &str,
    old_value: &str,
    new_value: &str,
    fallback_owner_user_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<usize> {
    if old_value == new_value {
        return Ok(0);
    }
    let rules = workflow_rule_repo::list(conn, workspace_id, entity_type)?;
    let related_type = entity_type; // TASK_RELATED_TYPES uses the same names.
    let today = Utc::now();
    let mut created = 0;

    for rule in rules.iter().filter(|r| r.is_active && r.trigger_status == new_value) {
        let due_date = (today + Duration::days(rule.due_in_days))
            .format("%Y-%m-%d")
            .to_string();
        let owner_user_id = rule
            .assignee_user_id
            .clone()
            .or_else(|| fallback_owner_user_id.map(String::from));

        let input = TaskInput {
            title: rule.task_title.clone(),
            description: rule.task_description.clone(),
            owner_user_id,
            priority: "Normal".into(),
            status: "Not Started".into(),
            due_date: Some(due_date),
            reminder_at: None,
            related_type: Some(related_type.to_string()),
            related_id: Some(entity_id.to_string()),
        };
        task_service::create(conn, workspace_id, &input, actor_user_id)?;
        created += 1;
    }

    Ok(created)
}
