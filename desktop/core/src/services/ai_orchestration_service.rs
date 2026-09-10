//! AI & Agentic Layer, Phase 6b: orchestration on top of Phase 6a's
//! Agents - Pipeline/Trigger CRUD (Administrator-only, same shape every
//! other admin builder here uses), and the actual execution engine both
//! a manual "Run Now" and a drained schedule/webhook/workflow-fired run
//! go through.
//!
//! **Gating**: Pipeline/Trigger CRUD and `run_manual` (used by the
//! admin-action route and the admin chat assistant's own
//! `run_ai_agent`/`run_ai_agent_pipeline` tools) all require
//! Administrator up front - Orchestration itself is an admin-only
//! capability. The underlying `run_internal` execution has **no gate of
//! its own** beyond what each step's own agent already enforces
//! (`chat_service::agent_requires_admin`) - deliberately, since a
//! drained schedule/webhook/workflow-triggered run has no human actor at
//! all. In practice this means an admin-scoped agent or pipeline step
//! simply can't run unattended unless `actor` genuinely resolves to a
//! real Administrator (e.g. a webhook call authenticated by an API
//! client whose `owner_user_id` is one) - record-scoped-only automation
//! is the safe default, not an oversight.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::ai_agent_pipeline::{AiAgentPipeline, AiAgentPipelineInput, AiAgentRun, AiAgentTrigger, AiAgentTriggerInput, TRIGGER_TARGET_TYPES, TRIGGER_TYPES};
use crate::repositories::{ai_agent_pending_run_repo, ai_agent_pipeline_repo, ai_agent_repo, ai_agent_run_repo};
use crate::services::chat_service;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn validate_pipeline_input(conn: &Connection, workspace_id: &str, input: &AiAgentPipelineInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Pipeline name is required".into()));
    }
    if input.steps.is_empty() {
        return Err(AppError::Validation("A pipeline needs at least one step".into()));
    }
    for step in &input.steps {
        let agent = ai_agent_repo::get(conn, &step.agent_id)?.ok_or_else(|| AppError::Validation("Selected agent does not exist".into()))?;
        if agent.workspace_id != workspace_id || !agent.is_active {
            return Err(AppError::Validation("Selected agent does not exist".into()));
        }
    }
    Ok(())
}

pub fn create_pipeline(conn: &Connection, workspace_id: &str, input: &AiAgentPipelineInput, actor_user_id: Option<&str>) -> AppResult<AiAgentPipeline> {
    require_admin(conn, actor_user_id)?;
    validate_pipeline_input(conn, workspace_id, input)?;
    let id = new_uuid();
    Ok(ai_agent_pipeline_repo::create(conn, &id, workspace_id, input, actor_user_id)?)
}

pub fn update_pipeline(conn: &Connection, id: &str, workspace_id: &str, input: &AiAgentPipelineInput, actor_user_id: Option<&str>) -> AppResult<AiAgentPipeline> {
    require_admin(conn, actor_user_id)?;
    ai_agent_pipeline_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Pipeline".into()))?;
    validate_pipeline_input(conn, workspace_id, input)?;
    Ok(ai_agent_pipeline_repo::update(conn, id, input, actor_user_id)?)
}

pub fn get_pipeline(conn: &Connection, id: &str) -> AppResult<Option<AiAgentPipeline>> {
    Ok(ai_agent_pipeline_repo::get(conn, id)?)
}

pub fn list_pipelines(conn: &Connection, workspace_id: &str, active_only: bool) -> AppResult<Vec<AiAgentPipeline>> {
    Ok(ai_agent_pipeline_repo::list(conn, workspace_id, active_only)?)
}

pub fn set_pipeline_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> AppResult<AiAgentPipeline> {
    require_admin(conn, actor_user_id)?;
    ai_agent_pipeline_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Pipeline".into()))?;
    Ok(ai_agent_pipeline_repo::set_active(conn, id, is_active, actor_user_id)?)
}

// --- Triggers ---------------------------------------------------------

fn validate_target_exists(conn: &Connection, workspace_id: &str, target_type: &str, target_id: &str) -> AppResult<()> {
    if !TRIGGER_TARGET_TYPES.contains(&target_type) {
        return Err(AppError::Validation(format!("Invalid target type '{target_type}'")));
    }
    let exists = match target_type {
        "agent" => ai_agent_repo::get(conn, target_id)?.map(|a| a.workspace_id == workspace_id).unwrap_or(false),
        "pipeline" => ai_agent_pipeline_repo::get(conn, target_id)?.map(|p| p.workspace_id == workspace_id).unwrap_or(false),
        _ => false,
    };
    if !exists {
        return Err(AppError::Validation(format!("The selected {target_type} does not exist")));
    }
    Ok(())
}

