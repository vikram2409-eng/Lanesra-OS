use tauri::State;

use crate::commands::require_workspace_id;
use lanesra_core::domain::AppResult;
use lanesra_core::models::report::{ArAgingBucket, LostReasonBreakdown, ReportRange, RevenueByMonth, SalesByOwner, WinRateByOwner};
use lanesra_core::services::report_service;
use crate::state::AppState;

#[tauri::command]
pub fn report_revenue_by_month(state: State<AppState>, range: ReportRange) -> AppResult<Vec<RevenueByMonth>> {
    let conn = state.conn.lock().unwrap();
    report_service::revenue_by_month(&conn, &require_workspace_id(&conn)?, &range.from, &range.to)
}

#[tauri::command]
pub fn report_win_rate_by_owner(state: State<AppState>, range: ReportRange) -> AppResult<Vec<WinRateByOwner>> {
    let conn = state.conn.lock().unwrap();
    report_service::win_rate_by_owner(&conn, &require_workspace_id(&conn)?, &range.from, &range.to)
}

#[tauri::command]
pub fn report_lost_reasons(state: State<AppState>, range: ReportRange) -> AppResult<Vec<LostReasonBreakdown>> {
    let conn = state.conn.lock().unwrap();
    report_service::lost_reason_breakdown(&conn, &require_workspace_id(&conn)?, &range.from, &range.to)
}

#[tauri::command]
pub fn report_ar_aging(state: State<AppState>, as_of_date: Option<String>) -> AppResult<Vec<ArAgingBucket>> {
    let conn = state.conn.lock().unwrap();
    report_service::ar_aging(&conn, &require_workspace_id(&conn)?, &as_of_date)
}

#[tauri::command]
pub fn report_sales_by_owner(state: State<AppState>, range: ReportRange) -> AppResult<Vec<SalesByOwner>> {
    let conn = state.conn.lock().unwrap();
    report_service::sales_by_owner(&conn, &require_workspace_id(&conn)?, &range.from, &range.to)
}
