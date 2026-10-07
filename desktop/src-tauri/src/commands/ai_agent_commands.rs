//! AI & Agentic Layer, Phase 6: Tauri commands for `ai_agent_service` -
//! plain sync CRUD over Agents and Skills, same shape as every other
//! admin-builder command module. Running an agent (chat) is
//! `chat_commands::send_agent_message` instead - genuinely async.

use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::ai::{AiAgentModelRouting, AiTokenUsageSummary};
use lanesra_core::models::ai_agent::{AiAgentDefinition, AiAgentGuardrailsUpdate, AiAgentInput, AiAgentMemoryUpdate, AiAgentMemorySnapshot, AiAgentVersion, AiAgentVersionInput, AiSkill, AiSkillInput};
use lanesra_core::models::agent_access_inspection::AgentAccessInspection;
use lanesra_core::models::ai_agent_policy::{AiAgentPolicy, AiAgentPolicyInput};
use lanesra_core::models::ai_approval::{AiApproval, AiApprovalInput, AiApprovalResolution};
use lanesra_core::models::ai_tool_registry::{AiToolRegistryOverride, AiToolRegistryOverrideInput};
use lanesra_core::services::{agent_access_inspector_service, agent_version_service, ai_agent_service, approval_service, policy_engine_service, tool_registry_service};

#[tauri::command]
pub fn list_ai_agents(state: State<AppState>, active_only: bool) -> AppResult<Vec<AiAgentDefinition>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_agent_service::list(&conn, &workspace_id, active_only)
}

