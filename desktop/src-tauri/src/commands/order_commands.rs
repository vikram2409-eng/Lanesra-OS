use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::invoice::InvoiceWithLines;
use lanesra_core::models::order::{Order, OrderInput, OrderWithLines};
use lanesra_core::services::order_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_orders(state: State<AppState>) -> AppResult<Vec<Order>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    order_service::list(&conn, &workspace_id)
}

#[tauri::command]
pub fn get_order(state: State<AppState>, id: String) -> AppResult<OrderWithLines> {
    let conn = state.conn.lock().unwrap();
    order_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_order(state: State<AppState>, input: OrderInput) -> AppResult<OrderWithLines> {
    let conn = state.conn.lock().unwrap();
    order_service::create(&conn, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_order_status(state: State<AppState>, id: String, status: String) -> AppResult<OrderWithLines> {
    let conn = state.conn.lock().unwrap();
    order_service::set_status(&conn, &id, &status, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn convert_order_to_invoice(state: State<AppState>, order_id: String) -> AppResult<InvoiceWithLines> {
    let conn = state.conn.lock().unwrap();
    order_service::convert_to_invoice(&conn, &order_id, current_actor(&state).as_deref())
}
