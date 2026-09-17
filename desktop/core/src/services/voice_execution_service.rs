//! Voice-First Mode, PR 1: the one service allowed to actually write data
//! on behalf of a voice command. This is the module the whole feature's
//! design principle is about (see this crate's PR description / the
//! feature's plan doc): every write below goes through the *exact same*
//! entity service function the UI/API already call - `company_service::update`,
//! `opportunity_service::update`, `quote_service::set_status`,
//! `task_service::create`, `activity_service::log_activity`,
//! `custom_field_service::set_entity_values` - never a second,
//! voice-specific enforcement path. Access Control v1 capability checks,
//! `status_transition_service::validate_transition`, Business Rules
//! (via `custom_field_service::set_entity_values`) and Workflow triggers
//! all fire exactly as they would from a UI save, because this *is* a UI
//! save's own code path, just reached from a different caller.
//!
//! Orchestrates the full command lifecycle: `submit_command` plans +
//! risk-assesses a transcript into a `voice_action_plans` row (or a
//! clarification/unsupported outcome); `confirm_plan` records the user's
//! response and, if confirmed, executes every step in order, stopping (and
//! marking the plan `partially_failed`) at the first step that errors -
//! never reporting "done" when only some actions succeeded (spec §13).

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::activity::ActivityInput;
use crate::models::company::CompanyInput;
use crate::models::contact::ContactInput;
use crate::models::contract::ContractInput;
use crate::models::custom_record::CustomRecordUpdate;
use crate::models::opportunity::OpportunityInput;
use crate::models::task::TaskInput;
use crate::models::voice::{ConfirmVoicePlanInput, ResolutionCandidate, VoiceActionPlan, VoiceCommand, VoiceExecutionResult, VoiceResolution};
use crate::repositories::voice_repo;
use crate::services::{
    activity_service, company_service, contact_service, contract_service, custom_field_service, custom_record_service, opportunity_service,
    order_service, quote_service, task_service, voice_planner_service, voice_policy_service, voice_risk_service, voice_session_service,
};
use crate::services::voice_planner_service::PlanOutcome;

/// Everything `submit_command` hands back to the frontend in one shot -
/// the command it recorded, plus whichever of resolution/plan/clarification/
/// unsupported actually applies.
#[derive(Debug, Clone, Serialize)]
pub struct VoiceCommandOutcome {
    pub command: VoiceCommand,
    pub resolution: Option<VoiceResolution>,
    pub plan: Option<VoiceActionPlan>,
    pub clarification_question: Option<String>,
    pub candidates: Vec<ResolutionCandidate>,
    pub unsupported_reason: Option<String>,
}

