//! Next-Gen program, Domain A, FND-02: Tauri commands for
//! `glossary_service` (Business Glossary terms).

use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::semantic::{BusinessGlossaryTerm, BusinessGlossaryTermInput};
use lanesra_core::services::glossary_service;

#[tauri::command]
pub fn list_glossary_terms(state: State<AppState>, active_only: bool) -> AppResult<Vec<BusinessGlossaryTerm>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    glossary_service::list(&conn, &workspace_id, active_only)
}

#[tauri::command]
pub fn create_glossary_term(state: State<AppState>, input: BusinessGlossaryTermInput) -> AppResult<BusinessGlossaryTerm> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    glossary_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_glossary_term(state: State<AppState>, id: String, input: BusinessGlossaryTermInput) -> AppResult<BusinessGlossaryTerm> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    glossary_service::update(&conn, &id, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn deactivate_glossary_term(state: State<AppState>, id: String) -> AppResult<BusinessGlossaryTerm> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    glossary_service::deactivate(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}
