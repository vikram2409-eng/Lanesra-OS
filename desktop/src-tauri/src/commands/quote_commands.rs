use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::domain::AppResult;
use crate::models::order::OrderWithLines;
use crate::models::quote::{Quote, QuoteInput, QuoteWithLines};
use crate::services::quote_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_quotes(state: State<AppState>) -> AppResult<Vec<Quote>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    quote_service::list(&conn, &workspace_id)
}

#[tauri::command]
pub fn get_quote(state: State<AppState>, id: String) -> AppResult<QuoteWithLines> {
    let conn = state.conn.lock().unwrap();
    quote_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_quote(state: State<AppState>, input: QuoteInput) -> AppResult<QuoteWithLines> {
    let conn = state.conn.lock().unwrap();
    quote_service::create(&conn, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_quote_status(state: State<AppState>, id: String, status: String) -> AppResult<QuoteWithLines> {
    let conn = state.conn.lock().unwrap();
    quote_service::set_status(&conn, &id, &status, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn convert_quote_to_order(state: State<AppState>, quote_id: String) -> AppResult<OrderWithLines> {
    let conn = state.conn.lock().unwrap();
    quote_service::convert_to_order(&conn, &quote_id, current_actor(&state).as_deref())
}