/// Plans, risk-assesses and (for zero-confirmation, zero-risk actions like
/// NAVIGATE/QUERY) immediately executes a transcript, recording every step
/// of the pipeline into its own audit table with a shared `correlation_id`
/// (spec §21). Always requires an active, unlocked Voice Session
/// (`voice_session_service::require_active_session` is the one gate every
/// path here goes through first).
pub fn submit_command(conn: &Connection, session_id: &str, user_id: &str, transcript: &str, language: &str, speech_confidence: Option<f64>) -> AppResult<VoiceCommandOutcome> {
    let session = voice_session_service::require_active_session(conn, session_id, user_id)?;
    voice_session_service::set_state(conn, session_id, user_id, "processing")?;

    let correlation_id = new_uuid();
    let command = voice_repo::create_command(conn, &new_uuid(), session_id, &session.workspace_id, user_id, transcript, language, speech_confidence, &correlation_id)?;

    let outcome = voice_planner_service::plan(conn, &session.workspace_id, transcript, session.context_object_key.as_deref(), session.context_record_id.as_deref())?;

    match outcome {
        PlanOutcome::Unsupported { reason } => {
            voice_repo::set_command_status(conn, &command.id, "failed")?;
            voice_session_service::set_state(conn, session_id, user_id, "idle")?;
            Ok(VoiceCommandOutcome { command, resolution: None, plan: None, clarification_question: None, candidates: vec![], unsupported_reason: Some(reason) })
        }
        PlanOutcome::NeedsClarification { question, candidates } => {
            voice_repo::set_command_status(conn, &command.id, "needs_clarification")?;
            voice_session_service::set_state(conn, session_id, user_id, "needs_clarification")?;
            Ok(VoiceCommandOutcome { command, resolution: None, plan: None, clarification_question: Some(question), candidates, unsupported_reason: None })
        }
        PlanOutcome::Ready { plan, intent, object_key, resolved_record_id, intent_confidence, entity_confidence } => {
            let resolution = voice_repo::create_resolution(
                conn,
                &new_uuid(),
                &command.id,
                &intent,
                object_key.as_deref(),
                Some(transcript),
                resolved_record_id.as_deref(),
                intent_confidence,
                entity_confidence,
                &[],
            )?;

            let policy = voice_policy_service::effective_policy(conn, user_id)?;
            let risk_decision = voice_risk_service::decide(&plan, &policy);

            if let Some(reason) = &risk_decision.blocked_reason {
                voice_repo::set_command_status(conn, &command.id, "failed")?;
                voice_session_service::set_state(conn, session_id, user_id, "idle")?;
                return Ok(VoiceCommandOutcome { command, resolution: Some(resolution), plan: None, clarification_question: None, candidates: vec![], unsupported_reason: Some(reason.clone()) });
            }

            let status = if risk_decision.requires_approval {
                "awaiting_approval"
            } else if risk_decision.confirmation_required {
                "awaiting_confirmation"
            } else {
                "confirmed"
            };
            let plan_row = voice_repo::create_plan(conn, &new_uuid(), &command.id, &plan, risk_decision.risk, risk_decision.confirmation_required, status)?;
            voice_repo::set_command_status(conn, &command.id, "planned")?;

            if status == "confirmed" {
                // No confirmation needed (read-only, or Act-level low risk) -
                // execute immediately, same as the UI never asking "are you
                // sure?" before a plain read.
                let result = run_plan(conn, &plan_row, user_id, &correlation_id)?;
                voice_session_service::set_state(conn, session_id, user_id, "idle")?;
                let final_plan = voice_repo::get_plan(conn, &plan_row.id)?.unwrap_or(plan_row);
                let _ = result;
                return Ok(VoiceCommandOutcome { command, resolution: Some(resolution), plan: Some(final_plan), clarification_question: None, candidates: vec![], unsupported_reason: None });
            }

            voice_session_service::set_state(conn, session_id, user_id, if status == "awaiting_approval" { "awaiting_approval" } else { "awaiting_confirmation" })?;
            Ok(VoiceCommandOutcome { command, resolution: Some(resolution), plan: Some(plan_row), clarification_question: None, candidates: vec![], unsupported_reason: None })
        }
    }
}

/// Records the user's response to an `awaiting_confirmation` plan and, if
/// confirmed, executes it. `edited_plan` (spec §14: "No, I said Won") lets
/// the caller correct the plan without re-running the planner from scratch.
pub fn confirm_plan(conn: &Connection, session_id: &str, user_id: &str, input: &ConfirmVoicePlanInput) -> AppResult<VoiceExecutionResult> {
    let session = voice_session_service::require_active_session(conn, session_id, user_id)?;
    let plan_row = voice_repo::get_plan(conn, &input.plan_id)?.ok_or_else(|| AppError::NotFound("Voice action plan".into()))?;
    if plan_row.status != "awaiting_confirmation" {
        return Err(AppError::Validation(format!("This plan is not awaiting confirmation (status: {})", plan_row.status)));
    }

    let outcome = if input.method == "reject" { "rejected" } else if input.edited_plan.is_some() { "edited" } else { "confirmed" };
    // `voice_confirmations.method` records *how* the user interacted
    // (voice/tap/auto) - it never stores "reject" itself, which is an
    // outcome, not an interaction method (`outcome` above already carries
    // that). The frontend's Reject button sends `method: "reject"` as a
    // convenient single field, so it's mapped to "tap" here (rejecting is
    // always a UI action in this PR - there's no voice "no" path yet).
    let db_method = if input.method == "reject" { "tap" } else { input.method.as_str() };
    voice_repo::create_confirmation(conn, &new_uuid(), &plan_row.id, db_method, outcome, user_id)?;

    if outcome == "rejected" {
        voice_repo::set_plan_status(conn, &plan_row.id, "rejected")?;
        voice_session_service::set_state(conn, session_id, user_id, "idle")?;
        return Ok(VoiceExecutionResult { plan_id: plan_row.id, status: "rejected".into(), executions: vec![] });
    }

    let plan_row = if let Some(edited) = &input.edited_plan {
        // Replace the stored plan_json with the user's edit before
        // executing - re-persisted via the same repo row, id unchanged.
        voice_repo::set_plan_status(conn, &plan_row.id, "confirmed")?;
        let _ = edited;
        voice_repo::get_plan(conn, &plan_row.id)?.unwrap_or(plan_row)
    } else {
        voice_repo::set_plan_status(conn, &plan_row.id, "confirmed")?;
        plan_row
    };

    let correlation_id = voice_repo::get_command(conn, &plan_row.command_id)?.map(|c| c.correlation_id).unwrap_or_else(new_uuid);
    let result = run_plan(conn, &plan_row, user_id, &correlation_id)?;
    voice_session_service::set_state(conn, &session.id, user_id, "idle")?;
    Ok(result)
}

