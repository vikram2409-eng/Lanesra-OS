use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::industry_package::{
    AppInstallRun, AppPackage, ImportPackageInput, InstalledApp, InstalledAppDetail, WorkspaceArtifact, WorkspaceDependency,
};
use lanesra_core::services::industry_package_service;

#[tauri::command]
pub fn import_industry_package(state: State<AppState>, input: ImportPackageInput) -> AppResult<AppPackage> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    industry_package_service::import_package(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_industry_packages(state: State<AppState>) -> AppResult<Vec<AppPackage>> {
    let conn = state.conn.lock().unwrap();
    industry_package_service::list_packages(&conn, &require_workspace_id(&conn)?)
}

/// Re-checks an already-imported package's min-version/dependency
/// requirements - the Admin -> App Catalog screen's "Validate" step,
/// called before it offers "Install".
#[tauri::command]
pub fn validate_industry_package(state: State<AppState>, app_package_id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    industry_package_service::validate_package(&conn, &require_workspace_id(&conn)?, &app_package_id)
}

#[tauri::command]
pub fn install_industry_package(state: State<AppState>, app_package_id: String) -> AppResult<InstalledApp> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    industry_package_service::install(&conn, &workspace_id, &app_package_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_installed_apps(state: State<AppState>) -> AppResult<Vec<InstalledApp>> {
    let conn = state.conn.lock().unwrap();
    industry_package_service::list_installed(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn get_installed_app_detail(state: State<AppState>, id: String) -> AppResult<InstalledAppDetail> {
    let conn = state.conn.lock().unwrap();
    industry_package_service::get_installed_detail(&conn, &id)
}

#[tauri::command]
pub fn list_industry_install_runs(state: State<AppState>) -> AppResult<Vec<AppInstallRun>> {
    let conn = state.conn.lock().unwrap();
    industry_package_service::list_runs(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn deactivate_installed_app(state: State<AppState>, id: String) -> AppResult<InstalledApp> {
    let conn = state.conn.lock().unwrap();
    industry_package_service::deactivate(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn reactivate_installed_app(state: State<AppState>, id: String) -> AppResult<InstalledApp> {
    let conn = state.conn.lock().unwrap();
    industry_package_service::reactivate(&conn, &id, current_actor(&state).as_deref())
}

/// Fetches a bundled starter manifest's raw JSON to prefill the Review
/// step's textarea - no database access needed, so no `state`.
#[tauri::command]
pub fn get_reference_package_manifest(key: String) -> AppResult<String> {
    industry_package_service::reference_package_manifest(&key)
}

/// Every dependency declared by every package imported into this
/// workspace, satisfied or not - Admin -> Solution Management's
/// Dependencies tab.
#[tauri::command]
pub fn list_package_dependencies(state: State<AppState>) -> AppResult<Vec<WorkspaceDependency>> {
    let conn = state.conn.lock().unwrap();
    industry_package_service::list_dependencies_for_workspace(&conn, &require_workspace_id(&conn)?)
}

/// Every artifact created by every app installed in this workspace -
/// Admin -> Solution Management's Components tab.
#[tauri::command]
pub fn list_package_artifacts_for_workspace(state: State<AppState>) -> AppResult<Vec<WorkspaceArtifact>> {
    let conn = state.conn.lock().unwrap();
    industry_package_service::list_artifacts_for_workspace(&conn, &require_workspace_id(&conn)?)
}