pub fn create_trigger(conn: &Connection, workspace_id: &str, input: &AiAgentTriggerInput, actor_user_id: Option<&str>) -> AppResult<AiAgentTrigger> {
    require_admin(conn, actor_user_id)?;
    if !TRIGGER_TYPES.contains(&input.trigger_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid trigger type '{}'", input.trigger_type)));
    }
    validate_target_exists(conn, workspace_id, &input.target_type, &input.target_id)?;
    if input.trigger_type == "schedule" && input.interval_minutes.unwrap_or(0) < 1 {
        return Err(AppError::Validation("A schedule trigger needs an interval of at least 1 minute".into()));
    }
    let id = new_uuid();
    Ok(ai_agent_pipeline_repo::create_trigger(conn, &id, workspace_id, input, actor_user_id)?)
}

pub fn list_triggers(conn: &Connection, target_type: &str, target_id: &str) -> AppResult<Vec<AiAgentTrigger>> {
    Ok(ai_agent_pipeline_repo::list_triggers_for_target(conn, target_type, target_id)?)
}

pub fn set_trigger_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_agent_pipeline_repo::set_trigger_active(conn, id, is_active)?)
}

pub fn delete_trigger(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_agent_pipeline_repo::delete_trigger(conn, id)?)
}

// --- Execution ----------------------------------------------------------

const PREVIOUS_OUTPUT_PLACEHOLDER: &str = "{{previous_output}}";
const TRIGGER_INPUT_PLACEHOLDER: &str = "{{trigger_input}}";

fn resolve_template(template: &str, previous_output: &str, trigger_input: &str) -> String {
    template.replace(PREVIOUS_OUTPUT_PLACEHOLDER, previous_output).replace(TRIGGER_INPUT_PLACEHOLDER, trigger_input)
}

/// A manual "Run Now" (the admin action route, and the admin chat
/// assistant's own `run_ai_agent`/`run_ai_agent_pipeline` tools) -
/// Administrator-gated up front, then delegates to `run_internal`.
#[allow(clippy::too_many_arguments)]
pub async fn run_manual(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], target_type: &str, target_id: &str, input_text: &str, actor_user_id: Option<&str>) -> AppResult<AiAgentRun> {
    require_admin(conn, actor_user_id)?;
    run_internal(conn, workspace_id, master_key, target_type, target_id, actor_user_id, input_text, Some("manual"), None, None).await
}

/// The entry point for a run with no human actor clicking anything - the
/// inline webhook Trigger route (`server/src/agent_v1.rs`) and
/// `drain_pending_runs` below both call this directly, skipping
/// `run_manual`'s own top-level admin gate. See this module's own doc
/// comment for why: each step's own agent-level gating
/// (`chat_service::agent_requires_admin`) is still fully enforced by
/// `run_internal`, just with no blanket "must be an Administrator" check
/// on top of it.
#[allow(clippy::too_many_arguments)]
pub async fn run_triggered(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    target_type: &str,
    target_id: &str,
    actor: Option<&str>,
    trigger_input: &str,
    triggered_by: Option<&str>,
    source_entity_type: Option<&str>,
    source_entity_id: Option<&str>,
) -> AppResult<AiAgentRun> {
    run_internal(conn, workspace_id, master_key, target_type, target_id, actor, trigger_input, triggered_by, source_entity_type, source_entity_id).await
}

