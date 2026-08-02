use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::domain::AppResult;
use crate::models::contract::{Contract, ContractInput};
use crate::services::contract_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_contracts(state: State<AppState>) -> AppResult<Vec<Contract>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    contract_service::list(&conn, &workspace_id)
}

#[tauri::command]
pub fn list_contracts_by_company(state: State<AppState>, company_id: String) -> AppResult<Vec<Contract>> {
    let conn = state.conn.lock().unwrap();
    contract_service::list_by_company(&conn, &company_id)
}

#[tauri::command]
pub fn get_contract(state: State<AppState>, id: String) -> AppResult<Contract> {
    let conn = state.conn.lock().unwrap();
    contract_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_contract(state: State<AppState>, input: ContractInput) -> AppResult<Contract> {
    let conn = state.conn.lock().unwrap();
    contract_service::create(&conn, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_contract(state: State<AppState>, id: String, input: ContractInput) -> AppResult<Contract> {
    let conn = state.conn.lock().unwrap();
    contract_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn archive_contract(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    contract_service::archive(&conn, &id, current_actor(&state).as_deref())
}
