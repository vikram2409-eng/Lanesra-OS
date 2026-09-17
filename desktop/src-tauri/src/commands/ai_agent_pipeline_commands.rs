//! AI & Agentic Layer, Phase 6b: Tauri commands for
//! `ai_orchestration_service` - Pipeline/Trigger CRUD and run history are
//! plain sync; `run_ai_agent`/`run_ai_agent_pipeline` (manual "Run Now")
//! and `drain_pending_async_work` (desktop's own poll for the
//! schedule/webhook/workflow-triggered queue, alongside the existing
//! `runScheduledWorkflows` interval in `App.tsx` - see that command's own
//! doc comment) are genuinely async.

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipeline, AiAgentPipelineInput, AiAgentRun, AiAgentTrigger, AiAgentTriggerInput, PipelineHierarchy};
use lanesra_core::services::{ai_agent_hierarchy_service, ai_orchestration_service};

#[tauri::command]
pub fn list_ai_agent_pipelines(state: State<AppState>, active_only: bool) -> AppResult<Vec<AiAgentPipeline>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_orchestration_service::list_pipelines(&conn, &workspace_id, active_only)
}

#[tauri::command]
pub fn create_ai_agent_pipeline(state: State<AppState>, input: AiAgentPipelineInput) -> AppResult<AiAgentPipeline> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_orchestration_service::create_pipeline(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_ai_agent_pipeline(state: State<AppState>, id: String, input: AiAgentPipelineInput) -> AppResult<AiAgentPipeline> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_orchestration_service::update_pipeline(&conn, &id, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_ai_agent_pipeline_active(state: State<AppState>, id: String, is_active: bool) -> AppResult<AiAgentPipeline> {
    let conn = state.conn.lock().unwrap();
    ai_orchestration_service::set_pipeline_active(&conn, &id, is_active, current_actor(&state).as_deref())
}

/// The Agent Hierarchy - who a Pipeline's steps run, and who each step's
/// agent may in turn delegate to. See `ai_agent_hierarchy_service`'s own
/// doc comment.
#[tauri::command]
pub fn get_ai_agent_pipeline_hierarchy(state: State<AppState>, id: String) -> AppResult<PipelineHierarchy> {
    let conn = state.conn.lock().unwrap();
    ai_agent_hierarchy_service::resolve(&conn, &id)
}

#[tauri::command]
pub fn create_ai_agent_trigger(state: State<AppState>, input: AiAgentTriggerInput) -> AppResult<AiAgentTrigger> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_orchestration_service::create_trigger(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_ai_agent_triggers(state: State<AppState>, target_type: String, target_id: String) -> AppResult<Vec<AiAgentTrigger>> {
    let conn = state.conn.lock().unwrap();
    ai_orchestration_service::list_triggers(&conn, &target_type, &target_id)
}

#[tauri::command]
pub fn set_ai_agent_trigger_active(state: State<AppState>, id: String, is_active: bool) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    ai_orchestration_service::set_trigger_active(&conn, &id, is_active, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_ai_agent_trigger(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    ai_orchestration_service::delete_trigger(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_ai_agent_runs(state: State<AppState>, target_type: String, target_id: String, limit: i64) -> AppResult<Vec<AiAgentRun>> {
    let conn = state.conn.lock().unwrap();
    ai_orchestration_service::list_runs(&conn, &target_type, &target_id, limit)
}

/// AI & Agentic Layer, Phase 7e: rejecting a paused run and exporting its
/// trace are both plain sync; `approve_ai_agent_pending_step` (resumes
/// real agent execution) is genuinely async, below.
#[tauri::command]
pub fn reject_ai_agent_pending_run(state: State<AppState>, id: String, reason: String) -> AppResult<AiAgentRun> {
    let conn = state.conn.lock().unwrap();
    ai_orchestration_service::reject_pending_run(&conn, &id, &reason, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn export_ai_agent_run_otlp(state: State<AppState>, id: String) -> AppResult<serde_json::Value> {
    let conn = state.conn.lock().unwrap();
    ai_orchestration_service::export_run_as_otlp(&conn, &id)
}

#[tauri::command]
pub async fn approve_ai_agent_pending_step(state: State<'_, AppState>, id: String, edited_output: Option<String>) -> AppResult<AiAgentRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        ai_orchestration_service::approve_pending_step(&conn, &workspace_id, &master_key, &id, edited_output.as_deref(), actor.as_deref()).await
    })
    .await
}

#[tauri::command]
pub async fn push_ai_agent_run_otlp(state: State<'_, AppState>, id: String) -> AppResult<String> {
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move { ai_orchestration_service::push_run_trace_to_otlp(&conn, &workspace_id, &id, actor.as_deref()).await }).await
}

/// AI & Agentic Layer, Phase 7g: drains the workspace's full pending
/// embedding queue right away - genuinely async, same reasoning
/// `push_ai_agent_run_otlp` above isn't a plain sync command. Returns how
/// many records were actually reindexed.
#[tauri::command]
pub async fn reindex_vector_search(state: State<'_, AppState>) -> AppResult<usize> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        lanesra_core::services::vector_search_service::reindex_workspace(&conn, &workspace_id, &master_key, actor.as_deref()).await
    })
    .await
}

#[tauri::command]
pub async fn run_ai_agent(state: State<'_, AppState>, id: String, input: String) -> AppResult<AiAgentRun> {
    run_target(state, "agent", id, input).await
}

#[tauri::command]
pub async fn run_ai_agent_pipeline(state: State<'_, AppState>, id: String, input: String) -> AppResult<AiAgentRun> {
    run_target(state, "pipeline", id, input).await
}

async fn run_target(state: State<'_, AppState>, target_type: &'static str, id: String, input: String) -> AppResult<AiAgentRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        ai_orchestration_service::run_manual(&conn, &workspace_id, &master_key, target_type, &id, &input, actor.as_deref()).await
    })
    .await
}

/// Desktop's own poll for everything the server's `job_scheduler.rs`
/// tick drains - a schedule Trigger, a `run_ai_agent` workflow action,
/// (the incidental fix) `call_connector_action`'s own long-unwired
/// queue, and (Phase 7g) a Custom Object record's own pending vector
/// search reindex. Called from `App.tsx`'s existing 5-minute `runScheduledWorkflows`
/// interval, not a separate one - `runScheduledWorkflows` itself stays
/// sync (scheduled workflow triggers make no network calls); this is the
/// one genuinely-async drain desktop needs alongside it.
#[tauri::command]
pub async fn drain_pending_async_work(state: State<'_, AppState>) -> AppResult<()> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let workspace_id = {
        let conn = state.conn.lock().unwrap();
        require_workspace_id(&conn)?
    };
    run_with_own_connection(db_path, move |conn| async move {
        let _ = lanesra_core::services::connector_execution_service::drain_pending_actions(&conn, &workspace_id, &master_key, 50).await;
        let _ = ai_orchestration_service::enqueue_due_schedules(&conn, &workspace_id);
        let _ = ai_orchestration_service::drain_pending_runs(&conn, &workspace_id, &master_key, 50).await;
        // AI & Agentic Layer, Phase 7g: same drain the server's
        // `job_scheduler.rs` tick runs, here on desktop's own poll.
        let _ = lanesra_core::services::vector_search_service::drain_pending_embeddings(&conn, &workspace_id, &master_key, 50).await;
        Ok(())
    })
    .await
}
