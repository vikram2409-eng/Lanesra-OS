//! AI Agent Platform v2, Phase 4 (GitHub issue #169): the three itemized
//! memory types on top of migration `0063_memory_architecture.sql`'s
//! `ai_memory_items` table. Agent Memory (today's `memory_md`) is
//! unaffected - see the migration's own doc comment.
//!
//! What each type is scoped by, and how long it lives:
//! - **Session Memory**: this agent's ongoing relationship with one user -
//!   `session_key = "{agent_id}:{user_id}"` (see `chat_service::
//!   send_agent_message`'s own construction of it). This codebase has no
//!   first-class conversation/session-id primitive yet (chat history is
//!   just an unbounded, persisted log per agent+user), so that pair is the
//!   honest session boundary available today - named here, not silently
//!   assumed. Written with a short default TTL (`SESSION_TTL_SECONDS`)
//!   unless the caller supplies its own.
//! - **Working Memory**: one real run - `run_id` is a genuine `ai_runs.id`
//!   (Execution Graph) or `ai_agent_runs.id` (Pipeline), threaded in via
//!   `models::ai_memory::AgentMemoryContext`. A `remember`/`get_memory`
//!   call for `memory_type: "working"` outside an actual run context (an
//!   interactive chat message, which has no durable run row) errors
//!   clearly rather than silently no-oping or inventing a key.
//! - **Entity Memory**: a durable fact tied to one record
//!   (`entity_type`/`entity_id`, the same shape `entity_registry::resolve`
//!   already uses) - normally never expires; an admin/user explicitly
//!   `forget`s it instead.
//!
//! **Policy gate**: every `remember` call for `classification: "restricted"`
//! content is checked against `policy_engine_service::evaluate_memory_write`
//! before it's persisted - a denied write returns a clear error, the same
//! "governance layer in front of the real write, never a second enforcement
//! path" principle Phase 2's Tool-Call Firewall established.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::ai_memory::{MemoryItem, MemoryItemInput, CLASSIFICATIONS, MEMORY_TYPES};
use crate::repositories::{ai_agent_repo, ai_memory_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// Session Memory's own default TTL when a caller doesn't supply one -
/// long enough to span a real back-and-forth, short enough that a stale
/// session doesn't linger indefinitely.
const SESSION_TTL_SECONDS: i64 = 6 * 3600;
/// Working Memory's own default TTL - a run's own lifetime is normally
/// much shorter than a session's.
const WORKING_TTL_SECONDS: i64 = 3600;

fn expires_at_from(ttl_seconds: Option<i64>, default_ttl: i64) -> Option<String> {
    let seconds = ttl_seconds.unwrap_or(default_ttl);
    if seconds <= 0 {
        return None;
    }
    Some((chrono::Utc::now() + chrono::Duration::seconds(seconds)).to_rfc3339())
}

/// The write path every `remember` call (an agent's own tool call, or a
/// future non-agent writer) goes through. `agent_id` is required - unlike
/// `policy_engine_service::evaluate`'s tool-call firewall, memory has no
/// fixed "records"/"admin" assistant caller, only named Agents.
pub fn remember(conn: &Connection, workspace_id: &str, agent_id: &str, input: &MemoryItemInput, context: &crate::models::ai_memory::AgentMemoryContext, created_by: &str) -> AppResult<MemoryItem> {
    if !MEMORY_TYPES.contains(&input.memory_type.as_str()) {
        return Err(AppError::Validation(format!("'{}' is not a valid memory_type", input.memory_type)));
    }
    if !CLASSIFICATIONS.contains(&input.classification.as_str()) {
        return Err(AppError::Validation(format!("'{}' is not a valid classification", input.classification)));
    }
    if input.content.trim().is_empty() {
        return Err(AppError::Validation("content is required".into()));
    }

    if !super::policy_engine_service::evaluate_memory_write(conn, workspace_id, Some(agent_id), &input.classification)? {
        return Err(AppError::Validation(format!(
            "This workspace's policy excludes '{}'-classified content from durable memory",
            input.classification
        )));
    }

    let (session_key, run_id, expires_at) = match input.memory_type.as_str() {
        "session" => {
            let key = context.session_key.as_deref().ok_or_else(|| AppError::Validation("Session Memory isn't available outside an interactive chat with a named agent".into()))?;
            (Some(key), None, expires_at_from(input.ttl_seconds, SESSION_TTL_SECONDS))
        }
        "working" => {
            let run_id = context.run_id.as_deref().ok_or_else(|| AppError::Validation("Working Memory isn't available outside a Pipeline or Execution Graph run".into()))?;
            (None, Some(run_id), expires_at_from(input.ttl_seconds, WORKING_TTL_SECONDS))
        }
        "entity" => {
            if input.entity_type.as_deref().unwrap_or_default().is_empty() || input.entity_id.as_deref().unwrap_or_default().is_empty() {
                return Err(AppError::Validation("Entity Memory requires both entity_type and entity_id".into()));
            }
            (None, None, expires_at_from(input.ttl_seconds, 0))
        }
        other => return Err(AppError::Validation(format!("'{other}' is not a valid memory_type"))),
    };

    let item = ai_memory_repo::create(
        conn,
        workspace_id,
        &input.memory_type,
        Some(agent_id),
        session_key,
        run_id,
        input.entity_type.as_deref(),
        input.entity_id.as_deref(),
        &input.content,
        &input.source,
        input.confidence,
        &input.classification,
        Some(created_by),
        expires_at.as_deref(),
    )?;
    Ok(item)
}

/// What a `get_memory` tool call (or `chat_service::agent_system_prompt`'s
/// own ambient Session Memory injection) reads back, scoped exactly the
/// same way `remember` writes - explicit `entity_type`/`entity_id` for
/// Entity Memory, `context`'s own session/run key for the other two.
pub fn list_context(conn: &Connection, agent_id: &str, memory_type: &str, entity_type: Option<&str>, entity_id: Option<&str>, context: &crate::models::ai_memory::AgentMemoryContext) -> AppResult<Vec<MemoryItem>> {
    match memory_type {
        "entity" => {
            let (Some(et), Some(eid)) = (entity_type, entity_id) else {
                return Err(AppError::Validation("Entity Memory requires both entity_type and entity_id".into()));
            };
            let workspace_id = agent_workspace(conn, agent_id)?;
            Ok(ai_memory_repo::list_entity(conn, &workspace_id, et, eid)?)
        }
        "session" => {
            let Some(key) = &context.session_key else {
                return Err(AppError::Validation("Session Memory isn't available outside an interactive chat with a named agent".into()));
            };
            Ok(ai_memory_repo::list_session(conn, agent_id, key, 25)?)
        }
        "working" => {
            let Some(run_id) = &context.run_id else {
                return Err(AppError::Validation("Working Memory isn't available outside a Pipeline or Execution Graph run".into()));
            };
            Ok(ai_memory_repo::list_working(conn, agent_id, run_id)?)
        }
        other => Err(AppError::Validation(format!("'{other}' is not a valid memory_type"))),
    }
}

fn agent_workspace(conn: &Connection, agent_id: &str) -> AppResult<String> {
    Ok(ai_agent_repo::get(conn, agent_id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?.workspace_id)
}

/// The admin Memory Inspector's own listing - every item in the workspace,
/// optionally narrowed by type and/or entity, unfiltered by expiry so an
/// admin can see (and clean up) an already-expired item too.
pub fn list_all(conn: &Connection, workspace_id: &str, memory_type: Option<&str>, entity_type: Option<&str>, entity_id: Option<&str>, actor_user_id: Option<&str>) -> AppResult<Vec<MemoryItem>> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_memory_repo::list_all(conn, workspace_id, memory_type, entity_type, entity_id)?)
}

/// The explicit "an admin/user can inspect and delete retained memory"
/// action this issue's own scope text asks for.
pub fn forget(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let deleted = ai_memory_repo::delete(conn, workspace_id, id)?;
    if deleted == 0 {
        return Err(AppError::NotFound("Memory item".into()));
    }
    Ok(())
}

/// TTL reclaim for Session/Working items - the same sweep shape
/// `workflow_service::run_scheduled` already established for the Workflow
/// engine's own scheduled trigger, called from the same job-scheduler tick.
pub fn sweep_expired(conn: &Connection, workspace_id: &str) -> AppResult<usize> {
    Ok(ai_memory_repo::sweep_expired(conn, workspace_id)?)
}
