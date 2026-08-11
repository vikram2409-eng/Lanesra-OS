pub mod auth_commands;
pub mod backup_commands;
pub mod business_rule_commands;
pub mod company_commands;
pub mod contact_commands;
pub mod contract_commands;
pub mod custom_field_commands;
pub mod custom_object_commands;
pub mod custom_record_commands;
pub mod custom_report_commands;
pub mod dashboard_commands;
pub mod invoice_commands;
pub mod notification_commands;
pub mod numbering_commands;
pub mod opportunity_commands;
pub mod order_commands;
pub mod product_commands;
pub mod quote_commands;
pub mod relationship_commands;
pub mod report_commands;
pub mod status_transition_commands;
pub mod task_commands;
pub mod user_commands;
pub mod workflow_commands;
pub mod workspace_commands;

use rusqlite::Connection;

use lanesra_core::domain::{AppError, AppResult};
use lanesra_core::repositories::workspace_repo;

use crate::state::AppState;

pub(crate) fn require_workspace_id(conn: &Connection) -> AppResult<String> {
    workspace_repo::get_current(conn)?
        .map(|w| w.id)
        .ok_or_else(|| AppError::Validation("No workspace has been set up yet".into()))
}

pub(crate) fn current_actor(state: &AppState) -> Option<String> {
    state.session_user_id.lock().unwrap().clone()
}
