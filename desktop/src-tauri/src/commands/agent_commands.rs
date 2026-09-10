//! AI & Agentic Layer, Phase 4: Tauri command for `agent_service`.
//! `ask_report` is genuinely async (a real outbound LLM call), so it
//! goes through `run_with_own_connection` for the same reason
//! `ai_commands::test_ai_key` does - see that module's own doc comment.

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{require_workspace_id, resolve_master_key};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::agent::{NlReportQuery, NlReportResult};
use lanesra_core::services::agent_service;

#[tauri::command]
pub async fn ask_report(state: State<'_, AppState>, query: NlReportQuery) -> AppResult<NlReportResult> {
    let master_key = resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let workspace_id = {
        let conn = state.conn.lock().unwrap();
        require_workspace_id(&conn)?
    };
    run_with_own_connection(db_path, move |conn| async move { agent_service::ask_report(&conn, &workspace_id, &master_key, &query).await }).await
}
