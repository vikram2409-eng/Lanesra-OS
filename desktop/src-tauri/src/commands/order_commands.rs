use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::access_role::Capability;
use lanesra_core::models::invoice::InvoiceWithLines;
use lanesra_core::models::order::{Order, OrderInput, OrderWithLines};
use lanesra_core::services::{access_service, order_service};
use crate::state::AppState;

#[tauri::command]
pub fn list_orders(state: State<AppState>) -> AppResult<Vec<Order>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let mut orders = order_service::list(&conn, &workspace_id)?;
    let visible = access_service::filter_visible(&conn, current_actor(&state).as_deref(), "Order", orders.iter().map(|o| o.id.as_str()))?;
    orders.retain(|o| visible.contains(&o.id));
    Ok(orders)
}

#[tauri::command]
pub fn get_order(state: State<AppState>, id: String) -> AppResult<OrderWithLines> {
    let conn = state.conn.lock().unwrap();
    access_service::require_capability(&conn, current_actor(&state).as_deref(), "Order", Capability::Read, Some(&id))?;
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
