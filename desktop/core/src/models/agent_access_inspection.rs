//! Agent Access Governance (issue #245): the Agent Access Inspector - a
//! real, traceable answer to "what can this agent actually touch", the
//! agent-shaped sibling of Access Control v1's own `AccessInspectorResult`
//! (`access_role.rs`). Reuses the exact enforcement evaluators
//! (`policy_engine_service::evaluate`, `access_service::explain_access`)
//! this call would run through for real - never a second evaluator.

use serde::Serialize;

use super::access_role::AccessInspectorResult;
use super::ai_agent_policy::PolicyDecision;

#[derive(Debug, Clone, Serialize)]
pub struct AgentAccessInspection {
    pub agent_id: String,
    pub agent_name: String,
    pub tool_name: String,
    /// Whether `tool_name` is actually in this agent's own
    /// `action_names` - a tool outside this list never even reaches
    /// `policy_engine_service::evaluate` for a real call (see
    /// `chat_service::execute_agent_tool`'s `other => Err(...)` arm), so
    /// this is checked and reported even though it doesn't gate the rest
    /// of this trace.
    pub tool_in_action_list: bool,
    pub policy_decision: PolicyDecision,
    /// Where the decision above actually came from - `"agent-specific
    /// policy"`, `"workspace default policy"`, or `"no policy configured
    /// (defaults to Allow)"`.
    pub policy_source: String,
    /// Whether this workspace has opted this agent's record writes into
    /// real Access Control v1 enforcement - see `chat_service::
    /// effective_write_actor`'s own doc comment. `false` means every
    /// `record_access` field below is `None`, not because nothing was
    /// checked but because nothing is enforced yet.
    pub record_access_enforced: bool,
    /// Who `record_access` was traced for - this agent's own
    /// `acts_as_user_id` if it has one, else the simulated/current user
    /// the caller supplied. `None` when `record_access_enforced` is
    /// `false`.
    pub acting_as_user_id: Option<String>,
    /// Only populated when `tool_name` is `create_record`/`update_record`/
    /// `archive_record`, an object/record was supplied, and
    /// `record_access_enforced` is `true` - the exact same
    /// `AccessInspectorResult` the real user-facing Access Inspector
    /// renders, traced for `acting_as_user_id` instead.
    pub record_access: Option<AccessInspectorResult>,
}
