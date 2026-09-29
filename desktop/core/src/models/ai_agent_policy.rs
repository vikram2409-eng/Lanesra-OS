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
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiAgentPolicyInput {
    pub require_approval_at_or_above: Option<RiskLevel>,
    pub blocked_tool_names: Vec<String>,
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
