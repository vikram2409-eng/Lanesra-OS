//! Admin extensibility Phase D (spec §23/ADM-WF): a no-code Trigger ->
//! Conditions -> Actions workflow engine, replacing the original single-
//! trigger (status transition) / single-action (create task) workflow_rules.
//!
//! Triggers: `record_created`/`record_updated`/`status_changed` fire from
//! [`fire_event`], called by the same 8 entity services (plus
//! custom_record_service) that already call it on every create/update -
//! no new integration point needed there, just a richer engine behind the
//! same call. `field_changed` fires from `custom_field_service::set_entity_values`
//! (see [`fire_field_changed`]), the same seam Business Rules already use.
//! `date_reached`/`due_overdue`/`scheduled` fire from [`run_scheduled`], a
//! periodic scan the frontend polls while the app is open (ADM-WF-11) -
//! see that function's own comment for why a full OS-level background
//! scheduler is out of scope.
//!
//! Conditions reuse `domain::conditions` (the same AND/OR matcher Business
//! Rules uses). Actions are stored as an `action_type` string plus an
//! opaque `params_json` blob parsed here into one of the `*Params` structs
//! below - see `models::workflow`'s doc comment for why.
//!
//! Recursion (ADM-WF-09): most actions that can create or update a record
//! write through a plain repo function, never back through a service's own
//! create()/update() (which is what fires events) - except
//! `create_related_record`, which does call `custom_record_service::create`,
//! and `update_field`/business rules' `set_value` when targeting a
//! built-in field, which route through `builtin_field_service::set_field`
//! (itself calling the target entity's own `*_service::update` so ordinary
//! validation still runs - see that module's doc comment). Both can
//! therefore recurse. A thread-local depth counter ([`WORKFLOW_DEPTH`])
//! bounds this regardless of branching factor or which of these two paths
//! caused it, without threading a depth parameter through every entity
//! service's public API.

use std::cell::Cell;
use std::collections::HashMap;

use chrono::{Duration, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::domain::conditions::conditions_match;
use crate::domain::ids::new_uuid;
use crate::domain::{builtin_fields, AppError, AppResult};
use crate::models::business_rule::builtin_trigger_field_for;
use crate::models::task::TaskInput;
use crate::models::workflow::{
    transition_field_for, WorkflowDefinition, WorkflowDefinitionInput, WorkflowDefinitionUpdate, ACTION_TYPES,
    CORE_WORKFLOW_ENTITY_TYPES, NOTIFICATION_AUDIENCES, TRIGGER_TYPES,
};
use crate::repositories::{
    company_repo, contract_repo, custom_field_repo, custom_record_repo, notification_repo, opportunity_repo,
    relationship_repo, task_repo, user_repo, workflow_repo,
};
use crate::services::{builtin_field_service, company_service, custom_object_service, custom_record_service, entity_registry, task_service};

thread_local! {
    static WORKFLOW_DEPTH: Cell<u8> = const { Cell::new(0) };
}
const MAX_WORKFLOW_DEPTH: u8 = 5;

/// RAII guard incrementing the thread-local recursion depth for the
/// lifetime of one top-level `fire_event`/`fire_field_changed`/
/// `run_scheduled` call, decrementing on every exit path (including `?`).
struct DepthGuard;
impl DepthGuard {
    fn enter() -> Option<DepthGuard> {
        let depth = WORKFLOW_DEPTH.with(|d| d.get());
        if depth >= MAX_WORKFLOW_DEPTH {
            return None;
        }
        WORKFLOW_DEPTH.with(|d| d.set(depth + 1));
        Some(DepthGuard)
    }
}
impl Drop for DepthGuard {
    fn drop(&mut self) {
        WORKFLOW_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    }
}

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage workflow automation".into()));
    }
    Ok(())
}

/// The valid values for `transition_field_for(entity_type)` - status for
/// most entities, Opportunity's stage list for Opportunity, and the fixed
/// custom-record status set for any custom object. Mirrors the pre-Phase-D
/// engine's own `valid_statuses_for` so a status_changed trigger's target
/// value is still validated against the entity's real enum, not just
/// checked for non-emptiness.
fn valid_transition_values_for(entity_type: &str) -> &'static [&'static str] {
    use crate::models::{
        company::COMPANY_STATUSES, contact::CONTACT_STATUSES, contract::CONTRACT_STATUSES, custom_object::CUSTOM_RECORD_STATUSES,
        invoice::INVOICE_STATUSES, opportunity::OPPORTUNITY_STAGES, order::ORDER_STATUSES, quote::QUOTE_STATUSES, task::TASK_STATUSES,
    };
    match entity_type {
        "Company" => COMPANY_STATUSES,
        "Contact" => CONTACT_STATUSES,
        "Opportunity" => OPPORTUNITY_STAGES,
        "Quote" => QUOTE_STATUSES,
        "Order" => ORDER_STATUSES,
        "Invoice" => INVOICE_STATUSES,
        "Contract" => CONTRACT_STATUSES,
        "Task" => TASK_STATUSES,
        _ => CUSTOM_RECORD_STATUSES,
    }
}