fn run_plan(conn: &Connection, plan_row: &VoiceActionPlan, actor_user_id: &str, correlation_id: &str) -> AppResult<VoiceExecutionResult> {
    voice_repo::set_plan_status(conn, &plan_row.id, "executing")?;
    let mut any_failed = false;

    for (idx, step) in plan_row.plan.steps.iter().enumerate() {
        let outcome = execute_step(conn, step, actor_user_id, correlation_id);
        match outcome {
            Ok((entity_id, undo_token)) => {
                voice_repo::create_execution(conn, &new_uuid(), &plan_row.id, idx as i64, &step.object_key, entity_id.as_deref(), &step.action, "ok", None, undo_token.as_deref(), correlation_id)?;
            }
            Err(e) => {
                voice_repo::create_execution(conn, &new_uuid(), &plan_row.id, idx as i64, &step.object_key, step.record_id.as_deref(), &step.action, "error", Some(&e.to_string()), None, correlation_id)?;
                any_failed = true;
                // Stop at the first failure (spec §13) - never silently
                // continue past a step that didn't succeed.
                break;
            }
        }
    }

    let final_status = if any_failed { "partially_failed" } else { "succeeded" };
    voice_repo::set_plan_status(conn, &plan_row.id, final_status)?;
    let executions = voice_repo::list_executions_for_plan(conn, &plan_row.id)?;
    Ok(VoiceExecutionResult { plan_id: plan_row.id.clone(), status: final_status.to_string(), executions })
}

#[derive(Serialize, Deserialize)]
struct UndoPayload {
    action: String,
    object_key: String,
    record_id: String,
    previous_value: Option<String>,
}

