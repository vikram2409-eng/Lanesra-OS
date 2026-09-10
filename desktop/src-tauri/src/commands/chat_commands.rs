//! AI & Agentic Layer, Phase 5: Tauri commands for `chat_service`.
//! `send_chat_message` is genuinely async (it may call out to an LLM
//! provider), so it goes through `run_with_own_connection` for the same
//! reason `ai_commands::test_ai_key` and `agent_commands::ask_report` do -
//! see `integration_commands`'s own doc comment. `get_chat_history` is
//! plain sync, like any other read.

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::{AppError, AppResult};
use lanesra_core::models::chat::ChatMessage;
use lanesra_core::services::chat_service;

#[tauri::command]
pub async fn send_chat_message(state: State<'_, AppState>, mode: String, text: String) -> AppResult<Vec<ChatMessage>> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, user_id) = {
        let conn = state.conn.lock().unwrap();
        let workspace_id = require_workspace_id(&conn)?;
        let user_id = current_actor(&state).ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
        (workspace_id, user_id)
    };
    run_with_own_connection(db_path, move |conn| async move {
        chat_service::send_message(&conn, &workspace_id, &master_key, &user_id, &mode, &text).await
    })
    .await
}

#[tauri::command]
pub fn get_chat_history(state: State<AppState>, mode: String) -> AppResult<Vec<ChatMessage>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let user_id = current_actor(&state).ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    chat_service::get_history(&conn, &workspace_id, &user_id, &mode)
}

/// AI & Agentic Layer, Phase 6: chatting with one specific named Agent -
/// same shape as `send_chat_message`/`get_chat_history` above.
#[tauri::command]
pub async fn send_agent_message(state: State<'_, AppState>, agent_id: String, text: String) -> AppResult<Vec<ChatMessage>> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, user_id) = {
        let conn = state.conn.lock().unwrap();
        let workspace_id = require_workspace_id(&conn)?;
        let user_id = current_actor(&state).ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
        (workspace_id, user_id)
    };
    run_with_own_connection(db_path, move |conn| async move {
        chat_service::send_agent_message(&conn, &workspace_id, &master_key, &user_id, &agent_id, &text).await
    })
    .await
}

#[tauri::command]
pub fn get_agent_chat_history(state: State<AppState>, agent_id: String) -> AppResult<Vec<ChatMessage>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let user_id = current_actor(&state).ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    chat_service::get_agent_history(&conn, &workspace_id, &user_id, &agent_id)
}