fn require_valid_entity_type(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<()> {
    if CORE_WORKFLOW_ENTITY_TYPES.contains(&entity_type) {
        return Ok(());
    }
    if custom_object_service::is_valid_dynamic_entity_type(conn, workspace_id, entity_type)? {
        return Ok(());
    }
    Err(AppError::Validation(format!("'{entity_type}' is not a recognized object type")))
}

// --- Validation -------------------------------------------------------

fn validate_conditions(conn: &Connection, workspace_id: &str, entity_type: &str, conditions: &[crate::models::workflow::WorkflowConditionInput]) -> AppResult<()> {
    let defs = custom_field_repo::list_definitions(conn, workspace_id, entity_type)?;
    let active_keys: Vec<&str> = defs.iter().filter(|d| d.is_active).map(|d| d.key.as_str()).collect();
    for c in conditions {
        if !crate::domain::conditions::TRIGGER_SOURCES.contains(&c.field_source.as_str()) {
            return Err(AppError::Validation(format!("Invalid condition source '{}'", c.field_source)));
        }
        if !crate::domain::conditions::CONDITION_OPERATORS.contains(&c.operator.as_str()) {
            return Err(AppError::Validation(format!("Invalid condition operator '{}'", c.operator)));
        }
        if !crate::domain::conditions::field_ref_is_valid(entity_type, &c.field_source, &c.field_key, active_keys.iter().copied()) {
            return Err(AppError::Validation(format!("'{}' is not a valid field to trigger on", c.field_key)));
        }
        // Addendum §3.2: field-to-field comparison, same rule as business
        // rules' identical check - see BusinessRuleCondition's doc comment.
        match (&c.compare_field_source, &c.compare_field_key) {
            (Some(src), Some(key)) => {
                if !crate::domain::conditions::TRIGGER_SOURCES.contains(&src.as_str()) {
                    return Err(AppError::Validation(format!("Invalid comparison field source '{src}'")));
                }
                if !crate::domain::conditions::field_ref_is_valid(entity_type, src, key, active_keys.iter().copied()) {
                    return Err(AppError::Validation(format!("'{key}' is not a valid field to compare against")));
                }
            }
            (None, None) => {}
            _ => return Err(AppError::Validation("A comparison field needs both a source and a key".into())),
        }
    }
    Ok(())
}

fn validate_actions(conn: &Connection, workspace_id: &str, entity_type: &str, actions: &[crate::models::workflow::WorkflowActionInput]) -> AppResult<()> {
    if actions.is_empty() {
        return Err(AppError::Validation("A workflow needs at least one action".into()));
    }
    for a in actions {
        if !ACTION_TYPES.contains(&a.action_type.as_str()) {
            return Err(AppError::Validation(format!("Invalid action type '{}'", a.action_type)));
        }
        parse_and_validate_params(conn, workspace_id, entity_type, &a.action_type, &a.params_json)?;
    }
    Ok(())
}

fn validate_shape(conn: &Connection, workspace_id: &str, entity_type: &str, input: &WorkflowDefinitionInput) -> AppResult<()> {
    require_valid_entity_type(conn, workspace_id, entity_type)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Workflow name is required".into()));
    }
    if !TRIGGER_TYPES.contains(&input.trigger_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid trigger type '{}'", input.trigger_type)));
    }
    match input.trigger_type.as_str() {
        "status_changed" => {
            let target = input.trigger_status.as_deref().unwrap_or("");
            if target.is_empty() {
                return Err(AppError::Validation("A target status/stage is required for this trigger".into()));
            }
            if !valid_transition_values_for(entity_type).contains(&target) {
                return Err(AppError::Validation(format!(
                    "'{target}' is not a valid {entity_type} {}", transition_field_for(entity_type)
                )));
            }
        }
        "date_reached" | "due_overdue" => {
            let field = input.trigger_field_key.as_deref().unwrap_or("");
            if !crate::models::workflow::date_fields_for(entity_type).contains(&field) {
                return Err(AppError::Validation(format!("'{field}' is not a date field this trigger can watch on {entity_type}")));
            }
        }
        "field_changed" => {
            let field = input.trigger_field_key.as_deref().unwrap_or("");
            if input.trigger_field_source == "builtin" {
                if builtin_fields::find_builtin_field(entity_type, field).is_none() {
                    return Err(AppError::Validation(format!("'{field}' is not a built-in field on {entity_type}")));
                }
            } else {
                let defs = custom_field_repo::list_definitions(conn, workspace_id, entity_type)?;
                if !defs.iter().any(|d| d.key == field && d.is_active) {
                    return Err(AppError::Validation(format!("'{field}' is not an active custom field to watch")));
                }
            }
        }
        "scheduled" => {
            if input.trigger_offset_days <= 0 {
                return Err(AppError::Validation("Scheduled workflows need a recurrence interval of at least 1 day".into()));
            }
        }
        _ => {}
    }
    if !crate::domain::conditions::MATCH_TYPES.contains(&input.match_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid match type '{}'", input.match_type)));
    }
    validate_conditions(conn, workspace_id, entity_type, &input.conditions)?;
    validate_actions(conn, workspace_id, entity_type, &input.actions)?;
    Ok(())
}

// --- CRUD ---------------------------------------------------------------

pub fn create_rule(conn: &Connection, workspace_id: &str, input: &WorkflowDefinitionInput, actor_user_id: Option<&str>) -> AppResult<WorkflowDefinition> {
    require_admin(conn, actor_user_id)?;
    validate_shape(conn, workspace_id, &input.entity_type, input)?;
    let id = new_uuid();
    Ok(workflow_repo::create(conn, &id, workspace_id, input, actor_user_id)?)
}

/// Only an Administrator can see (and manage) workflows - nothing
/// client-side evaluates these live, since they only take effect
/// server-side at the moment of a trigger.
pub fn list_rules(conn: &Connection, workspace_id: &str, entity_type: &str, actor_user_id: Option<&str>) -> AppResult<Vec<WorkflowDefinition>> {
    require_admin(conn, actor_user_id)?;
    Ok(workflow_repo::list(conn, workspace_id, entity_type)?)
}

pub fn update_rule(conn: &Connection, id: &str, input: &WorkflowDefinitionUpdate, actor_user_id: Option<&str>) -> AppResult<WorkflowDefinition> {
    require_admin(conn, actor_user_id)?;
    let existing = workflow_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Workflow".into()))?;
    if existing.is_protected {
        return Err(AppError::Validation("This workflow is protected by the system and cannot be modified".into()));
    }
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Workflow name is required".into()));
    }
    if !crate::domain::conditions::MATCH_TYPES.contains(&input.match_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid match type '{}'", input.match_type)));
    }
    validate_conditions(conn, &existing.workspace_id, &existing.entity_type, &input.conditions)?;
    validate_actions(conn, &existing.workspace_id, &existing.entity_type, &input.actions)?;
    Ok(workflow_repo::update(conn, id, input, actor_user_id)?)
}

