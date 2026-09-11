//! AI & Agentic Layer, Phase 7d: Tauri commands for `ai_eval_service` -
//! Suite CRUD and run history are plain sync; `run_ai_eval_suite` (a
//! real judge-model call per case) is genuinely async, same shape as
//! `ai_agent_pipeline_commands::run_ai_agent_pipeline`.

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::ai_eval::{AiEvalRun, AiEvalSuite, AiEvalSuiteInput};
use lanesra_core::services::ai_eval_service;

#[tauri::command]
pub fn list_ai_eval_suites(state: State<AppState>) -> AppResult<Vec<AiEvalSuite>> {
    let conn = state.conn.lock().unwrap();
    ai_eval_service::list_suites(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_ai_eval_suite(state: State<AppState>, input: AiEvalSuiteInput) -> AppResult<AiEvalSuite> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_eval_service::create_suite(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_ai_eval_suite(state: State<AppState>, id: String, input: AiEvalSuiteInput) -> AppResult<AiEvalSuite> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_eval_service::update_suite(&conn, &id, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_ai_eval_suite(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    ai_eval_service::delete_suite(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_ai_eval_runs(state: State<AppState>, suite_id: String, limit: i64) -> AppResult<Vec<AiEvalRun>> {
    let conn = state.conn.lock().unwrap();
    ai_eval_service::list_runs(&conn, &suite_id, limit)
}

#[tauri::command]
pub async fn run_ai_eval_suite(state: State<'_, AppState>, id: String) -> AppResult<AiEvalRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move { ai_eval_service::run_suite(&conn, &workspace_id, &master_key, &id, actor.as_deref()).await }).await
}