#[tauri::command]
pub fn create_ai_agent(state: State<AppState>, input: AiAgentInput) -> AppResult<AiAgentDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_agent_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_ai_agent(state: State<AppState>, id: String, input: AiAgentInput) -> AppResult<AiAgentDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_agent_service::update(&conn, &id, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_ai_agent_active(state: State<AppState>, id: String, is_active: bool) -> AppResult<AiAgentDefinition> {
    let conn = state.conn.lock().unwrap();
    ai_agent_service::set_active(&conn, &id, is_active, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_ai_agent_memory(state: State<AppState>, id: String, input: AiAgentMemoryUpdate) -> AppResult<AiAgentDefinition> {
    let conn = state.conn.lock().unwrap();
    ai_agent_service::set_memory(&conn, &id, &input.memory_md, current_actor(&state).as_deref())
}

/// Phase 7b: most-recent-first history of this agent's `memory_md`
/// changes - see `ai_agent_repo::update_memory`'s own doc comment on
/// where each snapshot comes from.
#[tauri::command]
pub fn list_ai_agent_memory_history(state: State<AppState>, id: String) -> AppResult<Vec<AiAgentMemorySnapshot>> {
    let conn = state.conn.lock().unwrap();
    ai_agent_service::list_memory_history(&conn, &id, current_actor(&state).as_deref())
}

/// Phase 7c: an agent's operational-boundary statement.
#[tauri::command]
pub fn set_ai_agent_guardrails(state: State<AppState>, id: String, input: AiAgentGuardrailsUpdate) -> AppResult<AiAgentDefinition> {
    let conn = state.conn.lock().unwrap();
    ai_agent_service::set_guardrails(&conn, &id, &input.guardrails_md, current_actor(&state).as_deref())
}

/// Phase 7a: an agent's Gateway routing policy - `routing: None` clears
/// it, returning this agent to the plain workspace-default behavior.
#[tauri::command]
pub fn set_ai_agent_model_routing(state: State<AppState>, id: String, routing: Option<AiAgentModelRouting>) -> AppResult<AiAgentDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_agent_service::set_model_routing(&conn, &id, &workspace_id, routing, current_actor(&state).as_deref())
}

/// Agent Access Governance (issue #245): an agent's own bound identity -
/// `acts_as_user_id: None` clears it, returning this agent to today's
/// unscoped record-write behavior.
#[tauri::command]
pub fn set_ai_agent_acts_as(state: State<AppState>, id: String, acts_as_user_id: Option<String>) -> AppResult<AiAgentDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_agent_service::set_acts_as(&conn, &id, &workspace_id, acts_as_user_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn get_ai_agent_token_usage(state: State<AppState>, id: String) -> AppResult<AiTokenUsageSummary> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_agent_service::token_usage_today(&conn, &id, &workspace_id)
}

// --- AI Agent Platform v2, Phase 1: version lifecycle + approvals -----

#[tauri::command]
pub fn list_ai_agent_versions(state: State<AppState>, agent_id: String) -> AppResult<Vec<AiAgentVersion>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_version_service::list_versions(&conn, &agent_id, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn create_ai_agent_version_draft(state: State<AppState>, agent_id: String, input: AiAgentVersionInput) -> AppResult<AiAgentVersion> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_version_service::create_draft(&conn, &agent_id, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_ai_agent_version_draft(state: State<AppState>, agent_id: String, version_id: String, input: AiAgentVersionInput) -> AppResult<AiAgentVersion> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_version_service::update_draft(&conn, &agent_id, &workspace_id, &version_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn transition_ai_agent_version_status(state: State<AppState>, agent_id: String, version_id: String, new_status: String) -> AppResult<AiAgentVersion> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_version_service::transition_status(&conn, &agent_id, &workspace_id, &version_id, &new_status, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_ai_approvals(state: State<AppState>, status: Option<String>) -> AppResult<Vec<AiApproval>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    approval_service::list(&conn, &workspace_id, status.as_deref(), current_actor(&state).as_deref())
}

#[tauri::command]
pub fn create_ai_approval(state: State<AppState>, input: AiApprovalInput) -> AppResult<AiApproval> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    approval_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn resolve_ai_approval(state: State<AppState>, id: String, resolution: AiApprovalResolution) -> AppResult<AiApproval> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    approval_service::resolve(&conn, &id, &workspace_id, &resolution, current_actor(&state).as_deref())
}

// --- AI Agent Platform v2, Phase 2: Policy Engine + Tool Registry -----

/// `agent_id: None` reads/writes the workspace-wide default policy.
#[tauri::command]
pub fn get_ai_agent_policy(state: State<AppState>, agent_id: Option<String>) -> AppResult<Option<AiAgentPolicy>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    policy_engine_service::get_policy(&conn, &workspace_id, agent_id.as_deref(), current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_ai_agent_policies(state: State<AppState>) -> AppResult<Vec<AiAgentPolicy>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    policy_engine_service::list_policies(&conn, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn upsert_ai_agent_policy(state: State<AppState>, agent_id: Option<String>, input: AiAgentPolicyInput) -> AppResult<AiAgentPolicy> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    policy_engine_service::upsert_policy(&conn, &workspace_id, agent_id.as_deref(), &input, current_actor(&state).as_deref())
}

/// Agent Access Governance (issue #245): the Agent Access Inspector -
/// `object_key`/`record_id` only matter for a record-write tool name,
/// `simulate_as_user_id` only matters when this agent has no `acts_as`
/// of its own and this workspace's policy enforces record access at all
/// - see `agent_access_inspector_service::inspect`'s own doc comment.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn inspect_agent_access(
    state: State<AppState>,
    agent_id: String,
    tool_name: String,
    object_key: Option<String>,
    record_id: Option<String>,
    simulate_as_user_id: Option<String>,
) -> AppResult<AgentAccessInspection> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_access_inspector_service::inspect(
        &conn,
        &workspace_id,
        &agent_id,
        &tool_name,
        object_key.as_deref(),
        record_id.as_deref(),
        simulate_as_user_id.as_deref(),
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn list_ai_tool_registry_overrides(state: State<AppState>) -> AppResult<Vec<AiToolRegistryOverride>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    tool_registry_service::list_overrides(&conn, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_ai_tool_registry_override(state: State<AppState>, input: AiToolRegistryOverrideInput) -> AppResult<AiToolRegistryOverride> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    tool_registry_service::set_override(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn clear_ai_tool_registry_override(state: State<AppState>, tool_name: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    tool_registry_service::clear_override(&conn, &workspace_id, &tool_name, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_ai_skills(state: State<AppState>, active_only: bool) -> AppResult<Vec<AiSkill>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_agent_service::list_skills(&conn, &workspace_id, active_only)
}

#[tauri::command]
pub fn create_ai_skill(state: State<AppState>, input: AiSkillInput) -> AppResult<AiSkill> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_agent_service::create_skill(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_ai_skill(state: State<AppState>, id: String, input: AiSkillInput) -> AppResult<AiSkill> {
    let conn = state.conn.lock().unwrap();
    ai_agent_service::update_skill(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_ai_skill_active(state: State<AppState>, id: String, is_active: bool) -> AppResult<AiSkill> {
    let conn = state.conn.lock().unwrap();
    ai_agent_service::set_skill_active(&conn, &id, is_active, current_actor(&state).as_deref())
}
