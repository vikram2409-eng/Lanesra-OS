use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::business_rule::{BusinessRule, BusinessRuleInput, BusinessRuleUpdate, BusinessRuleVersion};
use lanesra_core::models::custom_field::CustomFieldValues;
use lanesra_core::services::business_rule_service;
use lanesra_core::services::business_rule_service::RuleEvaluation;

#[tauri::command]
pub fn list_business_rules(state: State<AppState>, entity_type: String, active_only: bool) -> AppResult<Vec<BusinessRule>> {
    let conn = state.conn.lock().unwrap();
    business_rule_service::list_rules(&conn, &require_workspace_id(&conn)?, &entity_type, active_only)
}

#[tauri::command]
pub fn create_business_rule(state: State<AppState>, input: BusinessRuleInput) -> AppResult<BusinessRule> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    business_rule_service::create_rule(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_business_rule(state: State<AppState>, id: String, input: BusinessRuleUpdate) -> AppResult<BusinessRule> {
    let conn = state.conn.lock().unwrap();
    business_rule_service::update_rule(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn test_business_rules(state: State<AppState>, entity_type: String, context: CustomFieldValues) -> AppResult<RuleEvaluation> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    business_rule_service::test_rules(&conn, &workspace_id, &entity_type, &context, current_actor(&state).as_deref())
}

/// Admin UX polish (spec §10).
#[tauri::command]
pub fn duplicate_business_rule(state: State<AppState>, id: String) -> AppResult<BusinessRule> {
    let conn = state.conn.lock().unwrap();
    business_rule_service::duplicate_rule(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_business_rule_versions(state: State<AppState>, rule_id: String) -> AppResult<Vec<BusinessRuleVersion>> {
    let conn = state.conn.lock().unwrap();
    business_rule_service::list_versions(&conn, &rule_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn restore_business_rule_version(state: State<AppState>, rule_id: String, version_id: String) -> AppResult<BusinessRule> {
    let conn = state.conn.lock().unwrap();
    business_rule_service::restore_version(&conn, &rule_id, &version_id, current_actor(&state).as_deref())
}
