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

use crate::domain::{AppError, AppResult};
use crate::models::ai_agent_policy::{AiAgentPolicy, AiAgentPolicyInput, PolicyDecision};
use crate::models::ai_approval::AiApprovalInput;
use crate::models::ai_tool_registry::RiskLevel;
use crate::repositories::{ai_agent_policy_repo, ai_approval_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// The policy that actually governs a call for this `agent_id` - the
/// agent-specific policy if one exists, else the workspace-wide default,
/// else `None` (no policy configured anywhere in this workspace - every
/// call Allowed, today's exact behavior, unchanged).
pub fn resolve_policy(
    conn: &Connection,
    workspace_id: &str,
    agent_id: Option<&str>,
) -> AppResult<Option<AiAgentPolicy>> {
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
pub fn evaluate(
    conn: &Connection,
    workspace_id: &str,
    agent_id: Option<&str>,
    tool_name: &str,
    source: Option<&str>,
) -> AppResult<PolicyDecision> {
    let policy = resolve_policy(conn, workspace_id, agent_id)?;
    let Some(policy) = policy else {
        return Ok(PolicyDecision::Allow);
    };
    if policy.blocked_tool_names.iter().any(|n| n == tool_name) {
        let risk_level =
            super::tool_registry_service::effective_risk(conn, workspace_id, tool_name, source)?;
        return Ok(PolicyDecision::Deny { risk_level });
    }
    let Some(threshold) = policy.require_approval_at_or_above else {
        return Ok(PolicyDecision::Allow);
    };
    let risk_level =
        super::tool_registry_service::effective_risk(conn, workspace_id, tool_name, source)?;
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
pub fn evaluate_memory_write(
    conn: &Connection,
    workspace_id: &str,
    agent_id: Option<&str>,
    classification: &str,
) -> AppResult<bool> {
    if classification != "restricted" {
        return Ok(true);
    }
    let policy = resolve_policy(conn, workspace_id, agent_id)?;
    let excludes_restricted = policy.map(|p| p.exclude_restricted_memory).unwrap_or(true);
    Ok(!excludes_restricted)
}

/// Records the durable, auditable paper trail for a `RequireApproval`
/// decision - see this module's own doc comment for why this doesn't
/// (yet) pause and resume the calling run. `tainted` (WP-0145) records on
/// the approval that the call was escalated because its step had already
/// read untrusted content, so a reviewer sees why it waits.
pub fn record_pending_tool_call(
    conn: &Connection,
    workspace_id: &str,
    agent_id: Option<&str>,
    tool_name: &str,
    arguments: &Value,
    risk_level: RiskLevel,
    requested_by: Option<&str>,
    tainted: bool,
) -> AppResult<String> {
    let input = AiApprovalInput {
        subject_type: "tool_call".to_string(),
        subject_id: tool_name.to_string(),
        proposal: json!({"agent_id": agent_id, "tool_name": tool_name, "arguments": arguments, "risk_level": risk_level.as_str(), "tainted": tainted, "reason": if tainted { "untrusted_content" } else { "risk_threshold" }}),
    };
    Ok(super::approval_service::create(conn, workspace_id, &input, requested_by)?.id)
}

/// WP-0145: whether a tool call made in a step that has already read
/// untrusted content must be escalated to approval. True when the call is a
/// write-tier or external-action tool (effective risk above `Read`) and the
/// governing policy does not list it as taint-exempt. A pure decision over
/// the resolved policy and the effective risk the firewall supplies.
pub fn taint_requires_approval(
    conn: &Connection,
    workspace_id: &str,
    agent_id: Option<&str>,
    tool_name: &str,
    risk_level: RiskLevel,
) -> AppResult<bool> {
    if risk_level <= RiskLevel::Read {
        return Ok(false);
    }
    let exempt = resolve_policy(conn, workspace_id, agent_id)?
        .map(|p| p.taint_exempt_tool_names.iter().any(|n| n == tool_name))
        .unwrap_or(false);
    Ok(!exempt)
}

/// `value` with every object's keys sorted, so two equal argument sets
/// compare equal however their keys were ordered.
fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for k in keys {
                out.insert(k.clone(), canonical(&map[k]));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonical).collect()),
        other => other.clone(),
    }
}

