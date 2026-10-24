//! WP-0140: caps on an agent's tool bindings and tool rate limits
//! (migration 0086). See `services::tool_limits_service`.

use serde::{Deserialize, Serialize};

/// The smallest result cap: below this a reference would be longer than
/// the result it stands for.
pub const MIN_RESULT_CHARS: i64 = 100;

/// Rate-limit scopes: one tool by name, one connector (any of its
/// actions) by id, or every tool call in the workspace (`scope_key` `*`).
pub const RATE_LIMIT_SCOPES: &[&str] = &["tool", "connector", "workspace"];

/// One agent's binding to one tool, with its caps and live state.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AgentToolCap {
    pub agent_id: String,
    pub tool_name: String,
    /// Calls one step (one agent run) may make; `None` is no cap.
    pub max_calls_per_step: Option<i64>,
    /// Failures in a row after which the binding is disabled.
    pub failures_before_disable: Option<i64>,
    /// Characters of a result kept in the model's context; a longer result
    /// is stored and referenced.
    pub max_result_chars: Option<i64>,
    pub consecutive_failures: i64,
    /// Set when the binding was disabled; an Administrator re-enables it.
    pub disabled_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentToolCapInput {
    pub tool_name: String,
    pub max_calls_per_step: Option<i64>,
    pub failures_before_disable: Option<i64>,
    pub max_result_chars: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ToolRateLimit {
    pub id: String,
    pub workspace_id: String,
    pub scope: String,
    pub scope_key: String,
    pub max_calls: i64,
    pub per_seconds: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolRateLimitInput {
    pub scope: String,
    /// The tool name, the connector id, or `*` for the workspace.
    pub scope_key: String,
    pub max_calls: i64,
    pub per_seconds: i64,
}

/// A result too large for the model's context, kept whole.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StoredToolResult {
    pub id: String,
    pub agent_id: Option<String>,
    pub tool_name: String,
    pub body: String,
    pub created_at: String,
}
