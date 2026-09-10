//! AI & Agentic Layer, Phase 7a: Tauri commands for the Unified AI
//! Gateway - named `ai_providers` CRUD (`ai_provider_service`) and the
//! Gateway health view's read-only data (`ai_gateway_service`).
//! `test_ai_provider_key` is the one genuinely-async operation, run via
//! `run_with_own_connection` for the same reason `ai_commands::
//! test_ai_key` already is.

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id, resolve_master_key};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::ai::{AiGatewayFailoverEvent, AiProvider, AiProviderInput, AiTestResult};
use lanesra_core::services::{ai_gateway_service, ai_provider_service};

#[tauri::command]
pub fn list_ai_providers(state: State<AppState>, active_only: bool) -> AppResult<Vec<AiProvider>> {
    let conn = state.conn.lock().unwrap();
    ai_provider_service::list(&conn, &require_workspace_id(&conn)?, active_only)
}

#[tauri::command]
pub fn create_ai_provider(state: State<AppState>, input: AiProviderInput) -> AppResult<AiProvider> {
    let master_key = resolve_master_key(&state)?;
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_provider_service::create(&conn, &workspace_id, &master_key, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_ai_provider(state: State<AppState>, id: String, input: AiProviderInput) -> AppResult<AiProvider> {
    let master_key = resolve_master_key(&state)?;
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_provider_service::update(&conn, &id, &workspace_id, &master_key, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_ai_provider_active(state: State<AppState>, id: String, is_active: bool) -> AppResult<AiProvider> {
    let conn = state.conn.lock().unwrap();
    ai_provider_service::set_active(&conn, &id, is_active, current_actor(&state).as_deref())
}

#[tauri::command]
pub async fn test_ai_provider_key(state: State<'_, AppState>, id: String) -> AppResult<AiTestResult> {
    let master_key = resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let actor = current_actor(&state);
    run_with_own_connection(db_path, move |conn| async move { ai_provider_service::test_key(&conn, &id, &master_key, actor.as_deref()).await }).await
}

#[tauri::command]
pub fn list_ai_gateway_failover_events(state: State<AppState>, limit: i64) -> AppResult<Vec<AiGatewayFailoverEvent>> {
    let conn = state.conn.lock().unwrap();
    ai_gateway_service::recent_failover_events(&conn, &require_workspace_id(&conn)?, limit)
}
