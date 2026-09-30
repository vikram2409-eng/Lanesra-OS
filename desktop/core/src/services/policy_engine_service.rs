//! AI Agent Platform v2, Phase 2: a unified Policy Engine - the decision
//! `chat_service`'s Tool-Call Firewall (`execute_tool`/`execute_agent_tool`)
//! checks before either of the two real dispatchers
//! (`dispatch_record_tool`/`dispatch_admin_tool`, or
//! `connector_tool_service::dispatch`) ever runs. This is a
//! planning/governance layer *in front of* those dispatchers, never a
//! second enforcement path - the same non-negotiable principle every prior
//! phase of this codebase (Voice-First Mode, Access Control v1) is built
//! on. A tool call this evaluates as `Allow` still goes through the exact
//! same dispatcher, with the exact same admin-gating and validation, as it
//! always has.
//!
//! **v1 scope, named honestly**: `RequireApproval` does not pause and
//! later resume the calling chat/agent run - `chat_service::send_message`
//! and `run_agent_once` have no such mechanism today (unlike a Pipeline
//! step, which already can via `ai_orchestration_service`'s own
//! pause/approve/reject). Instead, a `RequireApproval` tool call is not
//! executed at all: a durable `ai_approvals` row is created for the
//! record (`subject_type = "tool_call"`), and the caller gets back a
//! clear "this action requires administrator approval and one has been
//! requested" result instead of the tool's real output. Wiring this into
//! an actual pause-and-resume of the calling run is real, named follow-up
//! work, not silently skipped.

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::domain::AppResult;
use crate::models::ai_agent_policy::{AiAgentPolicy, AiAgentPolicyInput, PolicyDecision};
use crate::models::ai_approval::AiApprovalInput;
use crate::models::ai_tool_registry::RiskLevel;
use crate::repositories::ai_agent_policy_repo;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// The policy that actually governs a call for this `agent_id` - the
/// agent-specific policy if one exists, else the workspace-wide default,
/// else `None` (no policy configured anywhere in this workspace - every
/// call Allowed, today's exact behavior, unchanged).
pub fn resolve_policy(conn: &Connection, workspace_id: &str, agent_id: Option<&str>) -> AppResult<Option<AiAgentPolicy>> {
    if let Some(agent_id) = agent_id {
        if let Some(policy) = ai_agent_policy_repo::get(conn, workspace_id, Some(agent_id))? {
            return Ok(Some(policy));
        }
    }
    Ok(ai_agent_policy_repo::get(conn, workspace_id, None)?)
}

/// The core Tool-Call Firewall check. `agent_id` is `None` for the fixed
/// "records"/"admin" chat assistants (only the workspace default policy
/// can govern those - they have no agent identity of their own to carry
/// a policy). `source` is `chat_service::tool_source`'s own
/// classification, passed straight through to
/// `tool_registry_service::effective_risk`.
pub fn evaluate(conn: &Connection, workspace_id: &str, agent_id: Option<&str>, tool_name: &str, source: Option<&str>) -> AppResult<PolicyDecision> {
    let policy = resolve_policy(conn, workspace_id, agent_id)?;
    let Some(policy) = policy else {
        return Ok(PolicyDecision::Allow);
    };
    if policy.blocked_tool_names.iter().any(|n| n == tool_name) {
        let risk_level = super::tool_registry_service::effective_risk(conn, workspace_id, tool_name, source)?;
        return Ok(PolicyDecision::Deny { risk_level });
    }
    let Some(threshold) = policy.require_approval_at_or_above else {
        return Ok(PolicyDecision::Allow);
    };
    let risk_level = super::tool_registry_service::effective_risk(conn, workspace_id, tool_name, source)?;
    if risk_level >= threshold {
        Ok(PolicyDecision::RequireApproval { risk_level })
    } else {
        Ok(PolicyDecision::Allow)
    }
}

/// AI Agent Platform v2, Phase 4: the memory-write sibling of `evaluate`
/// above - "may this content be persisted to memory" isn't a tool call
/// (there's no `tool_name`/risk-tier concept for a piece of text), so it
/// doesn't fit through `evaluate`'s `tool_registry_service::effective_risk`
/// path or `PolicyDecision`'s `RiskLevel`-carrying shape unmodified. Reuses
/// the same resolved `AiAgentPolicy` row; returns a plain `bool` rather
/// than `PolicyDecision` since memory has only two outcomes, never a
/// `RequireApproval` - there's no pending-approval concept meaningful for a
/// background memory capture the way there is for an explicit tool call.
/// Called by `ai_memory_service::remember` before a write, in front of
/// `ai_memory_repo::create`, never in place of it - the same "governance
/// layer, not a second enforcement path" principle this module's own top
/// doc comment states.
pub fn evaluate_memory_write(conn: &Connection, workspace_id: &str, agent_id: Option<&str>, classification: &str) -> AppResult<bool> {
    if classification != "restricted" {
        return Ok(true);
    }
    let policy = resolve_policy(conn, workspace_id, agent_id)?;
    let excludes_restricted = policy.map(|p| p.exclude_restricted_memory).unwrap_or(true);
    Ok(!excludes_restricted)
}

/// Records the durable, auditable paper trail for a `RequireApproval`
/// decision - see this module's own doc comment for why this doesn't
/// (yet) pause and resume the calling run.
pub fn record_pending_tool_call(
    conn: &Connection,
    workspace_id: &str,
    agent_id: Option<&str>,
    tool_name: &str,
    arguments: &Value,
    risk_level: RiskLevel,
    requested_by: Option<&str>,
) -> AppResult<()> {
    let input = AiApprovalInput {
        subject_type: "tool_call".to_string(),
        subject_id: tool_name.to_string(),
        proposal: json!({"agent_id": agent_id, "tool_name": tool_name, "arguments": arguments, "risk_level": risk_level.as_str()}),
    };
    super::approval_service::create(conn, workspace_id, &input, requested_by)?;
    Ok(())
}

pub fn get_policy(conn: &Connection, workspace_id: &str, agent_id: Option<&str>, actor_user_id: Option<&str>) -> AppResult<Option<AiAgentPolicy>> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_agent_policy_repo::get(conn, workspace_id, agent_id)?)
}

pub fn list_policies(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<AiAgentPolicy>> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_agent_policy_repo::list(conn, workspace_id)?)
}

pub fn upsert_policy(conn: &Connection, workspace_id: &str, agent_id: Option<&str>, input: &AiAgentPolicyInput, actor_user_id: Option<&str>) -> AppResult<AiAgentPolicy> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_agent_policy_repo::upsert(conn, workspace_id, agent_id, input, actor_user_id)?)
}
