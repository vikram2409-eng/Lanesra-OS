//! AI & Agentic Layer, Phase 6: Tauri commands for `ai_agent_service` -
//! plain sync CRUD over Agents and Skills, same shape as every other
//! admin-builder command module. Running an agent (chat) is
//! `chat_commands::send_agent_message` instead - genuinely async.

use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::ai_agent::{AiAgentDefinition, AiAgentInput, AiAgentMemoryUpdate, AiSkill, AiSkillInput};
use lanesra_core::services::ai_agent_service;

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
