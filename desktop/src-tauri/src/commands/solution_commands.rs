use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::solution::{Solution, SolutionDetail, SolutionInput, SolutionMemberInput, SolutionUpdate};
use lanesra_core::services::{industry_package_service, solution_service};

#[tauri::command]
pub fn list_solutions(state: State<AppState>) -> AppResult<Vec<Solution>> {
    let conn = state.conn.lock().unwrap();
    solution_service::list_for_workspace(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn get_solution_detail(state: State<AppState>, id: String) -> AppResult<SolutionDetail> {
    let conn = state.conn.lock().unwrap();
    solution_service::get_detail(&conn, &require_workspace_id(&conn)?, &id)
}

#[tauri::command]
pub fn create_solution(state: State<AppState>, input: SolutionInput) -> AppResult<Solution> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    solution_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_solution(state: State<AppState>, id: String, input: SolutionUpdate) -> AppResult<Solution> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    solution_service::update(&conn, &workspace_id, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_solution(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    solution_service::delete(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

/// Curates one existing workspace component into a Solution's membership.
#[tauri::command]
pub fn add_solution_component(state: State<AppState>, solution_id: String, input: SolutionMemberInput) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    solution_service::add_component(&conn, &workspace_id, &solution_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn remove_solution_component(state: State<AppState>, solution_id: String, artifact_type: String, metadata_id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    solution_service::remove_component(&conn, &workspace_id, &solution_id, &artifact_type, &metadata_id, current_actor(&state).as_deref())
}

/// Builds a `.lanesra`-style manifest scoped to exactly this Solution's
/// curated members - the "export it" step of "build a solution in test,
/// export it, import it in prod". The resulting JSON is handed off the
/// same way any other package manifest is (download, then Admin → App
/// Catalog → Import in the target workspace).
#[tauri::command]
pub fn export_solution(state: State<AppState>, solution_id: String) -> AppResult<String> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    industry_package_service::export_solution(&conn, &workspace_id, &solution_id, current_actor(&state).as_deref())
}
