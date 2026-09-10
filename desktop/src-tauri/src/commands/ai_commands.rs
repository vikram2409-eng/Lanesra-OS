//! AI & Agentic Layer, Phase 1: Tauri commands for `ai_service`. Mirrors
//! `integration_commands.rs`'s own shape 1:1 with `server::dispatch`'s
//! `get_ai_settings`/`save_ai_settings` arms - `test_ai_key` is the one
//! genuinely-async operation, run via `run_with_own_connection` for the
//! same `&rusqlite::Connection` isn't `Send` reason that module's own doc
//! comment explains.

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id, resolve_master_key};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::ai::{AiDailyTokenBudgetInput, AiSettings, AiSettingsInput, AiTestResult, AiTokenUsageSummary};
use lanesra_core::services::ai_service;

#[tauri::command]
pub fn get_ai_settings(state: State<AppState>) -> AppResult<AiSettings> {
    let conn = state.conn.lock().unwrap();
    ai_service::get_settings(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn save_ai_settings(state: State<AppState>, input: AiSettingsInput) -> AppResult<AiSettings> {
    let master_key = resolve_master_key(&state)?;
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_service::save_settings(&conn, &workspace_id, &master_key, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub async fn test_ai_key(state: State<'_, AppState>) -> AppResult<AiTestResult> {
    let master_key = resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let workspace_id = {
        let conn = state.conn.lock().unwrap();
        require_workspace_id(&conn)?
    };
    let actor = current_actor(&state);
    run_with_own_connection(db_path, move |conn| async move { ai_service::test_key(&conn, &workspace_id, &master_key, actor.as_deref()).await }).await
}

/// Phase 7a: the "System" tier of the Gateway's budget hierarchy.
#[tauri::command]
pub fn set_ai_daily_token_budget(state: State<AppState>, input: AiDailyTokenBudgetInput) -> AppResult<AiSettings> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_service::set_daily_token_budget(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn get_ai_token_usage_summary(state: State<AppState>) -> AppResult<AiTokenUsageSummary> {
    let conn = state.conn.lock().unwrap();
    ai_service::token_usage_today(&conn, &require_workspace_id(&conn)?)
}
