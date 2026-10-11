//! Next-Gen program, Domain A (Intelligence Foundation), FND-03: Tauri
//! commands for `test_eval_service` - Test Case Definition CRUD is plain
//! sync; `run_tests`/`run_for_solution` (agent_eval/agent_team_eval cases
//! make a real model call) are genuinely async, same shape as
//! `ai_eval_commands::run_ai_eval_suite`.

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::test_eval::{TestCaseDefinition, TestCaseDefinitionInput, TestRun};
use lanesra_core::services::test_eval_service;

#[tauri::command]
pub fn list_test_case_definitions(state: State<AppState>, active_only: bool) -> AppResult<Vec<TestCaseDefinition>> {
    let conn = state.conn.lock().unwrap();
    test_eval_service::list(&conn, &require_workspace_id(&conn)?, active_only)
}

#[tauri::command]
pub fn create_test_case_definition(state: State<AppState>, input: TestCaseDefinitionInput) -> AppResult<TestCaseDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    test_eval_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_test_case_definition(state: State<AppState>, id: String, input: TestCaseDefinitionInput) -> AppResult<TestCaseDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    test_eval_service::update(&conn, &id, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn deactivate_test_case_definition(state: State<AppState>, id: String) -> AppResult<TestCaseDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    test_eval_service::deactivate(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_test_case_definition(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    test_eval_service::delete(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_test_runs(state: State<AppState>, limit: i64) -> AppResult<Vec<TestRun>> {
    let conn = state.conn.lock().unwrap();
    test_eval_service::list_runs(&conn, &require_workspace_id(&conn)?, limit)
}

#[tauri::command]
pub fn list_test_runs_for_solution(state: State<AppState>, solution_id: String, limit: i64) -> AppResult<Vec<TestRun>> {
    let conn = state.conn.lock().unwrap();
    test_eval_service::list_runs_for_solution(&conn, &solution_id, limit)
}

#[tauri::command]
pub fn get_test_run(state: State<AppState>, id: String) -> AppResult<Option<TestRun>> {
    let conn = state.conn.lock().unwrap();
    test_eval_service::get_run(&conn, &id)
}

#[tauri::command]
pub async fn run_tests(state: State<'_, AppState>, case_ids: Vec<String>) -> AppResult<TestRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        test_eval_service::run_tests(&conn, &workspace_id, &master_key, &case_ids, None, "manual", actor.as_deref()).await
    })
    .await
}

#[tauri::command]
pub async fn run_tests_for_solution(state: State<'_, AppState>, solution_id: String) -> AppResult<TestRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move { test_eval_service::run_for_solution(&conn, &workspace_id, &master_key, &solution_id, actor.as_deref()).await }).await
}
