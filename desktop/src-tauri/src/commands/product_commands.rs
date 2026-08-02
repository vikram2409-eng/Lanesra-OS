use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::domain::AppResult;
use crate::models::product::{Product, ProductInput};
use crate::services::product_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_products(state: State<AppState>) -> AppResult<Vec<Product>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    product_service::list(&conn, &workspace_id)
}

#[tauri::command]
pub fn get_product(state: State<AppState>, id: String) -> AppResult<Product> {
    let conn = state.conn.lock().unwrap();
    product_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_product(state: State<AppState>, input: ProductInput) -> AppResult<Product> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    product_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_product(state: State<AppState>, id: String, input: ProductInput) -> AppResult<Product> {
    let conn = state.conn.lock().unwrap();
    product_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn archive_product(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    product_service::archive(&conn, &id, current_actor(&state).as_deref())
}
