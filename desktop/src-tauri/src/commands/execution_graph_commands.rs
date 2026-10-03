//! AI Agent Platform v2, Phase 3: Tauri commands for
//! `execution_graph_service` (graph CRUD/publish, plain sync) and
//! `graph_runtime_service` (start/resume/approve/cancel a run - genuinely
//! async, since a run may call an Agent node through `chat_service`).
//! No visual canvas UI consumes these yet (tracked separately, issue
//! #168's own "Explicitly not in this PR" note) - a graph is authored via
//! these commands directly (or via `execution_graph_service::graph_from_
//! workflow`/`graph_from_pipeline`) until that later phase lands.

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::ai_agent::AiAgentDefinition;
use lanesra_core::models::execution_graph::{ExecutionGraph, ExecutionGraphInput};
use lanesra_core::models::graph_run::GraphRun;
use lanesra_core::services::{execution_graph_service, graph_runtime_service};

#[tauri::command]
pub fn list_execution_graphs(state: State<AppState>) -> AppResult<Vec<ExecutionGraph>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    execution_graph_service::list(&conn, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn get_execution_graph(state: State<AppState>, id: String) -> AppResult<ExecutionGraph> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    execution_graph_service::get(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn create_execution_graph(state: State<AppState>, input: ExecutionGraphInput) -> AppResult<ExecutionGraph> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    execution_graph_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_execution_graph(state: State<AppState>, id: String, input: ExecutionGraphInput) -> AppResult<ExecutionGraph> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    execution_graph_service::update(&conn, &id, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn publish_execution_graph(state: State<AppState>, id: String) -> AppResult<ExecutionGraph> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    execution_graph_service::publish(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn promote_embedded_agent_node(state: State<AppState>, graph_id: String, node_key: String) -> AppResult<AiAgentDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    execution_graph_service::promote_embedded_agent_node(&conn, &graph_id, &workspace_id, &node_key, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_execution_graph_disabled(state: State<AppState>, id: String, disabled: bool) -> AppResult<ExecutionGraph> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    execution_graph_service::set_disabled(&conn, &id, &workspace_id, disabled, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn get_graph_run(state: State<AppState>, id: String) -> AppResult<GraphRun> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    graph_runtime_service::get_run(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_graph_runs(state: State<AppState>, graph_id: String) -> AppResult<Vec<GraphRun>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    graph_runtime_service::list_runs_for_graph(&conn, &graph_id, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub async fn start_graph_run(state: State<'_, AppState>, graph_id: String, trigger_input: String) -> AppResult<GraphRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        graph_runtime_service::start_run(&conn, &workspace_id, &master_key, &graph_id, &trigger_input, actor.as_deref(), None, None, None).await
    })
    .await
}

#[tauri::command]
pub async fn resume_graph_run(state: State<'_, AppState>, id: String) -> AppResult<GraphRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move { graph_runtime_service::resume_run(&conn, &workspace_id, &master_key, &id, actor.as_deref()).await }).await
}

#[tauri::command]
pub async fn resolve_graph_run_approval(state: State<'_, AppState>, id: String, approve: bool, notes: Option<String>) -> AppResult<GraphRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        graph_runtime_service::resolve_approval(&conn, &workspace_id, &master_key, &id, approve, notes.as_deref(), actor.as_deref()).await
    })
    .await
}

#[tauri::command]
pub async fn cancel_graph_run(state: State<'_, AppState>, id: String) -> AppResult<GraphRun> {
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move { graph_runtime_service::cancel_run(&conn, &workspace_id, &id, actor.as_deref()).await }).await
}
