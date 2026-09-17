use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::access_role::Capability;
use lanesra_core::models::contact::{Contact, ContactInput};
use lanesra_core::services::{access_service, contact_service};
use crate::state::AppState;

#[tauri::command]
pub fn list_contacts(state: State<AppState>) -> AppResult<Vec<Contact>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let mut contacts = contact_service::list(&conn, &workspace_id)?;
    let visible = access_service::filter_visible(&conn, current_actor(&state).as_deref(), "Contact", contacts.iter().map(|c| c.id.as_str()))?;
    contacts.retain(|c| visible.contains(&c.id));
    Ok(contacts)
}

#[tauri::command]
pub fn list_contacts_by_company(state: State<AppState>, company_id: String) -> AppResult<Vec<Contact>> {
    let conn = state.conn.lock().unwrap();
    let mut contacts = contact_service::list_by_company(&conn, &company_id)?;
    let visible = access_service::filter_visible(&conn, current_actor(&state).as_deref(), "Contact", contacts.iter().map(|c| c.id.as_str()))?;
    contacts.retain(|c| visible.contains(&c.id));
    Ok(contacts)
}

#[tauri::command]
pub fn get_contact(state: State<AppState>, id: String) -> AppResult<Contact> {
    let conn = state.conn.lock().unwrap();
    access_service::require_capability(&conn, current_actor(&state).as_deref(), "Contact", Capability::Read, Some(&id))?;
    contact_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_contact(state: State<AppState>, input: ContactInput) -> AppResult<Contact> {
    let conn = state.conn.lock().unwrap();
    contact_service::create(&conn, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_contact(state: State<AppState>, id: String, input: ContactInput) -> AppResult<Contact> {
    let conn = state.conn.lock().unwrap();
    contact_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn archive_contact(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    contact_service::archive(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn check_contact_duplicates(
    state: State<AppState>,
    company_id: String,
    email: String,
    exclude_id: Option<String>,
) -> AppResult<Vec<Contact>> {
    let conn = state.conn.lock().unwrap();
    contact_service::check_duplicates(&conn, &company_id, &email, exclude_id.as_deref())
}