pub fn list_runs(conn: &Connection, workspace_id: &str, workflow_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<crate::models::workflow::WorkflowRun>> {
    require_admin(conn, actor_user_id)?;
    Ok(workflow_repo::list_runs(conn, workspace_id, workflow_id)?)
}

// --- Trigger context / condition matching -------------------------------

fn build_trigger_context(conn: &Connection, entity_type: &str, entity_id: &str, override_status: Option<&str>) -> AppResult<HashMap<String, String>> {
    // ADM-WF "any field" targeting: every built-in field (not just the
    // transition field), merged with custom field values, so a condition
    // can target either kind identically - same shape business rules'
    // trigger context uses (custom_field_service::set_entity_values).
    let mut ctx = builtin_field_service::field_values(conn, entity_type, entity_id)?;
    for (k, v) in custom_field_repo::get_values(conn, entity_id)? {
        ctx.insert(k, v);
    }
    let builtin_key = transition_field_for(entity_type);
    let status = match override_status {
        Some(s) => s.to_string(),
        None => entity_registry::resolve(conn, entity_type, entity_id)?.map(|r| r.status).unwrap_or_default(),
    };
    ctx.insert(builtin_key.to_string(), status.clone());
    // Business rules' one built-in trigger field is always literally
    // "status"/"is_active" (never "stage") - mirrored here too so a
    // condition written against the generic built-in key still resolves
    // for entities where transition_field_for differs (Opportunity).
    ctx.entry(builtin_trigger_field_for(entity_type).to_string()).or_insert(status);
    Ok(ctx)
}

/// A condition's effective comparison value - see business_rule_service's
/// identical `resolve_condition_value` for the full explanation. Kept as a
/// separate copy (not shared) since it's typed against
/// `WorkflowCondition`, not `BusinessRuleCondition`.
fn resolve_condition_value<'a>(c: &'a crate::models::workflow::WorkflowCondition, ctx: &'a HashMap<String, String>) -> &'a str {
    match &c.compare_field_key {
        Some(key) => ctx.get(key).map(|s| s.as_str()).unwrap_or(""),
        None => c.value.as_str(),
    }
}

fn workflow_matches(wf: &WorkflowDefinition, ctx: &HashMap<String, String>) -> bool {
    if wf.conditions.is_empty() {
        return true; // a workflow with no extra conditions always fires on its trigger
    }
    conditions_match(
        &wf.match_type,
        wf.conditions.iter().map(|c| (c.group_id.as_deref(), c.field_key.as_str(), c.operator.as_str(), resolve_condition_value(c, ctx))),
        ctx,
    )
}

// --- record_created / record_updated / status_changed -------------------

/// Called by every entity service's create() (old_status: None) and
/// update() (old_status: Some(previous value)) - see this module's doc
/// comment. A no-op unless a DepthGuard is available (recursion bound).
pub fn fire_event(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
    entity_id: &str,
    old_status: Option<&str>,
    new_status: &str,
    fallback_owner_user_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<usize> {
    let Some(_guard) = DepthGuard::enter() else { return Ok(0) };
    let workflows = workflow_repo::list(conn, workspace_id, entity_type)?;
    let ctx = build_trigger_context(conn, entity_type, entity_id, Some(new_status))?;
    let mut fired = 0;

    for wf in workflows.iter().filter(|w| w.is_active) {
        let trigger_matches = match wf.trigger_type.as_str() {
            "record_created" => old_status.is_none(),
            "record_updated" => old_status.is_some(),
            "status_changed" => old_status.is_some_and(|old| old != new_status) && wf.trigger_status.as_deref() == Some(new_status),
            _ => false,
        };
        if !trigger_matches || !workflow_matches(wf, &ctx) {
            continue;
        }
        run_workflow(conn, workspace_id, wf, entity_type, entity_id, fallback_owner_user_id, actor_user_id)?;
        fired += 1;
    }
    Ok(fired)
}

/// Called by `custom_field_service::set_entity_values` after computing
/// which custom field values actually changed (`field_source: "custom"`),
/// and by every built-in entity service's `update()` after computing which
/// built-in fields actually changed (`field_source: "builtin"`) - fires
/// `field_changed` workflows watching any of them and matching that source.
/// The source distinction matters because a custom field and a built-in
/// field can share the same key (see `RuleEvaluation`'s doc comment for the
/// same concern on the business-rules side) - without it, a workflow
/// watching a builtin `notes` change could wrongly fire off a same-named
/// custom field edit, or vice versa.
pub fn fire_field_changed(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
    entity_id: &str,
    field_source: &str,
    changed_field_keys: &[String],
    fallback_owner_user_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<usize> {
    if changed_field_keys.is_empty() {
        return Ok(0);
    }
    let Some(_guard) = DepthGuard::enter() else { return Ok(0) };
    let workflows = workflow_repo::list(conn, workspace_id, entity_type)?;
    let ctx = build_trigger_context(conn, entity_type, entity_id, None)?;
    let mut fired = 0;

    for wf in workflows.iter().filter(|w| w.is_active && w.trigger_type == "field_changed" && w.trigger_field_source == field_source) {
        let watched = wf.trigger_field_key.as_deref().unwrap_or("");
        if !changed_field_keys.iter().any(|k| k == watched) || !workflow_matches(wf, &ctx) {
            continue;
        }
        run_workflow(conn, workspace_id, wf, entity_type, entity_id, fallback_owner_user_id, actor_user_id)?;
        fired += 1;
    }
    Ok(fired)
}

/// Diffs two built-in field snapshots (from `builtin_field_service::
/// field_values`, taken before and after an entity's own `update()` ran)
/// and returns which keys actually changed - the "did anything change"
/// check every entity's `update()` needs before calling `fire_field_changed`
/// with `field_source: "builtin"`, so a save that touches nothing observable
/// doesn't spuriously fire a workflow.
pub fn changed_builtin_keys(before: &HashMap<String, String>, after: &HashMap<String, String>) -> Vec<String> {
    after.iter().filter(|(k, v)| before.get(k.as_str()) != Some(v)).map(|(k, _)| k.clone()).collect()
}

// --- date_reached / due_overdue / scheduled -----------------------------

/// Personal Workspace has no OS-level background scheduler (spec ADM-WF-11
/// - "schedules execute while Lanesra is running; missed jobs are
/// evaluated on next service start"); this is the whole engine for that:
/// a scan the frontend calls periodically while the app is open, and once
/// at startup so anything missed while closed fires promptly. Team
/// Workspace's host service polling this on the same cadence covers
/// ADM-WF-12 without any code difference - the server dispatch layer just
/// needs a caller to invoke it on a timer, which is a deployment/ops
/// concern rather than an engine one.
pub fn run_scheduled(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<usize> {
    let Some(_guard) = DepthGuard::enter() else { return Ok(0) };
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let mut fired = 0;

    for wf in workflow_repo::list_active_by_trigger_types(conn, workspace_id, &["date_reached", "due_overdue"])? {
        let Some(field) = &wf.trigger_field_key else { continue };
        for entity_id in matching_date_records(conn, workspace_id, &wf.entity_type, field, &today, wf.trigger_offset_days)? {
            if workflow_repo::has_run_for_entity(conn, &wf.id, &entity_id)? {
                continue;
            }
            let ctx = build_trigger_context(conn, &wf.entity_type, &entity_id, None)?;
            if !workflow_matches(&wf, &ctx) {
                continue;
            }
            run_workflow(conn, workspace_id, &wf, &wf.entity_type, &entity_id, None, actor_user_id)?;
            fired += 1;
        }
    }

    for wf in workflow_repo::list_active_by_trigger_types(conn, workspace_id, &["scheduled"])? {
        let due = match &wf.last_scheduled_run_at {
            Some(last) => days_between(last, &today) >= wf.trigger_offset_days,
            None => true,
        };
        if !due {
            continue;
        }
        for entity_id in all_active_records(conn, workspace_id, &wf.entity_type)? {
            let ctx = build_trigger_context(conn, &wf.entity_type, &entity_id, None)?;
            if !workflow_matches(&wf, &ctx) {
                continue;
            }
            run_workflow(conn, workspace_id, &wf, &wf.entity_type, &entity_id, None, actor_user_id)?;
            fired += 1;
        }
        workflow_repo::set_last_scheduled_run(conn, &wf.id, &today)?;
    }

    Ok(fired)
}

fn days_between(from_iso_date: &str, to_iso_date: &str) -> i64 {
    match (
        chrono::NaiveDate::parse_from_str(&from_iso_date[..10.min(from_iso_date.len())], "%Y-%m-%d"),
        chrono::NaiveDate::parse_from_str(&to_iso_date[..10.min(to_iso_date.len())], "%Y-%m-%d"),
    ) {
        (Ok(from), Ok(to)) => (to - from).num_days(),
        _ => i64::MAX,
    }
}

/// IDs of non-archived `entity_type` records whose `date_field` is on or
/// before `today + offset_days` - the shared evaluation both date_reached
/// and due_overdue use (see this module's doc comment for why they're one
/// mechanism). Deliberately scoped to the small set of built-in entities
/// `date_fields_for` recognizes - custom objects have no date field of
/// their own yet, a documented gap rather than an oversight.
fn matching_date_records(conn: &Connection, workspace_id: &str, entity_type: &str, date_field: &str, today: &str, offset_days: i64) -> AppResult<Vec<String>> {
    let threshold = (Utc::now() - Duration::days(-offset_days)).format("%Y-%m-%d").to_string();
    let _ = today;
    let ids = match (entity_type, date_field) {
        ("Task", "due_date") => task_repo::list(conn, workspace_id)?
            .into_iter()
            .filter(|t| t.archived_at.is_none() && t.due_date.as_deref().is_some_and(|d| d <= threshold.as_str()))
            .map(|t| t.id)
            .collect(),
        ("Quote", "expiry_date") => crate::repositories::quote_repo::list(conn, workspace_id)?
            .into_iter()
            .filter(|q| q.archived_at.is_none() && q.expiry_date.as_deref().is_some_and(|d| d <= threshold.as_str()))
            .map(|q| q.id)
            .collect(),
        ("Contract", "end_date") => crate::repositories::contract_repo::list(conn, workspace_id)?
            .into_iter()
            .filter(|c| c.archived_at.is_none() && c.end_date.as_deref().is_some_and(|d| d <= threshold.as_str()))
            .map(|c| c.id)
            .collect(),
        ("Contract", "renewal_date") => crate::repositories::contract_repo::list(conn, workspace_id)?
            .into_iter()
            .filter(|c| c.archived_at.is_none() && c.renewal_date.as_deref().is_some_and(|d| d <= threshold.as_str()))
            .map(|c| c.id)
            .collect(),
        ("Invoice", "due_date") => crate::repositories::invoice_repo::list(conn, workspace_id)?
            .into_iter()
            .filter(|i| i.archived_at.is_none() && i.due_date.as_deref().is_some_and(|d| d <= threshold.as_str()))
            .map(|i| i.id)
            .collect(),
        _ => Vec::new(),
    };
    Ok(ids)
}

fn all_active_records(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<Vec<String>> {
    let ids = match entity_type {
        "Company" => company_repo::list(conn, workspace_id)?.into_iter().map(|r| r.id).collect(),
        "Opportunity" => opportunity_repo::list(conn, workspace_id)?.into_iter().map(|r| r.id).collect(),
        "Contract" => contract_repo::list(conn, workspace_id)?.into_iter().map(|r| r.id).collect(),
        "Task" => task_repo::list(conn, workspace_id)?.into_iter().map(|r| r.id).collect(),
        _ => custom_record_repo::list_all(conn, workspace_id, entity_type)?
            .into_iter()
            .filter(|r| r.archived_at.is_none())
            .map(|r| r.id)
            .collect(),
    };
    Ok(ids)
}

// --- Action execution -----------------------------------------------------

#[derive(Debug, Deserialize)]
struct CreateTaskParams {
    title: String,
    description: Option<String>,
    due_in_days: i64,
    assignee_user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateReminderParams {
    title: String,
    description: Option<String>,
    remind_in_days: i64,
    assignee_user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateFieldParams {
    target_field_key: String,
    #[serde(default = "default_field_source")]
    target_field_source: String,
    value: Option<String>,
    copy_from_field_key: Option<String>,
}

/// A target field's display label, for describe_action's test-mode
/// preview - falls back to the raw key if the field can't be resolved
/// (e.g. a stale key left behind by a deactivated custom field).
fn describe_field_label(conn: &Connection, workspace_id: &str, entity_type: &str, target_field_source: &str, target_field_key: &str) -> AppResult<String> {
    if target_field_source == "builtin" {
        Ok(builtin_fields::find_builtin_field(entity_type, target_field_key).map(|f| f.label.to_string()).unwrap_or_else(|| target_field_key.to_string()))
    } else {
        Ok(custom_field_repo::list_definitions(conn, workspace_id, entity_type)?
            .into_iter()
            .find(|d| d.key == target_field_key)
            .map(|d| d.label)
            .unwrap_or_else(|| target_field_key.to_string()))
    }
}

/// Shared by update_field/set_default_field/clear_field - all three target
/// a field on the trigger's own record the same way, differing only in
/// when/what they write at execution time.
fn validate_update_field_target(conn: &Connection, workspace_id: &str, entity_type: &str, target_field_source: &str, target_field_key: &str) -> AppResult<()> {
    if target_field_source == "builtin" {
        if !builtin_fields::is_actionable_builtin_field(entity_type, target_field_key) {
            return Err(AppError::Validation(format!("'{target_field_key}' is not an actionable built-in field on {entity_type}")));
        }
    } else {
        let defs = custom_field_repo::list_definitions(conn, workspace_id, entity_type)?;
        if !defs.iter().any(|d| d.key == target_field_key && d.is_active) {
            return Err(AppError::Validation(format!("'{target_field_key}' is not an active custom field to update")));
        }
    }
    Ok(())
}

fn default_field_source() -> String {
    "custom".to_string()
}

#[derive(Debug, Deserialize)]
struct AssignOwnerParams {
    user_id: Option<String>,
}

/// Addendum Phase 3 (§3.3 "generic create_record action"): generalizes the
/// original custom-object-only `create_related_record` action to any
/// entity type a workflow can safely construct with no per-type UI -
/// see `is_creatable_entity_type`'s doc comment for exactly which ones
/// and why. Linking is now optional (`relationship_definition_id: None`
/// creates a standalone record with nothing to link it to), where the
/// original action always required a relationship.
#[derive(Debug, Deserialize)]
struct CreateRecordParams {
    entity_type: String,
    #[serde(default)]
    relationship_definition_id: Option<String>,
    name_template: Option<String>,
}

/// Addendum Phase 3 (§3.3 "generic update_related_record action"): the
/// companion to `update_field` for reaching across a relationship - e.g.
/// "when an Opportunity is Won, set the linked Company's status to Active
/// Customer" - rather than only ever writing to the triggering record's
/// own fields. Applies to every record currently linked to the trigger
/// through the named relationship (more than one for a many_to_many
/// relationship), matching `update_field`'s `target_field_source`/
/// `copy_from_field_key` shape so the two actions read the same way in
/// the UI. `copy_from_field_key`, when set, reads from the *triggering*
/// record (the natural "roll this value up/down the relationship"
/// reading), not from the record(s) being written to.
#[derive(Debug, Deserialize)]
struct UpdateRelatedRecordParams {
    relationship_definition_id: String,
    target_field_key: String,
    #[serde(default = "default_field_source")]
    target_field_source: String,
    value: Option<String>,
    copy_from_field_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AddNotificationParams {
    message: String,
    audience: String,
}

fn parse_params<T: for<'de> Deserialize<'de>>(action_type: &str, params_json: &str) -> AppResult<T> {
    serde_json::from_str(params_json).map_err(|e| AppError::Validation(format!("Invalid parameters for '{action_type}': {e}")))
}

fn entity_supports_owner(entity_type: &str) -> bool {
    matches!(entity_type, "Company" | "Opportunity" | "Contract" | "Task") || !CORE_WORKFLOW_ENTITY_TYPES.contains(&entity_type)
}

/// Which entity types `create_record` can construct with nothing but a
/// name - deliberately narrow. Every core entity besides Company has a
/// required relational or line-item field (Contact needs `company_id`,
/// Opportunity/Quote/Order need a company and, for Quote/Order, at least
/// one line) that a no-code action can't safely synthesize without its
/// own dedicated per-type UI - out of scope here. Task/Reminder already
/// have dedicated actions for the common "auto-create a task" case. Any
/// active custom object qualifies unconditionally, same as the original
/// `create_related_record` action.
fn is_creatable_entity_type(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<bool> {
    if entity_type == "Company" {
        return Ok(true);
    }
    Ok(custom_object_service::get_by_key(conn, workspace_id, entity_type)?.is_some_and(|d| d.is_active))
}

/// The other side of `relationship_definition_id` from `entity_type` -
/// shared by `create_record`'s optional link target and
/// `update_related_record`'s required one. Errors rather than guessing
/// when `entity_type` isn't actually one of the definition's two sides.
fn other_side_of_relationship(def: &crate::models::relationship::RelationshipDefinition, entity_type: &str) -> AppResult<String> {
    if def.source_entity_type == entity_type {
        Ok(def.target_entity_type.clone())
    } else if def.target_entity_type == entity_type {
        Ok(def.source_entity_type.clone())
    } else {
        Err(AppError::Validation("Selected relationship does not connect to this record type".into()))
    }
}

/// Validates an action's params without executing anything - used both by
/// `validate_actions` at save time and indirectly documents each action's
/// expected `params_json` shape.
fn parse_and_validate_params(conn: &Connection, workspace_id: &str, entity_type: &str, action_type: &str, params_json: &str) -> AppResult<()> {
    match action_type {
        "create_task" => {
            let p: CreateTaskParams = parse_params(action_type, params_json)?;
            if p.title.trim().is_empty() {
                return Err(AppError::Validation("Task title is required".into()));
            }
        }
        "create_reminder" => {
            let p: CreateReminderParams = parse_params(action_type, params_json)?;
            if p.title.trim().is_empty() {
                return Err(AppError::Validation("Reminder title is required".into()));
            }
        }
        "update_field" => {
            let p: UpdateFieldParams = parse_params(action_type, params_json)?;
            validate_update_field_target(conn, workspace_id, entity_type, &p.target_field_source, &p.target_field_key)?;
            if p.value.is_none() && p.copy_from_field_key.is_none() {
                return Err(AppError::Validation("Provide a value or a field to copy from".into()));
            }
        }
        // Second addendum round: the trigger-time counterparts to
        // update_field - "only if currently empty" (mirrors business
        // rules' set_default) and "always write empty" respectively. Same
        // params shape as update_field; a value/copy-from field is
        // meaningless for clear_field, so that check is skipped there.
        "set_default_field" => {
            let p: UpdateFieldParams = parse_params(action_type, params_json)?;
            validate_update_field_target(conn, workspace_id, entity_type, &p.target_field_source, &p.target_field_key)?;
            if p.value.is_none() && p.copy_from_field_key.is_none() {
                return Err(AppError::Validation("Provide a value or a field to copy from".into()));
            }
        }
        "clear_field" => {
            let p: UpdateFieldParams = parse_params(action_type, params_json)?;
            validate_update_field_target(conn, workspace_id, entity_type, &p.target_field_source, &p.target_field_key)?;
        }
        "assign_owner" => {
            let _p: AssignOwnerParams = parse_params(action_type, params_json)?;
            if !entity_supports_owner(entity_type) {
                return Err(AppError::Validation(format!("{entity_type} has no owner field to assign")));
            }
        }
        "create_record" => {
            let p: CreateRecordParams = parse_params(action_type, params_json)?;
            if !is_creatable_entity_type(conn, workspace_id, &p.entity_type)? {
                return Err(AppError::Validation(format!(
                    "'{}' cannot be created by a workflow action - only Company and active custom objects are supported", p.entity_type
                )));
            }
            if let Some(rel_id) = &p.relationship_definition_id {
                let def = relationship_repo::get_definition(conn, rel_id)?.ok_or_else(|| AppError::Validation("Selected relationship does not exist".into()))?;
                let other = other_side_of_relationship(&def, entity_type)?;
                if other != p.entity_type {
                    return Err(AppError::Validation("Selected relationship does not connect these two record types".into()));
                }
            }
        }
        "update_related_record" => {
            let p: UpdateRelatedRecordParams = parse_params(action_type, params_json)?;
            let def = relationship_repo::get_definition(conn, &p.relationship_definition_id)?.ok_or_else(|| AppError::Validation("Selected relationship does not exist".into()))?;
            let other_type = other_side_of_relationship(&def, entity_type)?;
            if p.target_field_source == "builtin" {
                if !builtin_fields::is_actionable_builtin_field(&other_type, &p.target_field_key) {
                    return Err(AppError::Validation(format!("'{}' is not an actionable built-in field on {other_type}", p.target_field_key)));
                }
            } else {
                let defs = custom_field_repo::list_definitions(conn, workspace_id, &other_type)?;
                if !defs.iter().any(|d| d.key == p.target_field_key && d.is_active) {
                    return Err(AppError::Validation(format!("'{}' is not an active custom field on {other_type}", p.target_field_key)));
                }
            }
            if p.value.is_none() && p.copy_from_field_key.is_none() {
                return Err(AppError::Validation("Provide a value or a field to copy from".into()));
            }
        }
        "add_notification" => {
            let p: AddNotificationParams = parse_params(action_type, params_json)?;
            if p.message.trim().is_empty() {
                return Err(AppError::Validation("Notification message is required".into()));
            }
            if !NOTIFICATION_AUDIENCES.contains(&p.audience.as_str()) {
                return Err(AppError::Validation(format!("Invalid notification audience '{}'", p.audience)));
            }
        }
        _ => return Err(AppError::Validation(format!("Unknown action type '{action_type}'"))),
    }
    Ok(())
}

fn resolve_owner_fallback(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<Option<String>> {
    let owner = match entity_type {
        "Company" => company_repo::get(conn, entity_id)?.and_then(|r| r.owner_user_id),
        "Contact" => None,
        "Opportunity" => opportunity_repo::get(conn, entity_id)?.and_then(|r| r.owner_user_id),
        "Contract" => contract_repo::get(conn, entity_id)?.and_then(|r| r.owner_user_id),
        "Task" => task_repo::get(conn, entity_id)?.and_then(|r| r.owner_user_id),
        "Quote" => match crate::repositories::quote_repo::get(conn, entity_id)? {
            Some(q) => company_repo::get(conn, &q.company_id)?.and_then(|c| c.owner_user_id),
            None => None,
        },
        "Order" => match crate::repositories::order_repo::get(conn, entity_id)? {
            Some(o) => company_repo::get(conn, &o.company_id)?.and_then(|c| c.owner_user_id),
            None => None,
        },
        "Invoice" => match crate::repositories::invoice_repo::get(conn, entity_id)? {
            Some(i) => company_repo::get(conn, &i.company_id)?.and_then(|c| c.owner_user_id),
            None => None,
        },
        _ => custom_record_repo::get(conn, entity_id)?.filter(|r| r.object_key == entity_type).and_then(|r| r.owner_user_id),
    };
    Ok(owner)
}

fn admin_user_ids(conn: &Connection, workspace_id: &str) -> AppResult<Vec<String>> {
    Ok(crate::services::user_service::list(conn, workspace_id)?
        .into_iter()
        .filter(|u| u.roles.iter().any(|r| r == "Administrator"))
        .map(|u| u.id)
        .collect())
}

/// The effective value an update_field/set_default_field action would
/// write: a fixed value, or the current value of another field on this
/// same record when copy_from_field_key is set. The field being copied
/// *from* isn't source-tagged (no second field_source param, to keep the
/// action shape simple) - read both custom values and every built-in
/// field's current value and take whichever has that key.
fn resolve_update_field_value(conn: &Connection, entity_type: &str, entity_id: &str, p: &UpdateFieldParams) -> AppResult<String> {
    if let Some(src) = &p.copy_from_field_key {
        let mut ctx = builtin_field_service::field_values(conn, entity_type, entity_id)?;
        for (k, v) in custom_field_repo::get_values(conn, entity_id)? {
            ctx.insert(k, v);
        }
        Ok(ctx.get(src).cloned().unwrap_or_default())
    } else {
        Ok(p.value.clone().unwrap_or_default())
    }
}

/// Writes `value` to the target field (built-in or custom) and returns its
/// display label, for the run summary. Shared by update_field/
/// set_default_field/clear_field - they differ only in whether/when they
/// call this, not in how the write itself happens.
fn write_update_field_value(
    conn: &Connection, workspace_id: &str, entity_type: &str, entity_id: &str,
    target_field_source: &str, target_field_key: &str, value: &str, actor_user_id: Option<&str>,
) -> AppResult<String> {
    if target_field_source == "builtin" {
        builtin_field_service::set_field(conn, workspace_id, entity_type, entity_id, target_field_key, value, actor_user_id)?;
        Ok(target_field_key.to_string())
    } else {
        let defs = custom_field_repo::list_definitions(conn, workspace_id, entity_type)?;
        let def = defs
            .iter()
            .find(|d| d.key == target_field_key && d.is_active)
            .ok_or_else(|| AppError::Validation(format!("'{target_field_key}' is not an active custom field")))?;
        custom_field_repo::set_value(conn, &def.id, entity_id, value)?;
        Ok(def.label.clone())
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_action(
    conn: &Connection,
    workspace_id: &str,
    action_type: &str,
    params_json: &str,
    entity_type: &str,
    entity_id: &str,
    fallback_owner_user_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<String> {
    match action_type {
        "create_task" => {
            let p: CreateTaskParams = parse_params(action_type, params_json)?;
            let assignee = p.assignee_user_id.clone().or_else(|| fallback_owner_user_id.map(String::from));
            let due_date = (Utc::now() + Duration::days(p.due_in_days)).format("%Y-%m-%d").to_string();
            let task = task_service::create(
                conn, workspace_id,
                &TaskInput {
                    title: p.title.clone(), description: p.description, owner_user_id: assignee, priority: "Normal".into(),
                    status: "Not Started".into(), due_date: Some(due_date), reminder_at: None,
                    related_type: Some(entity_type.to_string()), related_id: Some(entity_id.to_string()),
                },
                actor_user_id,
            )?;
            Ok(format!("created task '{}'", task.title))
        }
        "create_reminder" => {
            let p: CreateReminderParams = parse_params(action_type, params_json)?;
            let assignee = p.assignee_user_id.clone().or_else(|| fallback_owner_user_id.map(String::from));
            let remind_at = (Utc::now() + Duration::days(p.remind_in_days)).to_rfc3339();
            let task = task_service::create(
                conn, workspace_id,
                &TaskInput {
                    title: p.title.clone(), description: p.description, owner_user_id: assignee, priority: "Normal".into(),
                    status: "Not Started".into(), due_date: None, reminder_at: Some(remind_at),
                    related_type: Some(entity_type.to_string()), related_id: Some(entity_id.to_string()),
                },
                actor_user_id,
            )?;
            Ok(format!("created reminder '{}'", task.title))
        }
        "update_field" => {
            let p: UpdateFieldParams = parse_params(action_type, params_json)?;
            let value = resolve_update_field_value(conn, entity_type, entity_id, &p)?;
            let label = write_update_field_value(conn, workspace_id, entity_type, entity_id, &p.target_field_source, &p.target_field_key, &value, actor_user_id)?;
            Ok(format!("set {label} = \"{value}\""))
        }
        // Second addendum round: "only if currently empty" - mirrors
        // business rules' set_default action, just applied at workflow
        // trigger time to a field on the record that just triggered it.
        "set_default_field" => {
            let p: UpdateFieldParams = parse_params(action_type, params_json)?;
            let mut ctx = builtin_field_service::field_values(conn, entity_type, entity_id)?;
            for (k, v) in custom_field_repo::get_values(conn, entity_id)? {
                ctx.insert(k, v);
            }
            let currently_empty = ctx.get(&p.target_field_key).map(|s| s.trim().is_empty()).unwrap_or(true);
            if !currently_empty {
                return Ok(format!("{} already has a value - left unchanged", p.target_field_key));
            }
            let value = resolve_update_field_value(conn, entity_type, entity_id, &p)?;
            let label = write_update_field_value(conn, workspace_id, entity_type, entity_id, &p.target_field_source, &p.target_field_key, &value, actor_user_id)?;
            Ok(format!("set default {label} = \"{value}\""))
        }
        // Always writes empty, unconditionally - the trigger-time
        // counterpart to business rules' clear_value action.
        "clear_field" => {
            let p: UpdateFieldParams = parse_params(action_type, params_json)?;
            let label = write_update_field_value(conn, workspace_id, entity_type, entity_id, &p.target_field_source, &p.target_field_key, "", actor_user_id)?;
            Ok(format!("cleared {label}"))
        }
        "assign_owner" => {
            let p: AssignOwnerParams = parse_params(action_type, params_json)?;
            match entity_type {
                "Company" => company_repo::set_owner(conn, entity_id, p.user_id.as_deref())?,
                "Opportunity" => opportunity_repo::set_owner(conn, entity_id, p.user_id.as_deref())?,
                "Contract" => contract_repo::set_owner(conn, entity_id, p.user_id.as_deref())?,
                "Task" => task_repo::set_owner(conn, entity_id, p.user_id.as_deref())?,
                _ => custom_record_repo::set_owner(conn, entity_id, p.user_id.as_deref())?,
            }
            Ok("assigned owner".into())
        }
        "create_record" => {
            let p: CreateRecordParams = parse_params(action_type, params_json)?;
            let source = entity_registry::resolve(conn, entity_type, entity_id)?
                .ok_or_else(|| AppError::Validation("Triggering record no longer exists".into()))?;
            let name = p.name_template.clone().unwrap_or_else(|| format!("Related to {}", source.display_name));
            let is_custom_object = custom_object_service::get_by_key(conn, workspace_id, &p.entity_type)?.is_some_and(|d| d.is_active);
            let new_id = if is_custom_object {
                custom_record_service::create(
                    conn, workspace_id,
                    &crate::models::custom_record::CustomRecordInput { object_key: p.entity_type.clone(), primary_name: name.clone(), status: "Active".into(), owner_user_id: None, notes: None },
                    actor_user_id,
                )?.id
            } else if p.entity_type == "Company" {
                company_service::create(
                    conn, workspace_id,
                    &crate::models::company::CompanyInput {
                        name: name.clone(), status: "Prospect".into(), owner_user_id: None, tax_number: None,
                        billing_address: None, shipping_address: None, tags: None, notes: None,
                        ..Default::default()
                    },
                    actor_user_id,
                )?.id
            } else {
                return Err(AppError::Validation(format!("'{}' cannot be created by a workflow action", p.entity_type)));
            };
            if let Some(rel_id) = &p.relationship_definition_id {
                let def = relationship_repo::get_definition(conn, rel_id)?
                    .ok_or_else(|| AppError::Validation("Selected relationship does not exist".into()))?;
                let (source_type, source_id, target_type, target_id) = if def.source_entity_type == entity_type {
                    (entity_type, entity_id.to_string(), p.entity_type.as_str(), new_id.clone())
                } else {
                    (p.entity_type.as_str(), new_id.clone(), entity_type, entity_id.to_string())
                };
                crate::services::relationship_service::link(conn, workspace_id, &def.id, source_type, &source_id, target_type, &target_id, actor_user_id)?;
            }
            Ok(format!("created {} '{}'", p.entity_type, name))
        }
        "update_related_record" => {
            let p: UpdateRelatedRecordParams = parse_params(action_type, params_json)?;
            let def = relationship_repo::get_definition(conn, &p.relationship_definition_id)?
                .ok_or_else(|| AppError::Validation("Selected relationship does not exist".into()))?;
            let (other_type, related_ids): (String, Vec<String>) = if def.source_entity_type == entity_type {
                (def.target_entity_type.clone(), relationship_repo::list_instances_where_source(conn, &def.id, entity_id)?.into_iter().map(|i| i.target_id).collect())
            } else {
                (def.source_entity_type.clone(), relationship_repo::list_instances_where_target(conn, &def.id, entity_id)?.into_iter().map(|i| i.source_id).collect())
            };
            if related_ids.is_empty() {
                return Ok(format!("no linked {other_type} record to update"));
            }
            let value = if let Some(src) = &p.copy_from_field_key {
                // Reads from the *triggering* record (not the related
                // record being written to) - see UpdateRelatedRecordParams'
                // doc comment for why that's the intuitive direction.
                let mut ctx = builtin_field_service::field_values(conn, entity_type, entity_id)?;
                for (k, v) in custom_field_repo::get_values(conn, entity_id)? {
                    ctx.insert(k, v);
                }
                ctx.get(src).cloned().unwrap_or_default()
            } else {
                p.value.clone().unwrap_or_default()
            };
            for related_id in &related_ids {
                if p.target_field_source == "builtin" {
                    builtin_field_service::set_field(conn, workspace_id, &other_type, related_id, &p.target_field_key, &value, actor_user_id)?;
                } else {
                    let defs = custom_field_repo::list_definitions(conn, workspace_id, &other_type)?;
                    let def2 = defs.iter().find(|d| d.key == p.target_field_key && d.is_active)
                        .ok_or_else(|| AppError::Validation(format!("'{}' is not an active custom field on {other_type}", p.target_field_key)))?;
                    custom_field_repo::set_value(conn, &def2.id, related_id, &value)?;
                }
            }
            Ok(format!("set {} on {} linked {other_type} record(s)", p.target_field_key, related_ids.len()))
        }
        "add_notification" => {
            let p: AddNotificationParams = parse_params(action_type, params_json)?;
            if p.audience == "all_admins" {
                for admin_id in admin_user_ids(conn, workspace_id)? {
                    notification_repo::create(conn, workspace_id, Some(&admin_id), &p.message, Some(entity_type), Some(entity_id))?;
                }
            } else {
                let owner = resolve_owner_fallback(conn, entity_type, entity_id)?.or_else(|| fallback_owner_user_id.map(String::from));
                if let Some(user_id) = owner {
                    notification_repo::create(conn, workspace_id, Some(&user_id), &p.message, Some(entity_type), Some(entity_id))?;
                }
            }
            Ok("sent notification".into())
        }
        other => Err(AppError::Validation(format!("Unknown action type '{other}'"))),
    }
}

fn run_workflow(
    conn: &Connection,
    workspace_id: &str,
    wf: &WorkflowDefinition,
    entity_type: &str,
    entity_id: &str,
    fallback_owner_user_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    let mut summaries = Vec::new();
    let mut errors = Vec::new();
    for action in &wf.actions {
        match apply_action(conn, workspace_id, &action.action_type, &action.params_json, entity_type, entity_id, fallback_owner_user_id, actor_user_id) {
            Ok(summary) => summaries.push(summary),
            Err(e) => errors.push(e.to_string()),
        }
    }
    let outcome = if errors.is_empty() { "success" } else { "error" };
    let error_message = if errors.is_empty() { None } else { Some(errors.join("; ")) };
    workflow_repo::record_run(
        conn, workspace_id, &wf.id, entity_type, Some(entity_id), &wf.trigger_type, outcome,
        Some(&summaries.join("; ")), error_message.as_deref(),
    )?;
    Ok(())
}

// --- Test / simulation mode --------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowTestMatch {
    pub workflow_id: String,
    pub workflow_name: String,
    pub trigger_type: String,
    pub action_descriptions: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct WorkflowTestResult {
    pub matches: Vec<WorkflowTestMatch>,
}

/// Addendum Phase 3 (mirrors `business_rule_service::test_rules`): lets an
/// admin try a hypothetical field context against every active workflow
/// for an entity type without executing anything. Like business rules'
/// own test mode, this only simulates the *conditions* half - which
/// real-world event (create/update/status change/etc.) actually fired is
/// not part of the simulation, since the hypothetical context has no
/// "before" state to diff against. Actions are described in plain
/// language via `describe_action` rather than run through `apply_action`,
/// so nothing is written - no task created, no field changed, no
/// notification sent.
pub fn test_workflows(conn: &Connection, workspace_id: &str, entity_type: &str, ctx: &HashMap<String, String>, actor_user_id: Option<&str>) -> AppResult<WorkflowTestResult> {
    require_admin(conn, actor_user_id)?;
    let workflows = workflow_repo::list(conn, workspace_id, entity_type)?;
    let mut matches = Vec::new();
    for wf in workflows.iter().filter(|w| w.is_active) {
        if !workflow_matches(wf, ctx) {
            continue;
        }
        let mut action_descriptions = Vec::new();
        for action in &wf.actions {
            action_descriptions.push(describe_action(conn, workspace_id, entity_type, &action.action_type, &action.params_json)?);
        }
        matches.push(WorkflowTestMatch { workflow_id: wf.id.clone(), workflow_name: wf.name.clone(), trigger_type: wf.trigger_type.clone(), action_descriptions });
    }
    Ok(WorkflowTestResult { matches })
}

/// Read-only mirror of `apply_action` for `test_workflows` - parses the
/// same params and resolves the same labels, but never writes anything
/// (no task/record creation, no field writes, no notifications sent, no
/// relationship links).
fn describe_action(conn: &Connection, workspace_id: &str, entity_type: &str, action_type: &str, params_json: &str) -> AppResult<String> {
    match action_type {
        "create_task" => {
            let p: CreateTaskParams = parse_params(action_type, params_json)?;
            Ok(format!("would create task \"{}\"", p.title))
        }
        "create_reminder" => {
            let p: CreateReminderParams = parse_params(action_type, params_json)?;
            Ok(format!("would create reminder \"{}\"", p.title))
        }
        "update_field" => {
            let p: UpdateFieldParams = parse_params(action_type, params_json)?;
            let label = describe_field_label(conn, workspace_id, entity_type, &p.target_field_source, &p.target_field_key)?;
            match (&p.value, &p.copy_from_field_key) {
                (Some(v), _) => Ok(format!("would set {label} = \"{v}\"")),
                (None, Some(src)) => Ok(format!("would set {label} = value copied from '{src}'")),
                (None, None) => Ok(format!("would set {label}")),
            }
        }
        "set_default_field" => {
            let p: UpdateFieldParams = parse_params(action_type, params_json)?;
            let label = describe_field_label(conn, workspace_id, entity_type, &p.target_field_source, &p.target_field_key)?;
            Ok(format!("would set default {label} (only if currently empty)"))
        }
        "clear_field" => {
            let p: UpdateFieldParams = parse_params(action_type, params_json)?;
            let label = describe_field_label(conn, workspace_id, entity_type, &p.target_field_source, &p.target_field_key)?;
            Ok(format!("would clear {label}"))
        }
        "assign_owner" => {
            let p: AssignOwnerParams = parse_params(action_type, params_json)?;
            match &p.user_id {
                Some(uid) => {
                    let name = user_repo::find_by_id(conn, uid)?.map(|u| u.display_name).unwrap_or_else(|| "that user".into());
                    Ok(format!("would assign owner to {name}"))
                }
                None => Ok("would clear the owner".into()),
            }
        }
        "create_record" => {
            let p: CreateRecordParams = parse_params(action_type, params_json)?;
            Ok(format!("would create a new {}", p.entity_type))
        }
        "update_related_record" => {
            let p: UpdateRelatedRecordParams = parse_params(action_type, params_json)?;
            let rel_label = relationship_repo::get_definition(conn, &p.relationship_definition_id)?
                .map(|d| d.forward_label)
                .unwrap_or_else(|| "related".into());
            Ok(format!("would set {} on linked {rel_label} record(s)", p.target_field_key))
        }
        "add_notification" => {
            let p: AddNotificationParams = parse_params(action_type, params_json)?;
            let audience = if p.audience == "all_admins" { "all admins" } else { "the owner" };
            Ok(format!("would notify {audience}: \"{}\"", p.message))
        }
        other => Err(AppError::Validation(format!("Unknown action type '{other}'"))),
    }
}
