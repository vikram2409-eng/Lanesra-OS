//! WP-0108/WP-0109: Tauri commands for Agent Identities and their
//! credentials, and for choosing the identity a trigger's runs act as.
//! Every one is Administrator-only in `agent_identity_service`.

use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::agent_identity::{
    AgentCredential, AgentIdentity, AgentIdentityInput, IssuedAgentCredential,
};
use lanesra_core::models::ai_agent_pipeline::AiAgentTrigger;
use lanesra_core::services::{agent_identity_service, ai_orchestration_service};

#[tauri::command]
pub fn list_agent_identities(state: State<AppState>) -> AppResult<Vec<AgentIdentity>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_identity_service::list(&conn, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn create_agent_identity(
    state: State<AppState>,
    input: AgentIdentityInput,
) -> AppResult<AgentIdentity> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_identity_service::create(
        &conn,
        &workspace_id,
        &input,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn update_agent_identity(
    state: State<AppState>,
    id: String,
    input: AgentIdentityInput,
) -> AppResult<AgentIdentity> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_identity_service::update(
        &conn,
        &workspace_id,
        &id,
        &input,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn set_agent_identity_active(
    state: State<AppState>,
    id: String,
    active: bool,
) -> AppResult<AgentIdentity> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_identity_service::set_active(
        &conn,
        &workspace_id,
        &id,
        active,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn delete_agent_identity(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_identity_service::delete(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_agent_credentials(
    state: State<AppState>,
    identity_id: String,
) -> AppResult<Vec<AgentCredential>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_identity_service::list_credentials(
        &conn,
        &workspace_id,
        &identity_id,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn issue_agent_credential(
    state: State<AppState>,
    identity_id: String,
    label: String,
    lifetime_hours: Option<i64>,
) -> AppResult<IssuedAgentCredential> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_identity_service::issue_credential(
        &conn,
        &workspace_id,
        &identity_id,
        &label,
        lifetime_hours,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn rotate_agent_credential(
    state: State<AppState>,
    credential_id: String,
    lifetime_hours: Option<i64>,
) -> AppResult<IssuedAgentCredential> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_identity_service::rotate_credential(
        &conn,
        &workspace_id,
        &credential_id,
        lifetime_hours,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn revoke_agent_credential(state: State<AppState>, credential_id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    agent_identity_service::revoke_credential(
        &conn,
        &workspace_id,
        &credential_id,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn set_ai_agent_trigger_identity(
    state: State<AppState>,
    id: String,
    agent_identity_id: Option<String>,
) -> AppResult<AiAgentTrigger> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_orchestration_service::set_trigger_identity(
        &conn,
        &workspace_id,
        &id,
        agent_identity_id.as_deref(),
        current_actor(&state).as_deref(),
    )
}