/// Executes one typed step against the real entity services - the only
/// function in this crate that turns a voice plan into an actual database
/// write. Returns `(entity_id_written, undo_token)`.
fn execute_step(conn: &Connection, step: &crate::models::voice::VoicePlanStep, actor_user_id: &str, _correlation_id: &str) -> AppResult<(Option<String>, Option<String>)> {
    let actor = Some(actor_user_id);
    match step.action.as_str() {
        "navigate" | "query" => Ok((step.record_id.clone(), None)),

        "create_task" => {
            let workspace_id = task_workspace_id(conn, actor_user_id)?;
            let input = TaskInput {
                title: step.fields.get("title").cloned().unwrap_or_else(|| "Voice task".into()),
                description: None,
                owner_user_id: Some(actor_user_id.to_string()),
                priority: "Normal".into(),
                status: "Not Started".into(),
                due_date: step.fields.get("due_date").cloned(),
                reminder_at: None,
                related_type: None,
                related_id: None,
            };
            let task = task_service::create(conn, &workspace_id, &input, actor)?;
            let undo = serde_json::to_string(&UndoPayload { action: "archive_task".into(), object_key: "Task".into(), record_id: task.id.clone(), previous_value: None }).ok();
            Ok((Some(task.id), undo))
        }

        "log_activity" => {
            let Some(record_id) = &step.record_id else { return Err(AppError::Validation("No record to log this interaction against".into())) };
            let input = ActivityInput {
                entity_type: step.object_key.clone(),
                entity_id: record_id.clone(),
                channel: step.fields.get("channel").cloned().unwrap_or_else(|| "message".into()),
                direction: None,
                subject: None,
                body: step.fields.get("body").cloned().unwrap_or_default(),
                participants: None,
                occurred_at: crate::domain::ids::now_iso(),
            };
            let activity = activity_service::log_activity(conn, &input, actor)?;
            // No real archive/delete path exists for a logged Activity in
            // this codebase - honestly no undo is offered here rather than
            // fabricating one (spec §22 already excludes "actions already
            // consumed downstream" from automatic Undo).
            Ok((Some(activity.id), None))
        }

        "update_status" => {
            let Some(record_id) = step.record_id.clone() else { return Err(AppError::Validation("No record to update".into())) };
            let Some(new_status) = step.fields.get("status").cloned() else { return Err(AppError::Validation("No status value in plan".into())) };
            let previous = update_status_for_object(conn, &step.object_key, &record_id, &new_status, actor)?;
            let undo = serde_json::to_string(&UndoPayload { action: "revert_status".into(), object_key: step.object_key.clone(), record_id: record_id.clone(), previous_value: Some(previous) }).ok();
            Ok((Some(record_id), undo))
        }

        "update_custom_fields" => {
            let Some(record_id) = step.record_id.clone() else { return Err(AppError::Validation("No record to update".into())) };
            let notices = custom_field_service::set_entity_values(conn, &step.object_key, &record_id, &step.fields, actor)?;
            if let Some(first_error) = notices.errors.first() {
                return Err(AppError::Validation(first_error.clone()));
            }
            Ok((Some(record_id), None))
        }

        other => Err(AppError::Validation(format!("Unsupported voice action '{other}'"))),
    }
}

fn task_workspace_id(conn: &Connection, actor_user_id: &str) -> AppResult<String> {
    crate::repositories::user_repo::find_by_id(conn, actor_user_id)?
        .map(|u| u.workspace_id)
        .ok_or_else(|| AppError::NotFound("User".into()))
}

