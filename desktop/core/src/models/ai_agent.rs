//! AI & Agentic Layer, Phase 6: the AI Agent Foundry's own entities - a
//! named, admin-defined `AiAgentDefinition` (persona + Memory + Actions +
//! delegation) and a reusable `AiSkill` library. See
//! `services::ai_agent_service` and `services::chat_service::
//! run_agent_once`/`send_agent_message` for how these actually run.

use serde::{Deserialize, Serialize};

use crate::models::ai::AiAgentModelRouting;

#[derive(Debug, Clone, Serialize)]
pub struct AiAgentDefinition {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub system_prompt: String,
    /// A living document this agent reads every run and can revise itself
    /// via the always-available `update_memory` tool - see
    /// `chat_service`'s own doc comment. Never `null`, only ever `""`
    /// before anything's been written.
    pub memory_md: String,
    /// Individual tool names drawn from `chat_service::record_tools()`/
    /// `admin_tools()` - not a coarse scope. Whether this agent needs
    /// Administrator is computed from this set, not stored -
    /// see `ai_agent_service::agent_requires_admin`.
    pub action_names: Vec<String>,
    /// Other active agents this one may call via the `delegate_to_agent`
    /// tool - hierarchy. Populated by `ai_agent_repo::list_delegate_ids`.
    pub delegate_agent_ids: Vec<String>,
    /// Skills attached to this agent - only their name+description are
    /// injected into its system prompt; `instructions_md` is loaded only
    /// when the model calls `use_skill`.
    pub skill_ids: Vec<String>,
    /// Phase 7a: the Gateway's optional per-agent routing policy - `None`
    /// means this agent still dispatches straight through the workspace's
    /// single `ai_settings` default, exactly as every agent did before
    /// this phase. Populated by `ai_agent_repo::get_routing`, edited
    /// separately via `ai_agent_service::set_model_routing` - the same
    /// "its own admin action, not part of the main create/update form
    /// payload" shape `memory_md`/`set_memory` already established.
    pub model_routing: Option<AiAgentModelRouting>,
    pub is_active: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiAgentInput {
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub system_prompt: String,
    pub action_names: Vec<String>,
    /// Replace-all-on-update, same convention business rule
    /// conditions/actions already use - not a diff against the prior set.
    pub delegate_agent_ids: Vec<String>,
    pub skill_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiAgentMemoryUpdate {
    pub memory_md: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiSkill {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    /// What the model sees up front, on every agent this skill is
    /// attached to, before it decides whether to call `use_skill`.
    pub description: String,
    /// Only ever returned via the `use_skill` tool result - never
    /// injected into a system prompt directly.
    pub instructions_md: String,
    pub is_active: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiSkillInput {
    pub name: String,
    pub description: String,
    pub instructions_md: String,
}
