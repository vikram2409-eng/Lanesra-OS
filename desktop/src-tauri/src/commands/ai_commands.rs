//! AI & Agentic Layer, Phase 1: Tauri commands for `ai_service`. Mirrors
//! `integration_commands.rs`'s own shape 1:1 with `server::dispatch`'s
//! `get_ai_settings`/`save_ai_settings` arms - `test_ai_key` is the one
//! genuinely-async operation, run via `run_with_own_connection` for the
//! same `&rusqlite::Connection` isn't `Send` reason that module's own doc
//! comment explains.

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id, resolve_master_key};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::ai::{
    AiDailyTokenBudgetInput, AiEmbeddingSettingsInput, AiObservabilitySettingsInput, AiSettings,
    AiSettingsInput, AiTestResult, AiTokenUsageSummary, VectorSearchStatus,
};
use lanesra_core::models::run_event::RunEvent;
use lanesra_core::models::run_trace::{RunSpan, SpanBody, TraceBodiesInput};
use lanesra_core::models::tool_limits::{
    AgentToolCap, AgentToolCapInput, StoredToolResult, ToolRateLimit, ToolRateLimitInput,
};
use lanesra_core::services::{
    ai_service, tool_limits_service, trace_service, vector_search_service,
};

#[tauri::command]
pub fn get_ai_settings(state: State<AppState>) -> AppResult<AiSettings> {
    let conn = state.conn.lock().unwrap();
    ai_service::get_settings(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn save_ai_settings(state: State<AppState>, input: AiSettingsInput) -> AppResult<AiSettings> {
    let master_key = resolve_master_key(&state)?;
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_service::save_settings(
        &conn,
        &workspace_id,
        &master_key,
        &input,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub async fn test_ai_key(state: State<'_, AppState>) -> AppResult<AiTestResult> {
    let master_key = resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let workspace_id = {
        let conn = state.conn.lock().unwrap();
        require_workspace_id(&conn)?
    };
    let actor = current_actor(&state);
    run_with_own_connection(db_path, move |conn| async move {
        ai_service::test_key(&conn, &workspace_id, &master_key, actor.as_deref()).await
    })
    .await
}

/// Phase 7a: the "System" tier of the Gateway's budget hierarchy.
#[tauri::command]
pub fn set_ai_daily_token_budget(
    state: State<AppState>,
    input: AiDailyTokenBudgetInput,
) -> AppResult<AiSettings> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_service::set_daily_token_budget(
        &conn,
        &workspace_id,
        &input,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn get_ai_token_usage_summary(state: State<AppState>) -> AppResult<AiTokenUsageSummary> {
    let conn = state.conn.lock().unwrap();
    ai_service::token_usage_today(&conn, &require_workspace_id(&conn)?)
}

/// Phase 7f: where a run's OTLP trace can be pushed - `push_ai_agent_run_
/// otlp` (a real outbound call) is its own async command, alongside
/// `ai_agent_pipeline_commands`'s other genuinely-async operations.
#[tauri::command]
pub fn set_ai_otlp_endpoint(
    state: State<AppState>,
    input: AiObservabilitySettingsInput,
) -> AppResult<AiSettings> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_service::set_otlp_endpoint(
        &conn,
        &workspace_id,
        &input,
        current_actor(&state).as_deref(),
    )
}

/// Phase 7g: same shape as `set_ai_otlp_endpoint` above -
/// `reindex_vector_search` (real outbound embedding calls) is its own
/// async command, alongside `ai_agent_pipeline_commands`'s other
/// genuinely-async operations.
#[tauri::command]
pub fn set_ai_embedding_model(
    state: State<AppState>,
    input: AiEmbeddingSettingsInput,
) -> AppResult<AiSettings> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_service::set_embedding_model(
        &conn,
        &workspace_id,
        &input,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn get_vector_search_status(state: State<AppState>) -> AppResult<VectorSearchStatus> {
    let conn = state.conn.lock().unwrap();
    vector_search_service::status(&conn, &require_workspace_id(&conn)?)
}

/// WP-0111: whether run traces keep prompt and output bodies (`off`,
/// `redacted` or `full`). Administrator only, and audited.
#[tauri::command]
pub fn set_ai_trace_bodies(
    state: State<AppState>,
    input: TraceBodiesInput,
) -> AppResult<AiSettings> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    trace_service::set_trace_bodies(
        &conn,
        &workspace_id,
        &input,
        current_actor(&state).as_deref(),
    )
}

/// WP-0111: a run's span tree, in the order the spans started.
#[tauri::command]
pub fn list_run_spans(
    state: State<AppState>,
    run_kind: String,
    run_id: String,
) -> AppResult<Vec<RunSpan>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    trace_service::spans_for_run(&conn, &workspace_id, &run_kind, &run_id)
}

/// WP-0111: a span's stored bodies, for an Administrator.
#[tauri::command]
pub fn get_run_span_bodies(state: State<AppState>, span_id: String) -> AppResult<Vec<SpanBody>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    trace_service::span_bodies(
        &conn,
        &workspace_id,
        &span_id,
        current_actor(&state).as_deref(),
    )
}

/// WP-0112: a run's stored events after `after_seq`, for an
/// Administrator. Live events arrive on the `run-event` channel.
#[tauri::command]
pub fn list_run_events(
    state: State<AppState>,
    run_id: String,
    after_seq: i64,
) -> AppResult<Vec<RunEvent>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    lanesra_core::services::run_event_service::list_run_events(
        &conn,
        &workspace_id,
        &run_id,
        after_seq,
        current_actor(&state).as_deref(),
    )
}

/// WP-0112 (ADR-0008 §5): forwards every published run event to the
/// window on the `run-event` channel, the same envelope Team Workspace
/// streams, while the signed-in user is an Administrator. Runs here on its
/// own thread for the life of the app.
pub fn forward_run_events(app: tauri::AppHandle) {
    use tauri::{Emitter, Manager};
    use tokio::sync::broadcast::error::RecvError;
    let mut rx = lanesra_core::services::run_event_service::subscribe();
    loop {
        match rx.blocking_recv() {
            Ok(event) => {
                let state = app.state::<AppState>();
                let viewer = current_actor(&state);
                let allowed = {
                    let conn = state.conn.lock().unwrap();
                    lanesra_core::services::run_event_service::require_viewer(
                        &conn,
                        viewer.as_deref(),
                    )
                    .is_ok()
                };
                if allowed {
                    let _ = app.emit("run-event", &*event);
                }
            }
            // Missed events stay stored; the window replays them with
            // `list_run_events`.
            Err(RecvError::Lagged(_)) => continue,
            Err(RecvError::Closed) => return,
        }
    }
}

/// WP-0140: an agent's tool caps (calls per step, failures before
/// disable, result size).
#[tauri::command]
pub fn list_agent_tool_caps(
    state: State<AppState>,
    agent_id: String,
) -> AppResult<Vec<AgentToolCap>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    tool_limits_service::list_agent_tool_caps(&conn, &workspace_id, &agent_id)
}

