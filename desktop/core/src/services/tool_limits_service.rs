//! WP-0140: per-tool caps and rate limits (REQ-LOOP-014, REQ-TOOL-008).
//!
//! **Caps on an agent's tool binding** (`ai_agent_tool_caps`), enforced by
//! the agent loop (`chat_service::run_agent_once`) for each tool it calls:
//! - calls per step - one agent run may call the tool at most this many
//!   times; a further call is refused with `policy.tool_cap` and not run;
//! - failures before disable - after this many failures in a row the
//!   binding is disabled, and calls are refused with `policy.tool_disabled`
//!   until an Administrator re-enables it;
//! - result size - a result longer than this is stored whole in
//!   `ai_tool_results`, and the model gets its start plus a reference.
//!
//! **Rate limits** (`ai_tool_rate_limits`) for one tool, one connector or
//! the whole workspace, checked by the Tool-Call Firewall before any call,
//! from the Assistant chat as well as agents. A call over a limit is
//! refused with `policy.rate_limited` and a retry time, and never reaches
//! the tool. The window counts calls the firewall let through.
//!
//! With no rows, nothing is capped or limited.

use rusqlite::Connection;
use serde_json::json;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::tool_limits::{
    AgentToolCap, AgentToolCapInput, StoredToolResult, ToolRateLimit, ToolRateLimitInput,
    MIN_RESULT_CHARS, RATE_LIMIT_SCOPES,
};
use crate::repositories::{ai_agent_repo, audit_repo, tool_limits_repo};

/// The longest rate-limit window, which is also how long calls are kept.
pub const MAX_WINDOW_SECONDS: i64 = 86_400;

fn times(n: i64) -> String {
    if n == 1 {
        "once".to_string()
    } else {
        format!("{n} times")
    }
}

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn policy_error(code: &'static str, message: String, retry_after_ms: Option<u64>) -> AppError {
    AppError::Classified {
        class: ErrorClass::Policy,
        code,
        message,
        retry_after_ms,
        http_status: None,
    }
}

fn check_positive(name: &str, value: Option<i64>, min: i64) -> AppResult<()> {
    match value {
        Some(v) if v < min => Err(AppError::Validation(format!(
            "{name} must be at least {min}"
        ))),
        _ => Ok(()),
    }
}

// --- Caps on an agent's tool bindings ------------------------------------