/// The execution engine itself - runs a lone Agent (a 1-step run) or a
/// Pipeline in order, writing one `ai_agent_runs` row plus one
/// `ai_agent_run_steps` row per step. See this module's own doc comment
/// for why this has no admin gate of its own.
#[allow(clippy::too_many_arguments)]
async fn run_internal(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    target_type: &str,
    target_id: &str,
    actor: Option<&str>,
    trigger_input: &str,
    triggered_by: Option<&str>,
    source_entity_type: Option<&str>,
    source_entity_id: Option<&str>,
) -> AppResult<AiAgentRun> {
    let steps: Vec<(String, String)> = match target_type {
        "agent" => {
            let agent = ai_agent_repo::get(conn, target_id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
            vec![(agent.id, TRIGGER_INPUT_PLACEHOLDER.to_string())]
        }
        "pipeline" => {
            let pipeline = ai_agent_pipeline_repo::get(conn, target_id)?.ok_or_else(|| AppError::NotFound("Pipeline".into()))?;
            if pipeline.steps.is_empty() {
                return Err(AppError::Validation("This pipeline has no steps".into()));
            }
            pipeline.steps.into_iter().map(|s| (s.agent_id, s.input_template)).collect()
        }
        other => return Err(AppError::Validation(format!("Unknown target type '{other}'"))),
    };

    let run_id = new_uuid();
    ai_agent_run_repo::start_run(conn, &run_id, workspace_id, target_type, target_id, triggered_by, source_entity_type, source_entity_id)?;

    let mut previous_output = String::new();
    let mut overall_error: Option<String> = None;
    for (step_order, (agent_id, template)) in steps.into_iter().enumerate() {
        let input_text = resolve_template(&template, &previous_output, trigger_input);
        let agent = match ai_agent_repo::get(conn, &agent_id)? {
            Some(a) => a,
            None => {
                let msg = format!("Step {} names an agent that no longer exists", step_order + 1);
                ai_agent_run_repo::append_run_step(conn, &run_id, &agent_id, step_order as i64, &input_text, None, Some(&msg), 0)?;
                overall_error = Some(msg);
                break;
            }
        };
        match chat_service::run_agent_once_with_text(conn, workspace_id, master_key, actor, &agent, &input_text).await {
            Ok(outcome) => {
                let tool_calls = outcome.produced.iter().filter(|m| m.role == "tool").count() as i64;
                ai_agent_run_repo::append_run_step(conn, &run_id, &agent_id, step_order as i64, &input_text, Some(&outcome.final_text), None, tool_calls)?;
                previous_output = outcome.final_text;
            }
            Err(e) => {
                let msg = e.to_string();
                ai_agent_run_repo::append_run_step(conn, &run_id, &agent_id, step_order as i64, &input_text, None, Some(&msg), 0)?;
                overall_error = Some(msg);
                break;
            }
        }
    }

    let status = if overall_error.is_some() { "failed" } else { "succeeded" };
    ai_agent_run_repo::finish_run(conn, &run_id, status, overall_error.as_deref())?;
    Ok(ai_agent_run_repo::get_run(conn, &run_id)?.expect("just finished"))
}

/// The one enqueue point for a genuinely-async trigger firing from a
/// context that must never block on it - a due schedule (`enqueue_due_
/// schedules` below), the new Workflow Automation action, or (were an
/// inbound trigger ever changed to defer instead of run inline) a
/// webhook. Resolves `{{trigger_input}}` immediately (from whatever the
/// caller already built) - `run_internal` still resolves
/// `{{previous_output}}` per step when this drains.
#[allow(clippy::too_many_arguments)]
pub fn enqueue(conn: &Connection, workspace_id: &str, target_type: &str, target_id: &str, resolved_input_text: &str, triggered_by: Option<&str>, source_entity_type: Option<&str>, source_entity_id: Option<&str>) -> AppResult<()> {
    let id = new_uuid();
    Ok(ai_agent_pending_run_repo::enqueue(conn, &id, workspace_id, target_type, target_id, resolved_input_text, triggered_by, source_entity_type, source_entity_id)?)
}

/// Finds schedule Triggers whose interval has elapsed and enqueues one
/// pending run each - called once per `job_scheduler` tick, right before
/// `drain_pending_runs`, so a newly-due schedule still drains in the
/// same tick it becomes due rather than waiting a full extra interval.
pub fn enqueue_due_schedules(conn: &Connection, workspace_id: &str) -> AppResult<usize> {
    let due = ai_agent_pipeline_repo::list_due_schedule_triggers(conn, workspace_id)?;
    let count = due.len();
    for trigger in &due {
        enqueue(conn, workspace_id, &trigger.target_type, &trigger.target_id, "", Some("schedule"), None, None)?;
        ai_agent_pipeline_repo::mark_trigger_run(conn, &trigger.id)?;
    }
    Ok(count)
}

/// The async drain for everything `enqueue`/`enqueue_due_schedules`
/// queued - called from `job_scheduler.rs`'s tick (server) and a
/// matching desktop poll, same "enqueue now, drain later" shape
/// `connector_execution_service::drain_pending_actions` already
/// established for `call_connector_action`. A single item's failure
/// (a bad target, a provider error) is recorded on its own
/// `ai_agent_runs` row via `run_internal`'s own error handling and never
/// aborts the batch.
pub async fn drain_pending_runs(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], limit: i64) -> AppResult<usize> {
    let batch = ai_agent_pending_run_repo::list_batch(conn, workspace_id, limit)?;
    let mut drained = 0;
    for item in &batch {
        let _ = run_internal(
            conn, workspace_id, master_key, &item.target_type, &item.target_id, None, &item.resolved_input_text,
            item.triggered_by.as_deref(), item.source_entity_type.as_deref(), item.source_entity_id.as_deref(),
        )
        .await;
        ai_agent_pending_run_repo::delete(conn, &item.id)?;
        drained += 1;
    }
    Ok(drained)
}

pub fn list_runs(conn: &Connection, target_type: &str, target_id: &str, limit: i64) -> AppResult<Vec<AiAgentRun>> {
    Ok(ai_agent_run_repo::list_runs_for_target(conn, target_type, target_id, limit)?)
}
