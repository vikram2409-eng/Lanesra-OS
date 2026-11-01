//! AI Agent Platform v2, Phase 1: create/list/resolve durable `ai_approvals`
//! rows - see migration `0059_ai_agent_versioning.sql` for why
//! `subject_type`/`subject_id` name what's pending rather than a
//! column-per-kind.
//!
//! WP-0150: every pause for a person is one of these rows - a pipeline step
//! that needs approval (`pipeline_run`), an execution-graph Approval node,
//! a tool call the policy held (`tool_call`) and a voice plan that needs
//! approval (`voice_plan`) - and `decide` is the one way to decide them: it
//! resolves the row and resumes whatever waits on it. `resolve` alone is
//! for the services that already own the resume (`graph_runtime_service`,
//! `ai_orchestration_service`) and for subjects nothing waits on.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::ai_approval::{AiApproval, AiApprovalInput, AiApprovalResolution};
use crate::repositories::{ai_agent_run_repo, ai_approval_repo, graph_run_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn validate_input(input: &AiApprovalInput) -> AppResult<()> {
    if input.subject_type.trim().is_empty() {
        return Err(AppError::Validation("subject_type is required".into()));
    }
    if input.subject_id.trim().is_empty() {
        return Err(AppError::Validation("subject_id is required".into()));
    }
    Ok(())
}

/// Any authenticated actor can request an approval - it's whoever resolves
/// it (an Administrator) that's gated, not who can ask for one, the same
/// "requesting isn't the sensitive part, granting is" reasoning
/// `voice_execution_service`'s own risk-gated actions already follow.
pub fn create(
    conn: &Connection,
    workspace_id: &str,
    input: &AiApprovalInput,
    requested_by: Option<&str>,
) -> AppResult<AiApproval> {
    validate_input(input)?;
    Ok(ai_approval_repo::create(
        conn,
        workspace_id,
        input,
        requested_by,
    )?)
}

pub fn get(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<AiApproval> {
    require_admin(conn, actor_user_id)?;
    let approval =
        ai_approval_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Approval".into()))?;
    if approval.workspace_id != workspace_id {
        return Err(AppError::NotFound("Approval".into()));
    }
    Ok(approval)
}

/// `status` narrows to `pending`/`approved`/`rejected`; `None` lists every
/// approval regardless of outcome (a full audit view).
pub fn list(
    conn: &Connection,
    workspace_id: &str,
    status: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<AiApproval>> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_approval_repo::list(conn, workspace_id, status)?)
}

/// Approve or reject a pending approval - resolving one that's already
/// been resolved is rejected outright (no re-resolving), so `resolved_by`/
/// `resolved_at` always describe the one decision that actually stuck.
pub fn resolve(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    resolution: &AiApprovalResolution,
    actor_user_id: Option<&str>,
) -> AppResult<AiApproval> {
    require_admin(conn, actor_user_id)?;
    let approval =
        ai_approval_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Approval".into()))?;
    if approval.workspace_id != workspace_id {
        return Err(AppError::NotFound("Approval".into()));
    }
    if approval.status != "pending" {
        return Err(AppError::Validation(format!(
            "This approval was already {}",
            approval.status
        )));
    }
    let new_status = if resolution.approve {
        "approved"
    } else {
        "rejected"
    };
    Ok(ai_approval_repo::resolve(
        conn,
        id,
        new_status,
        actor_user_id,
        resolution.resolution_notes.as_deref(),
    )?)
}

/// WP-0150: the one way a person decides an approval, from any screen. It
/// resolves the approval and resumes whatever waits on it:
///
/// - an execution-graph run, at its Approval node or at an agent node whose
///   tool call was held (`graph_runtime_service::resolve_approval`);
/// - a pipeline run, at a step that needs approval or a held tool call
///   (`approve_pending_step` re-runs or continues it; rejecting ends it);
/// - a voice plan (back to its speaker to confirm, or rejected);
/// - anything else (a held call from the Assistant chat, a version
///   publish): resolved; an approved tool call then runs once when the
///   same call is made again.
pub async fn decide(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    master_key: &[u8; 32],
    resolution: &AiApprovalResolution,
    actor_user_id: Option<&str>,
) -> AppResult<AiApproval> {
    let approval = get(conn, id, workspace_id, actor_user_id)?;
    if approval.status != "pending" {
        return Err(AppError::Validation(format!(
            "This approval was already {}",
            approval.status
        )));
    }
    let notes = resolution.resolution_notes.as_deref();
    if let Some(run_id) = graph_run_repo::run_waiting_on(conn, workspace_id, id)? {
        super::graph_runtime_service::resolve_approval(
            conn,
            workspace_id,
            master_key,
            &run_id,
            resolution.approve,
            notes,
            actor_user_id,
        )
        .await?;
    } else if let Some(run_id) = ai_agent_run_repo::run_waiting_on(conn, workspace_id, id)? {
        if resolution.approve {
            super::ai_orchestration_service::approve_pending_step(
                conn,
                workspace_id,
                master_key,
                &run_id,
                None,
                actor_user_id,
            )
            .await?;
        } else {
            super::ai_orchestration_service::reject_pending_run(
                conn,
                &run_id,
                notes.unwrap_or("Rejected in the approvals inbox"),
                actor_user_id,
            )?;
        }
    } else {
        resolve(conn, id, workspace_id, resolution, actor_user_id)?;
        if approval.subject_type == "voice_plan" {
            super::voice_execution_service::decide_plan(
                conn,
                &approval.subject_id,
                resolution.approve,
            )?;
        }
    }
    get(conn, id, workspace_id, actor_user_id)
}

/// The synchronous resolve the Team Workspace command dispatcher offers: it
/// refuses an approval that a run or voice plan waits on, since resolving
/// that one without resuming it would leave the run stuck (RAID I-13);
/// those are decided with `decide`.
pub fn resolve_unless_waiting(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    resolution: &AiApprovalResolution,
    actor_user_id: Option<&str>,
) -> AppResult<AiApproval> {
    let approval = get(conn, id, workspace_id, actor_user_id)?;
    let waiting = graph_run_repo::run_waiting_on(conn, workspace_id, id)?.is_some()
        || ai_agent_run_repo::run_waiting_on(conn, workspace_id, id)?.is_some()
        || approval.subject_type == "voice_plan";
    if waiting {
        return Err(AppError::Validation(
            "This approval resumes a paused run or plan; decide it from the approvals inbox".into(),
        ));
    }
    resolve(conn, id, workspace_id, resolution, actor_user_id)
}