pub fn list_agent_tool_caps(
    conn: &Connection,
    workspace_id: &str,
    agent_id: &str,
) -> AppResult<Vec<AgentToolCap>> {
    let agent =
        ai_agent_repo::get(conn, agent_id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    if agent.workspace_id != workspace_id {
        return Err(AppError::NotFound("Agent".into()));
    }
    Ok(tool_limits_repo::list_caps(conn, agent_id)?)
}

/// Replaces an agent's tool caps. A binding that stays keeps its failure
/// state; a tool left out loses its caps. Administrator only, audited.
pub fn set_agent_tool_caps(
    conn: &Connection,
    workspace_id: &str,
    agent_id: &str,
    caps: &[AgentToolCapInput],
    actor_user_id: Option<&str>,
) -> AppResult<Vec<AgentToolCap>> {
    require_admin(conn, actor_user_id)?;
    let agent =
        ai_agent_repo::get(conn, agent_id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    if agent.workspace_id != workspace_id {
        return Err(AppError::NotFound("Agent".into()));
    }
    let tools: Vec<String> = super::chat_service::agent_tool_names(conn, &agent)?;
    let mut keep = Vec::new();
    for cap in caps {
        let name = cap.tool_name.trim();
        if !tools.iter().any(|t| t == name) {
            return Err(AppError::Validation(format!(
                "'{name}' isn't one of this agent's tools"
            )));
        }
        check_positive("Calls per step", cap.max_calls_per_step, 1)?;
        check_positive("Failures before disable", cap.failures_before_disable, 1)?;
        check_positive("Result size", cap.max_result_chars, MIN_RESULT_CHARS)?;
        tool_limits_repo::upsert_cap(
            conn,
            agent_id,
            name,
            cap.max_calls_per_step,
            cap.failures_before_disable,
            cap.max_result_chars,
            actor_user_id,
        )?;
        keep.push(name.to_string());
    }
    tool_limits_repo::delete_caps_except(conn, agent_id, &keep)?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "ai_agent_tool_caps_changed",
        Some("ai_agent"),
        Some(agent_id),
        &format!(
            "Tool caps for '{}' set on {} tool(s)",
            agent.name,
            keep.len()
        ),
        Some(&json!({"tools": keep}).to_string()),
    )?;
    Ok(tool_limits_repo::list_caps(conn, agent_id)?)
}

/// Re-enables a binding the failure cap disabled. Administrator only,
/// audited.
pub fn reenable_agent_tool(
    conn: &Connection,
    workspace_id: &str,
    agent_id: &str,
    tool_name: &str,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<AgentToolCap>> {
    require_admin(conn, actor_user_id)?;
    let caps = list_agent_tool_caps(conn, workspace_id, agent_id)?;
    if !caps.iter().any(|c| c.tool_name == tool_name) {
        return Err(AppError::NotFound("Tool cap".into()));
    }
    tool_limits_repo::set_disabled(conn, agent_id, tool_name, false)?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "ai_agent_tool_reenabled",
        Some("ai_agent"),
        Some(agent_id),
        &format!("Tool '{tool_name}' re-enabled for this agent"),
        Some(&json!({"tool": tool_name}).to_string()),
    )?;
    Ok(tool_limits_repo::list_caps(conn, agent_id)?)
}

/// What the agent loop decided before calling a capped tool.
pub enum CapCheck {
    /// Run it; `cap` is the binding's caps, if it has any.
    Run(Option<AgentToolCap>),
    /// Do not run it; the error is what the model is told.
    Refuse { kind: &'static str, error: AppError },
}

/// Checks a binding before a call. `calls_this_step` counts this tool's
/// calls so far in the current step.
pub fn check_binding(
    conn: &Connection,
    agent_id: &str,
    tool_name: &str,
    calls_this_step: i64,
) -> AppResult<CapCheck> {
    let Some(cap) = tool_limits_repo::get_cap(conn, agent_id, tool_name)? else {
        return Ok(CapCheck::Run(None));
    };
    if cap.disabled_at.is_some() {
        return Ok(CapCheck::Refuse {
            kind: "disabled",
            error: policy_error(
                "policy.tool_disabled",
                format!(
                    "'{tool_name}' is disabled for this agent after {} failures in a row; an Administrator can re-enable it",
                    cap.consecutive_failures
                ),
                None,
            ),
        });
    }
    if let Some(max) = cap.max_calls_per_step {
        if calls_this_step >= max {
            return Ok(CapCheck::Refuse {
                kind: "calls_per_step",
                error: policy_error(
                    "policy.tool_cap",
                    format!(
                        "'{tool_name}' may be called at most {} in one step",
                        times(max)
                    ),
                    None,
                ),
            });
        }
    }
    Ok(CapCheck::Run(Some(cap)))
}

/// Records a capped binding's outcome. Returns `true` when this failure
/// disabled the binding.
pub fn record_binding_outcome(conn: &Connection, cap: &AgentToolCap, ok: bool) -> AppResult<bool> {
    let failures = tool_limits_repo::record_outcome(conn, &cap.agent_id, &cap.tool_name, ok)?;
    if let Some(limit) = cap.failures_before_disable {
        if !ok && failures >= limit {
            tool_limits_repo::set_disabled(conn, &cap.agent_id, &cap.tool_name, true)?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// A result over the binding's size cap: stores it whole and returns what
/// the model gets instead, and the stored id.
pub fn store_large_result(
    conn: &Connection,
    workspace_id: &str,
    agent_id: &str,
    tool_name: &str,
    content: &str,
    max_chars: i64,
) -> AppResult<(String, String)> {
    let id = new_uuid();
    tool_limits_repo::store_result(conn, &id, workspace_id, Some(agent_id), tool_name, content)?;
    let total = content.chars().count();
    let preview: String = content.chars().take(max_chars as usize).collect();
    let replacement = format!(
        "The result of '{tool_name}' is {total} characters, over this tool's limit of {max_chars}, so it was stored as tool_result:{id}. Its first {max_chars} characters:\n{preview}"
    );
    Ok((replacement, id))
}

/// A stored result, for an Administrator.
pub fn get_stored_result(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<StoredToolResult> {
    require_admin(conn, actor_user_id)?;
    tool_limits_repo::get_result(conn, workspace_id, id)?
        .ok_or_else(|| AppError::NotFound("Stored tool result".into()))
}

// --- Rate limits -----------------------------------------------------------

pub fn list_rate_limits(conn: &Connection, workspace_id: &str) -> AppResult<Vec<ToolRateLimit>> {
    Ok(tool_limits_repo::list_limits(conn, workspace_id)?)
}

/// Adds a rate limit, or changes the one for the same scope and key.
/// Administrator only, audited.
pub fn save_rate_limit(
    conn: &Connection,
    workspace_id: &str,
    input: &ToolRateLimitInput,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<ToolRateLimit>> {
    require_admin(conn, actor_user_id)?;
    let scope = input.scope.trim();
    if !RATE_LIMIT_SCOPES.contains(&scope) {
        return Err(AppError::Validation(
            "A rate limit applies to a tool, a connector or the workspace".into(),
        ));
    }
    let key = if scope == "workspace" {
        "*"
    } else {
        input.scope_key.trim()
    };
    if key.is_empty() {
        return Err(AppError::Validation(format!("Name the {scope} to limit")));
    }
    if input.max_calls < 1 {
        return Err(AppError::Validation("Allow at least 1 call".into()));
    }
    if !(1..=MAX_WINDOW_SECONDS).contains(&input.per_seconds) {
        return Err(AppError::Validation(format!(
            "The window must be between 1 and {MAX_WINDOW_SECONDS} seconds"
        )));
    }
    tool_limits_repo::upsert_limit(
        conn,
        &new_uuid(),
        workspace_id,
        scope,
        key,
        input.max_calls,
        input.per_seconds,
        actor_user_id,
    )?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "ai_tool_rate_limit_saved",
        Some("ai_tool_rate_limit"),
        None,
        &format!(
            "Rate limit for {scope} '{key}': {} calls per {} s",
            input.max_calls, input.per_seconds
        ),
        Some(&json!({"scope": scope, "scope_key": key, "max_calls": input.max_calls, "per_seconds": input.per_seconds}).to_string()),
    )?;
    Ok(tool_limits_repo::list_limits(conn, workspace_id)?)
}

pub fn delete_rate_limit(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<ToolRateLimit>> {
    require_admin(conn, actor_user_id)?;
    if tool_limits_repo::delete_limit(conn, workspace_id, id)? == 0 {
        return Err(AppError::NotFound("Rate limit".into()));
    }
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "ai_tool_rate_limit_deleted",
        Some("ai_tool_rate_limit"),
        Some(id),
        "Rate limit removed",
        None,
    )?;
    Ok(tool_limits_repo::list_limits(conn, workspace_id)?)
}

/// The connector a tool name belongs to (`connector_action:<id>:<key>` or
/// `connector_write_action:<id>:<key>`).
pub fn connector_of(tool_name: &str) -> Option<&str> {
    let rest = tool_name
        .strip_prefix("connector_action:")
        .or_else(|| tool_name.strip_prefix("connector_write_action:"))?;
    rest.split_once(':').map(|(id, _)| id)
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// The firewall's rate check (AC-2): refuses a call over any matching
/// limit, else counts it. Called only for a call the policy allows, just
/// before the tool would run.
pub fn check_and_count_call(
    conn: &Connection,
    workspace_id: &str,
    tool_name: &str,
) -> AppResult<()> {
    check_and_count_call_at(conn, workspace_id, tool_name, now_ms())
}

pub fn check_and_count_call_at(
    conn: &Connection,
    workspace_id: &str,
    tool_name: &str,
    at_ms: i64,
) -> AppResult<()> {
    let limits = tool_limits_repo::list_limits(conn, workspace_id)?;
    if limits.is_empty() {
        return Ok(());
    }
    let connector = connector_of(tool_name);
    for limit in &limits {
        let applies = match limit.scope.as_str() {
            "tool" => limit.scope_key == tool_name,
            "connector" => connector == Some(limit.scope_key.as_str()),
            _ => true,
        };
        if !applies {
            continue;
        }
        let window_ms = limit.per_seconds * 1000;
        let (count, oldest) = tool_limits_repo::count_calls_since(
            conn,
            workspace_id,
            &limit.scope,
            &limit.scope_key,
            at_ms - window_ms,
        )?;
        if count >= limit.max_calls {
            let retry_after_ms = oldest.map(|o| (o + window_ms - at_ms).max(0) as u64);
            let what = match limit.scope.as_str() {
                "tool" => format!("'{tool_name}'"),
                "connector" => format!("connector '{}'", limit.scope_key),
                _ => "this workspace's tools".to_string(),
            };
            return Err(policy_error(
                "policy.rate_limited",
                format!(
                    "Rate limit reached: {what} may be called {} per {} s; try again later",
                    times(limit.max_calls),
                    limit.per_seconds
                ),
                retry_after_ms,
            ));
        }
    }
    tool_limits_repo::record_call(conn, workspace_id, tool_name, connector, at_ms)?;
    tool_limits_repo::prune_calls_before(conn, workspace_id, at_ms - MAX_WINDOW_SECONDS * 1000)?;
    Ok(())
}
