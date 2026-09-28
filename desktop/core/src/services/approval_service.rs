//! AI Agent Platform v2, Phase 1: create/list/resolve durable `ai_approvals`
//! rows - see migration `0059_ai_agent_versioning.sql`'s own doc comment
//! for why `subject_type`/`subject_id` name what's pending rather than a
//! column-per-kind, and that same comment for why this isn't yet wired to
//! `ai_agent_runs.awaiting_approval` (later-phase work, not this one).
//! `agent_version_service::transition_status` doesn't call into this file
//! today - Publish is a direct admin action, not itself gated behind an
//! approval in Phase 1 - but a Draft/Test agent publish is exactly the
//! kind of thing `subject_type = "agent_version_publish"` exists for a
//! caller to route through this service by hand (see the seeded example
//! in `tests/ai_agent_versioning.rs`).

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::ai_approval::{AiApproval, AiApprovalInput, AiApprovalResolution};
use crate::repositories::ai_approval_repo;

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
pub fn create(conn: &Connection, workspace_id: &str, input: &AiApprovalInput, requested_by: Option<&str>) -> AppResult<AiApproval> {
    validate_input(input)?;
    Ok(ai_approval_repo::create(conn, workspace_id, input, requested_by)?)
}

pub fn get(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<AiApproval> {
    require_admin(conn, actor_user_id)?;
    let approval = ai_approval_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Approval".into()))?;
    if approval.workspace_id != workspace_id {
        return Err(AppError::NotFound("Approval".into()));
    }
    Ok(approval)
}

/// `status` narrows to `pending`/`approved`/`rejected`; `None` lists every
/// approval regardless of outcome (a full audit view).
pub fn list(conn: &Connection, workspace_id: &str, status: Option<&str>, actor_user_id: Option<&str>) -> AppResult<Vec<AiApproval>> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_approval_repo::list(conn, workspace_id, status)?)
}

/// Approve or reject a pending approval - resolving one that's already
/// been resolved is rejected outright (no re-resolving), so `resolved_by`/
/// `resolved_at` always describe the one decision that actually stuck.
pub fn resolve(conn: &Connection, id: &str, workspace_id: &str, resolution: &AiApprovalResolution, actor_user_id: Option<&str>) -> AppResult<AiApproval> {
    require_admin(conn, actor_user_id)?;
    let approval = ai_approval_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Approval".into()))?;
    if approval.workspace_id != workspace_id {
        return Err(AppError::NotFound("Approval".into()));
    }
    if approval.status != "pending" {
        return Err(AppError::Validation(format!("This approval was already {}", approval.status)));
    }
    let new_status = if resolution.approve { "approved" } else { "rejected" };
    Ok(ai_approval_repo::resolve(conn, id, new_status, actor_user_id, resolution.resolution_notes.as_deref())?)
}