/// WP-0150: an approved, unused grant for exactly this call - the same
/// agent, tool and arguments - is used up and returned; the call may then
/// run once. Anything else (other arguments, another agent, a used grant)
/// leaves the call held.
pub fn take_tool_grant(
    conn: &Connection,
    workspace_id: &str,
    agent_id: Option<&str>,
    tool_name: &str,
    arguments: &Value,
) -> AppResult<Option<String>> {
    let wanted = canonical(arguments);
    for grant in ai_approval_repo::unused_tool_call_grants(conn, workspace_id)? {
        let p = &grant.proposal;
        let same = p.get("tool_name").and_then(Value::as_str) == Some(tool_name)
            && p.get("agent_id").and_then(Value::as_str) == agent_id
            && p.get("arguments").map(canonical).as_ref() == Some(&wanted);
        if same && ai_approval_repo::consume(conn, &grant.id)? {
            return Ok(Some(grant.id));
        }
    }
    Ok(None)
}

// ---- WP-0128: loop hook points -------------------------------------------
//
// The Tool-Call Firewall above (`evaluate`) governs a tool call. WP-0128
// generalises that into three hook points around the agent loop, each
// configured as a Policy object (a guardrail attached at that point, no
// code), evaluated in a fixed order with short-circuit:
//
//   1. a step boundary (before a step runs, against the step input),
//   2. a model call (against the model's output text),
//   3. a tool call (against the tool's arguments).
//
// At each point every enabled guardrail attached there (WP-0133) runs; the
// first that fails blocks execution. Because the loop reaches the points in
// order, the first block short-circuits the rest (AC-1). Every verdict -
// allow as well as block - is recorded on a `policy` trace span carrying a
// `lanesra.policy.hook` attribute and emitted as a `policy.decided` run
// event (AC-2). A point with no guardrail attached is a no-op, so a
// workspace that configures none sees no change.

use crate::models::ai_guardrail::Guardrail;
use crate::services::trace_service::{self, SpanStatus};

/// One of the three points in the agent loop a policy can govern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookPoint {
    /// Before a step runs, against the step input.
    Step,
    /// A model call, against the model's output text.
    Model,
    /// A tool call, against the tool's arguments.
    Tool,
}

impl HookPoint {
    pub fn as_str(self) -> &'static str {
        match self {
            HookPoint::Step => "step",
            HookPoint::Model => "model",
            HookPoint::Tool => "tool",
        }
    }

    /// The three points in the order the loop evaluates them; a block at an
    /// earlier point short-circuits the later ones.
    pub const ORDER: [HookPoint; 3] = [HookPoint::Step, HookPoint::Model, HookPoint::Tool];
}

/// What a loop hook decided. `Block` names the guardrail that blocked and
/// why, for the caller's error message and the audit trail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookVerdict {
    Allow,
    Block {
        hook: HookPoint,
        guardrail_id: String,
        guardrail_name: String,
        reason: String,
    },
}

impl HookVerdict {
    pub fn is_block(&self) -> bool {
        matches!(self, HookVerdict::Block { .. })
    }

    /// The `AppError::PolicyBlocked` a blocked hook turns into for the run
    /// loop; `Allow` has none.
    pub fn into_error(self) -> Option<AppError> {
        match self {
            HookVerdict::Allow => None,
            HookVerdict::Block {
                hook,
                guardrail_name,
                reason,
                ..
            } => Some(AppError::PolicyBlocked(format!(
                "Blocked at the {} hook by guardrail '{}': {}",
                hook.as_str(),
                guardrail_name,
                reason
            ))),
        }
    }
}

/// A guardrail of a kind the loop can evaluate without a live model - every
/// kind but `judge`, which needs a separate model verdict the loop does not
/// have at a hook point yet (named follow-up). A `judge` guardrail attached
/// at a hook point is skipped, not an error.
fn runnable_at_hook(g: &Guardrail) -> bool {
    g.checker_type != "judge"
}

