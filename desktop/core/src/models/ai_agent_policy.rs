//! AI Agent Platform v2, Phase 2: a unified Policy Engine row - one per
//! (workspace, agent), or one workspace-wide default when `agent_id` is
//! `None`. See migration `0061_agent_policy_engine.sql`'s own doc comment
//! for the no-row-means-Allow-everything default and
//! `services::policy_engine_service` for how a policy is resolved and
//! evaluated against a specific tool call.

use serde::{Deserialize, Serialize};

use super::ai_tool_registry::RiskLevel;

#[derive(Debug, Clone, Serialize)]
pub struct AiAgentPolicy {
    pub id: String,
    pub workspace_id: String,
    /// `None` is the workspace-wide default policy.
    pub agent_id: Option<String>,
    /// A tool whose effective risk is at or above this level is queued
    /// for approval instead of dispatched immediately. `None` means
    /// "never require approval by risk level" under this policy.
    pub require_approval_at_or_above: Option<RiskLevel>,
    /// Tool names always denied outright under this policy, regardless
    /// of risk level.
    pub blocked_tool_names: Vec<String>,
    /// AI Agent Platform v2, Phase 4 (migration 0063): whether a
    /// `'restricted'`-classified `ai_memory_service::remember` write under
    /// this policy's scope is excluded from durable persistence rather
    /// than stored. Defaults `true` (excluded) - see
    /// `services::policy_engine_service::evaluate_memory_write`.
    pub exclude_restricted_memory: bool,
    /// Agent Access Governance (issue #245): whether a record write this
    /// policy's scope makes (`create_record`/`update_record`/
    /// `archive_record`) is checked for real against Access Control v1,
    /// instead of the `actor_user_id: None` "unattributed/system"
    /// convention every AI-driven write has always used. Defaults
    /// `false` - zero behavior change for a workspace that never opens
    /// this screen. See `chat_service::effective_write_actor`.
    pub enforce_record_access: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiAgentPolicyInput {
    pub require_approval_at_or_above: Option<RiskLevel>,
    pub blocked_tool_names: Vec<String>,
    #[serde(default = "default_exclude_restricted_memory")]
    pub exclude_restricted_memory: bool,
    #[serde(default)]
    pub enforce_record_access: bool,
}

fn default_exclude_restricted_memory() -> bool {
    true
}

/// What `policy_engine_service::evaluate` decided for one tool call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum PolicyDecision {
    Allow,
    /// Carries the effective risk level that triggered this, for the
    /// caller's own error/result message.
    RequireApproval { risk_level: RiskLevel },
    Deny { risk_level: RiskLevel },
}
