use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::{AppError, AppResult};
use lanesra_core::models::voice::{
    ConfirmVoicePlanInput, SetVoicePinInput, VoiceActivityEntry, VoiceExecutionResult, VoicePolicyBinding, VoicePolicyBindingInput,
    VoicePreferencesInput, VoiceProviderProfile, VoiceSession, VoiceUserSettings,
};
use lanesra_core::services::voice_execution_service::VoiceCommandOutcome;
use lanesra_core::services::{voice_audit_service, voice_execution_service, voice_policy_service, voice_provider_service, voice_session_service};

fn require_actor(state: &State<AppState>) -> AppResult<String> {
    current_actor(state).ok_or_else(|| AppError::Validation("Not authenticated".into()))
}

#[tauri::command]
pub fn get_voice_settings(state: State<AppState>) -> AppResult<VoiceUserSettings> {
    let conn = state.conn.lock().unwrap();
    voice_session_service::get_settings(&conn, &require_actor(&state)?)
}

#[tauri::command]
pub fn set_voice_pin(state: State<AppState>, input: SetVoicePinInput) -> AppResult<VoiceUserSettings> {
    let conn = state.conn.lock().unwrap();
    voice_session_service::set_pin(&conn, &require_actor(&state)?, &input)
}

#[tauri::command]
pub fn update_voice_preferences(state: State<AppState>, input: VoicePreferencesInput) -> AppResult<VoiceUserSettings> {
    let conn = state.conn.lock().unwrap();
    voice_session_service::update_preferences(&conn, &require_actor(&state)?, &input)
}

#[tauri::command]
pub fn unlock_voice_session(state: State<AppState>, pin: String) -> AppResult<VoiceSession> {
    let conn = state.conn.lock().unwrap();
    voice_session_service::unlock(&conn, &require_actor(&state)?, &pin)
}

#[tauri::command]
pub fn get_current_voice_session(state: State<AppState>) -> AppResult<Option<VoiceSession>> {
    let conn = state.conn.lock().unwrap();
    voice_session_service::current_session(&conn, &require_actor(&state)?)
}

#[tauri::command]
pub fn extend_voice_session(state: State<AppState>, session_id: String) -> AppResult<VoiceSession> {
    let conn = state.conn.lock().unwrap();
    voice_session_service::extend(&conn, &session_id, &require_actor(&state)?)
}

#[tauri::command]
pub fn expire_voice_session(state: State<AppState>, session_id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    voice_session_service::expire(&conn, &session_id, &require_actor(&state)?)
}

#[tauri::command]
pub fn set_voice_session_context(state: State<AppState>, session_id: String, object_key: Option<String>, record_id: Option<String>) -> AppResult<VoiceSession> {
    let conn = state.conn.lock().unwrap();
    voice_session_service::set_context(&conn, &session_id, &require_actor(&state)?, object_key.as_deref(), record_id.as_deref())
}

#[tauri::command]
pub fn reset_voice_conversation(state: State<AppState>, session_id: String) -> AppResult<VoiceSession> {
    let conn = state.conn.lock().unwrap();
    voice_session_service::reset_conversation(&conn, &session_id, &require_actor(&state)?)
}

// Voice-First Mode, PR 2: a RUN_AGENT/RUN_PIPELINE command may call out to
// an LLM provider (`chat_service::send_agent_message`/
// `ai_orchestration_service::run_manual`), so both of these are genuinely
// async now - same reasoning, and the same `run_with_own_connection`
// pattern, as `chat_commands::send_agent_message`'s own doc comment. Every
// other voice command (unlock, context, activity, policy, ...) stays
// plain sync above.
#[tauri::command]
pub async fn submit_voice_command(state: State<'_, AppState>, session_id: String, transcript: String, language: String, speech_confidence: Option<f64>) -> AppResult<VoiceCommandOutcome> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let user_id = require_actor(&state)?;
    run_with_own_connection(db_path, move |conn| async move {
        voice_execution_service::submit_command(&conn, &session_id, &user_id, &transcript, &language, speech_confidence, &master_key).await
    })
    .await
}

#[tauri::command]
pub async fn confirm_voice_plan(state: State<'_, AppState>, session_id: String, input: ConfirmVoicePlanInput) -> AppResult<VoiceExecutionResult> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let user_id = require_actor(&state)?;
    run_with_own_connection(db_path, move |conn| async move {
        voice_execution_service::confirm_plan(&conn, &session_id, &user_id, &input, &master_key).await
    })
    .await
}

#[tauri::command]
pub fn undo_voice_execution(state: State<AppState>, execution_id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    voice_execution_service::undo(&conn, &execution_id, &require_actor(&state)?)
}

#[tauri::command]
pub fn list_my_voice_activity(state: State<AppState>, limit: i64) -> AppResult<Vec<VoiceActivityEntry>> {
    let conn = state.conn.lock().unwrap();
    voice_audit_service::list_my_voice_activity(&conn, &require_actor(&state)?, limit)
}

#[tauri::command]
pub fn search_voice_activity(state: State<AppState>, limit: i64) -> AppResult<Vec<VoiceActivityEntry>> {
    let conn = state.conn.lock().unwrap();
    voice_audit_service::search_voice_activity(&conn, &require_workspace_id(&conn)?, current_actor(&state).as_deref(), limit)
}

#[tauri::command]
pub fn list_voice_policy_bindings(state: State<AppState>) -> AppResult<Vec<VoicePolicyBinding>> {
    let conn = state.conn.lock().unwrap();
    voice_policy_service::list_policy_bindings(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn upsert_voice_policy_binding(state: State<AppState>, input: VoicePolicyBindingInput) -> AppResult<VoicePolicyBinding> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    voice_policy_service::upsert_policy_binding(&conn, &workspace_id, current_actor(&state).as_deref(), &input)
}

#[tauri::command]
pub fn list_voice_providers(state: State<AppState>) -> AppResult<Vec<VoiceProviderProfile>> {
    let conn = state.conn.lock().unwrap();
    voice_provider_service::list_providers(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn voice_provider_health_check(state: State<AppState>, provider_id: String) -> AppResult<VoiceProviderProfile> {
    let conn = state.conn.lock().unwrap();
    voice_provider_service::health_check(&conn, &require_workspace_id(&conn)?, &provider_id)
}