/// Runs the guardrails attached at `point` against `subject`, in their
/// stable order. Returns `Block` at the first that fails, else `Allow`.
/// Records a `policy` hook span and a `policy.decided` event for the
/// verdict (AC-2); a point with no runnable guardrail attached is a no-op
/// (no span, no event, `Allow`).
pub fn evaluate_loop_hook(
    conn: &Connection,
    workspace_id: &str,
    point: HookPoint,
    subject: &str,
) -> AppResult<HookVerdict> {
    let guardrails: Vec<Guardrail> =
        crate::repositories::ai_guardrail_repo::list_for_hook(conn, workspace_id, point.as_str())?
            .into_iter()
            .filter(runnable_at_hook)
            .collect();
    if guardrails.is_empty() {
        return Ok(HookVerdict::Allow);
    }
    for g in &guardrails {
        let config: Value = serde_json::from_str(&g.config_json).unwrap_or(Value::Null);
        let outcome = super::guardrail_service::evaluate(&g.checker_type, &config, subject, None)?;
        if !outcome.passed {
            record_hook_span(
                conn,
                point,
                SpanStatus::Error,
                "block",
                Some(g),
                Some(&outcome.detail),
            );
            return Ok(HookVerdict::Block {
                hook: point,
                guardrail_id: g.id.clone(),
                guardrail_name: g.name.clone(),
                reason: outcome.detail,
            });
        }
    }
    record_hook_span(conn, point, SpanStatus::Ok, "allow", None, None);
    Ok(HookVerdict::Allow)
}

fn record_hook_span(
    conn: &Connection,
    point: HookPoint,
    status: SpanStatus,
    verdict: &str,
    guardrail: Option<&Guardrail>,
    reason: Option<&str>,
) {
    // Shares the `policy` span kind with the firewall; the
    // `lanesra.policy.hook` attribute marks it a loop hook (and drives the
    // `policy.decided` event in `trace_service`).
    trace_service::start(
        conn,
        "policy",
        point.as_str(),
        json!({
            "lanesra.policy.hook": point.as_str(),
            "lanesra.policy.verdict": verdict,
            "lanesra.policy.guardrail": guardrail.map(|g| g.name.as_str()),
            "lanesra.policy.guardrail_id": guardrail.map(|g| g.id.as_str()),
            "lanesra.policy.reason": reason,
        }),
    )
    .end(conn, status, json!({}));
}

/// The subjects available at one loop round, one per hook point. A `None`
/// subject skips that point (nothing to check there yet).
#[derive(Debug, Default, Clone)]
pub struct LoopSubjects<'a> {
    pub step: Option<&'a str>,
    pub model: Option<&'a str>,
    pub tool: Option<&'a str>,
}

impl<'a> LoopSubjects<'a> {
    fn subject(&self, point: HookPoint) -> Option<&'a str> {
        match point {
            HookPoint::Step => self.step,
            HookPoint::Model => self.model,
            HookPoint::Tool => self.tool,
        }
    }
}

/// Evaluates the loop hooks in order (step, then model, then tool),
/// returning the first `Block` and leaving the remaining points unevaluated
/// (AC-1). A point with no subject is skipped.
pub fn evaluate_loop_round(
    conn: &Connection,
    workspace_id: &str,
    subjects: &LoopSubjects,
) -> AppResult<HookVerdict> {
    for point in HookPoint::ORDER {
        let Some(subject) = subjects.subject(point) else {
            continue;
        };
        let verdict = evaluate_loop_hook(conn, workspace_id, point, subject)?;
        if verdict.is_block() {
            return Ok(verdict);
        }
    }
    Ok(HookVerdict::Allow)
}

pub fn get_policy(
    conn: &Connection,
    workspace_id: &str,
    agent_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<Option<AiAgentPolicy>> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_agent_policy_repo::get(conn, workspace_id, agent_id)?)
}

pub fn list_policies(
    conn: &Connection,
    workspace_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<AiAgentPolicy>> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_agent_policy_repo::list(conn, workspace_id)?)
}

pub fn upsert_policy(
    conn: &Connection,
    workspace_id: &str,
    agent_id: Option<&str>,
    input: &AiAgentPolicyInput,
    actor_user_id: Option<&str>,
) -> AppResult<AiAgentPolicy> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_agent_policy_repo::upsert(
        conn,
        workspace_id,
        agent_id,
        input,
        actor_user_id,
    )?)
}
