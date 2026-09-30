//! AI Agent Platform v2 (GitHub issue #170, backend half): Tauri
//! commands for `mcp_client_service` - plain CRUD is sync; `discover`
//! (a real handshake against the external server) is genuinely async,
//! the same `run_with_own_connection` shape `ai_knowledge_commands`'s own
//! embeddings-provider calls already use.

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::ai_mcp::{AgentMcpToolOption, McpServer, McpServerInput, McpServerUpdate, McpTool};
use lanesra_core::services::mcp_client_service;

#[tauri::command]
pub fn create_mcp_server(state: State<AppState>, input: McpServerInput) -> AppResult<McpServer> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    mcp_client_service::create_server(&conn, &workspace_id, &master_key, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_mcp_servers(state: State<AppState>) -> AppResult<Vec<McpServer>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    mcp_client_service::list_servers(&conn, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn get_mcp_server(state: State<AppState>, id: String) -> AppResult<McpServer> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    mcp_client_service::get_server(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_mcp_server(state: State<AppState>, id: String, input: McpServerUpdate) -> AppResult<McpServer> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    mcp_client_service::update_server(&conn, &workspace_id, &master_key, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_mcp_server(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    mcp_client_service::delete_server(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_mcp_tools(state: State<AppState>, id: String) -> AppResult<Vec<McpTool>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    mcp_client_service::list_tools(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_mcp_tool_flags(state: State<AppState>, id: String, tool_name: String, is_write: bool, enabled: bool) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    mcp_client_service::set_tool_flags(&conn, &workspace_id, &id, &tool_name, is_write, enabled, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_agent_mcp_tools(state: State<AppState>) -> AppResult<Vec<AgentMcpToolOption>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    mcp_client_service::list_options(&conn, &workspace_id)
}

#[tauri::command]
pub async fn discover_mcp_tools(state: State<'_, AppState>, id: String) -> AppResult<Vec<McpTool>> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move { mcp_client_service::discover_tools(&conn, &workspace_id, &master_key, &id, actor.as_deref()).await }).await
}
