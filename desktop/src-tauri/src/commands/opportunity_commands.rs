use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::opportunity::{
    Opportunity, OpportunityInput, OpportunityProduct, OpportunityProductInput,
};
use lanesra_core::services::opportunity_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_opportunities(state: State<AppState>) -> AppResult<Vec<Opportunity>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    opportunity_service::list(&conn, &workspace_id)
}

#[tauri::command]
pub fn list_opportunities_by_company(
    state: State<AppState>,
    company_id: String,
) -> AppResult<Vec<Opportunity>> {
    let conn = state.conn.lock().unwrap();
    opportunity_service::list_by_company(&conn, &company_id)
}

#[tauri::command]
pub fn get_opportunity(state: State<AppState>, id: String) -> AppResult<Opportunity> {
    let conn = state.conn.lock().unwrap();
    opportunity_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_opportunity(state: State<AppState>, input: OpportunityInput) -> AppResult<Opportunity> {
    let conn = state.conn.lock().unwrap();
    opportunity_service::create(&conn, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_opportunity(
    state: State<AppState>,
    id: String,
    input: OpportunityInput,
) -> AppResult<Opportunity> {
    let conn = state.conn.lock().unwrap();
    opportunity_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn archive_opportunity(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    opportunity_service::archive(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_opportunity_products(
    state: State<AppState>,
    opportunity_id: String,
    products: Vec<OpportunityProductInput>,
) -> AppResult<Vec<OpportunityProduct>> {
    let conn = state.conn.lock().unwrap();
    opportunity_service::set_products(&conn, &opportunity_id, &products, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_opportunity_products(
    state: State<AppState>,
    opportunity_id: String,
) -> AppResult<Vec<OpportunityProduct>> {
    let conn = state.conn.lock().unwrap();
    opportunity_service::list_products(&conn, &opportunity_id)
}