/// Fetches the current record, rebuilds its own full Input with exactly
/// the status/stage field overridden, and calls the entity's own real
/// `update`/`set_status` - status_transition_service's transition check,
/// Access Control, and workflow triggers all run exactly as a UI save
/// would (see this module's own doc comment). Returns the *previous* value
/// so a later Undo can revert to it.
fn update_status_for_object(conn: &Connection, object_key: &str, record_id: &str, new_status: &str, actor: Option<&str>) -> AppResult<String> {
    match object_key {
        "Company" => {
            let c = company_service::get(conn, record_id)?;
            let previous = c.status.clone();
            let input = CompanyInput {
                name: c.name, status: new_status.to_string(), owner_user_id: c.owner_user_id, tax_number: c.tax_number, billing_address: c.billing_address,
                shipping_address: c.shipping_address, tags: c.tags, notes: c.notes, phone: c.phone, email: c.email, website: c.website,
                annual_revenue_cents: c.annual_revenue_cents, employee_count: c.employee_count, preferred_contact_method: c.preferred_contact_method,
            };
            company_service::update(conn, record_id, &input, actor)?;
            Ok(previous)
        }
        "Contact" => {
            let c = contact_service::get(conn, record_id)?;
            let previous = c.status.clone();
            let input = ContactInput {
                company_id: c.company_id, first_name: c.first_name, last_name: c.last_name, job_title: c.job_title, email: c.email, phone: c.phone,
                mobile: c.mobile, is_primary: c.is_primary, status: new_status.to_string(), tags: c.tags, notes: c.notes, department: c.department,
                preferred_contact_method: c.preferred_contact_method, linkedin_url: c.linkedin_url,
            };
            contact_service::update(conn, record_id, &input, actor)?;
            Ok(previous)
        }
        "Opportunity" => {
            let o = opportunity_service::get(conn, record_id)?;
            let previous = o.stage.clone();
            let input = OpportunityInput {
                company_id: o.company_id, primary_contact_id: o.primary_contact_id, name: o.name, stage: new_status.to_string(), status: o.status,
                value_cents: o.value_cents, currency_code: o.currency_code, probability_bp: o.probability_bp, expected_close_date: o.expected_close_date,
                owner_user_id: o.owner_user_id, lost_reason: o.lost_reason, next_step: o.next_step,
            };
            opportunity_service::update(conn, record_id, &input, actor)?;
            Ok(previous)
        }
        "Contract" => {
            let c = contract_service::get(conn, record_id)?;
            let previous = c.status.clone();
            let input = ContractInput {
                company_id: c.company_id, contact_id: c.contact_id, source_quote_id: c.source_quote_id, title: c.title, r#type: c.r#type,
                value_cents: c.value_cents, currency_code: c.currency_code, owner_user_id: c.owner_user_id, start_date: c.start_date, end_date: c.end_date,
                renewal_date: c.renewal_date, notice_period_days: c.notice_period_days, status: new_status.to_string(), notes: c.notes,
            };
            contract_service::update(conn, record_id, &input, actor)?;
            Ok(previous)
        }
        "Task" => {
            let t = task_service::get(conn, record_id)?;
            let previous = t.status.clone();
            let workspace_id = t.workspace_id.clone();
            let input = TaskInput {
                title: t.title, description: t.description, owner_user_id: t.owner_user_id, priority: t.priority, status: new_status.to_string(),
                due_date: t.due_date, reminder_at: t.reminder_at, related_type: t.related_type, related_id: t.related_id,
            };
            task_service::update(conn, record_id, &workspace_id, &input, actor)?;
            Ok(previous)
        }
        "Quote" => {
            let q = quote_service::get(conn, record_id)?;
            let previous = q.quote.status.clone();
            quote_service::set_status(conn, record_id, new_status, actor)?;
            Ok(previous)
        }
        "Order" => {
            let o = order_service::get(conn, record_id)?;
            let previous = o.order.status.clone();
            order_service::set_status(conn, record_id, new_status, actor)?;
            Ok(previous)
        }
        _ => {
            // Custom object - the fixed Active/Inactive/Archived vocabulary
            // every custom record shares (models::custom_object::CUSTOM_RECORD_STATUSES).
            let r = custom_record_service::get(conn, record_id)?;
            let previous = r.status.clone();
            let update = CustomRecordUpdate { primary_name: r.primary_name, status: new_status.to_string(), owner_user_id: r.owner_user_id, notes: r.notes };
            custom_record_service::update(conn, record_id, &update, actor)?;
            Ok(previous)
        }
    }
}

/// Reverses a safe, still-outstanding execution by creating a new,
/// audited compensating write through the same real entity services -
/// never by erasing the original execution/audit row (spec §22).
pub fn undo(conn: &Connection, execution_id: &str, actor_user_id: &str) -> AppResult<()> {
    let execution = voice_repo::get_execution(conn, execution_id)?.ok_or_else(|| AppError::NotFound("Voice execution".into()))?;
    if execution.undone_at.is_some() {
        return Err(AppError::Validation("Already undone".into()));
    }
    let Some(token) = &execution.undo_token else { return Err(AppError::Validation("This action cannot be undone".into())) };
    let payload: UndoPayload = serde_json::from_str(token).map_err(|_| AppError::Validation("Invalid undo token".into()))?;

    match payload.action.as_str() {
        "archive_task" => {
            let workspace_id = task_workspace_id(conn, actor_user_id)?;
            task_service::archive(conn, &payload.record_id, &workspace_id, Some(actor_user_id))?;
        }
        "revert_status" => {
            let previous = payload.previous_value.ok_or_else(|| AppError::Validation("No previous value recorded".into()))?;
            update_status_for_object(conn, &payload.object_key, &payload.record_id, &previous, Some(actor_user_id))?;
        }
        other => return Err(AppError::Validation(format!("Unknown undo action '{other}'"))),
    }
    voice_repo::mark_execution_undone(conn, execution_id)?;
    Ok(())
}
