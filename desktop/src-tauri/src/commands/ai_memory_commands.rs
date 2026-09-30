//! AI Agent Platform v2, Phase 4: Tauri commands for `ai_memory_service`
//! (the admin Memory Inspector - list/forget, plain sync) and
//! `ai_knowledge_service` (Knowledge Collections/Sources CRUD + search -
//! source create/update/search are genuinely async, since they call the
//! workspace's configured embeddings provider).

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::ai_knowledge::{KnowledgeCollection, KnowledgeCollectionInput, KnowledgeSearchHit, KnowledgeSource, KnowledgeSourceInput};
use lanesra_core::models::ai_memory::MemoryItem;
use lanesra_core::services::{ai_knowledge_service, ai_memory_service};

#[tauri::command]
pub fn list_memory_items(state: State<AppState>, memory_type: Option<String>, entity_type: Option<String>, entity_id: Option<String>) -> AppResult<Vec<MemoryItem>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_memory_service::list_all(&conn, &workspace_id, memory_type.as_deref(), entity_type.as_deref(), entity_id.as_deref(), current_actor(&state).as_deref())
}

#[tauri::command]
pub fn forget_memory_item(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_memory_service::forget(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_knowledge_collections(state: State<AppState>) -> AppResult<Vec<KnowledgeCollection>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_knowledge_service::list_collections(&conn, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn create_knowledge_collection(state: State<AppState>, input: KnowledgeCollectionInput) -> AppResult<KnowledgeCollection> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_knowledge_service::create_collection(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_knowledge_collection(state: State<AppState>, id: String, input: KnowledgeCollectionInput) -> AppResult<KnowledgeCollection> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_knowledge_service::update_collection(&conn, &workspace_id, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_knowledge_collection(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_knowledge_service::delete_collection(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_knowledge_sources(state: State<AppState>, collection_id: Option<String>) -> AppResult<Vec<KnowledgeSource>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_knowledge_service::list_sources(&conn, &workspace_id, collection_id.as_deref(), current_actor(&state).as_deref())
}

#[tauri::command]
pub fn get_knowledge_source(state: State<AppState>, id: String) -> AppResult<KnowledgeSource> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_knowledge_service::get_source(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_knowledge_source(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ai_knowledge_service::delete_source(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub async fn create_knowledge_source(state: State<'_, AppState>, input: KnowledgeSourceInput) -> AppResult<KnowledgeSource> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move { ai_knowledge_service::create_source(&conn, &workspace_id, &master_key, &input, actor.as_deref()).await }).await
}

#[tauri::command]
pub async fn update_knowledge_source(state: State<'_, AppState>, id: String, input: KnowledgeSourceInput) -> AppResult<KnowledgeSource> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move { ai_knowledge_service::update_source(&conn, &workspace_id, &master_key, &id, &input, actor.as_deref()).await }).await
}

#[tauri::command]
pub async fn search_knowledge_preview(state: State<'_, AppState>, query: String, collection_id: Option<String>) -> AppResult<Vec<KnowledgeSearchHit>> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let workspace_id = {
        let conn = state.conn.lock().unwrap();
        require_workspace_id(&conn)?
    };
    run_with_own_connection(db_path, move |conn| async move { ai_knowledge_service::search_knowledge(&conn, &workspace_id, &master_key, &query, collection_id.as_deref(), 10).await }).await
}
