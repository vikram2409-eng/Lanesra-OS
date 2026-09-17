use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::access_role::Capability;
use lanesra_core::models::contract::{Contract, ContractInput};
use lanesra_core::services::{access_service, contract_service};
use crate::state::AppState;

#[tauri::command]
pub fn list_contracts(state: State<AppState>) -> AppResult<Vec<Contract>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let mut contracts = contract_service::list(&conn, &workspace_id)?;
    let visible = access_service::filter_visible(&conn, current_actor(&state).as_deref(), "Contract", contracts.iter().map(|c| c.id.as_str()))?;
    contracts.retain(|c| visible.contains(&c.id));
    Ok(contracts)
}

#[tauri::command]
pub fn list_contracts_by_company(state: State<AppState>, company_id: String) -> AppResult<Vec<Contract>> {
    let conn = state.conn.lock().unwrap();
    let mut contracts = contract_service::list_by_company(&conn, &company_id)?;
    let visible = access_service::filter_visible(&conn, current_actor(&state).as_deref(), "Contract", contracts.iter().map(|c| c.id.as_str()))?;
    contracts.retain(|c| visible.contains(&c.id));
    Ok(contracts)
}

#[tauri::command]
pub fn get_contract(state: State<AppState>, id: String) -> AppResult<Contract> {
    let conn = state.conn.lock().unwrap();
    access_service::require_capability(&conn, current_actor(&state).as_deref(), "Contract", Capability::Read, Some(&id))?;
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
