use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::invoice::{Invoice, InvoiceInput, InvoiceWithLines, PaymentInput};
use lanesra_core::services::invoice_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_invoices(state: State<AppState>) -> AppResult<Vec<Invoice>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    invoice_service::list(&conn, &workspace_id)
}

#[tauri::command]
pub fn get_invoice(state: State<AppState>, id: String) -> AppResult<InvoiceWithLines> {
    let conn = state.conn.lock().unwrap();
    invoice_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_invoice(state: State<AppState>, input: InvoiceInput) -> AppResult<InvoiceWithLines> {
    let conn = state.conn.lock().unwrap();
    invoice_service::create(&conn, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn issue_invoice(state: State<AppState>, id: String) -> AppResult<InvoiceWithLines> {
    let conn = state.conn.lock().unwrap();
    invoice_service::issue(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn void_invoice(state: State<AppState>, id: String) -> AppResult<InvoiceWithLines> {
    let conn = state.conn.lock().unwrap();
    invoice_service::void(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn record_invoice_payment(
    state: State<AppState>,
    id: String,
    payment: PaymentInput,
) -> AppResult<InvoiceWithLines> {
    let conn = state.conn.lock().unwrap();
    invoice_service::record_payment(&conn, &id, &payment, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn refresh_overdue_invoices(state: State<AppState>) -> AppResult<usize> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    invoice_service::refresh_overdue(&conn, &workspace_id)
}