#[tauri::command]
pub fn set_agent_tool_caps(
    state: State<AppState>,
    agent_id: String,
    caps: Vec<AgentToolCapInput>,
) -> AppResult<Vec<AgentToolCap>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    tool_limits_service::set_agent_tool_caps(
        &conn,
        &workspace_id,
        &agent_id,
        &caps,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn reenable_agent_tool(
    state: State<AppState>,
    agent_id: String,
    tool_name: String,
) -> AppResult<Vec<AgentToolCap>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    tool_limits_service::reenable_agent_tool(
        &conn,
        &workspace_id,
        &agent_id,
        &tool_name,
        current_actor(&state).as_deref(),
    )
}

/// WP-0140: tool, connector and workspace rate limits.
#[tauri::command]
pub fn list_tool_rate_limits(state: State<AppState>) -> AppResult<Vec<ToolRateLimit>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    tool_limits_service::list_rate_limits(&conn, &workspace_id)
}

#[tauri::command]
pub fn save_tool_rate_limit(
    state: State<AppState>,
    input: ToolRateLimitInput,
) -> AppResult<Vec<ToolRateLimit>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    tool_limits_service::save_rate_limit(
        &conn,
        &workspace_id,
        &input,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn delete_tool_rate_limit(state: State<AppState>, id: String) -> AppResult<Vec<ToolRateLimit>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    tool_limits_service::delete_rate_limit(
        &conn,
        &workspace_id,
        &id,
        current_actor(&state).as_deref(),
    )
}

/// WP-0140: a tool result too large for the model's context, kept whole.
#[tauri::command]
pub fn get_stored_tool_result(state: State<AppState>, id: String) -> AppResult<StoredToolResult> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    tool_limits_service::get_stored_result(
        &conn,
        &workspace_id,
        &id,
        current_actor(&state).as_deref(),
    )
}
