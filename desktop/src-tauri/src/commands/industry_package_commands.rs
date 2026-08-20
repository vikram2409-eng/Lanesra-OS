use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::industry_package::{
    AppInstallRun, AppPackage, ImportPackageInput, InstalledApp, InstalledAppDetail, PackageUpdateDiff, WorkspaceArtifact, WorkspaceDependency,
};
use lanesra_core::models::solution_component::{LocalWorkspaceSummary, WorkspaceComponent};
use lanesra_core::services::{industry_package_service, solution_component_service};

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

/// Every component in the workspace (hand-built or package-installed),
/// each tagged with its owning publisher - the Phase 3 superset of
/// `list_package_artifacts_for_workspace` above.
#[tauri::command]
pub fn list_solution_components(state: State<AppState>) -> AppResult<Vec<WorkspaceComponent>> {
    let conn = state.conn.lock().unwrap();
    solution_component_service::list_for_workspace(&conn, &require_workspace_id(&conn)?)
}

/// The Managed/Unmanaged distinction's Unmanaged half - a count of
/// everything still owned by the `local` publisher, for the synthetic
/// "Local Workspace" row in Solution Packages.
#[tauri::command]
pub fn get_local_workspace_summary(state: State<AppState>) -> AppResult<LocalWorkspaceSummary> {
    let conn = state.conn.lock().unwrap();
    solution_component_service::local_workspace_summary(&conn, &require_workspace_id(&conn)?)
}

/// Every imported version of one package - Solution Management's
/// Releases view.
#[tauri::command]
pub fn list_package_versions(state: State<AppState>, package_id: String) -> AppResult<Vec<AppPackage>> {
    let conn = state.conn.lock().unwrap();
    industry_package_service::list_package_versions(&conn, &require_workspace_id(&conn)?, &package_id)
}

/// Builds a re-importable `.lanesra`-style manifest from everything the
/// `local` publisher currently owns.
#[tauri::command]
pub fn export_local_workspace(state: State<AppState>) -> AppResult<String> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    industry_package_service::export_local_workspace(&conn, &workspace_id, current_actor(&state).as_deref())
}

/// The update-with-diff review step: what would change if
/// `new_app_package_id` (an already-imported newer version) were applied
/// over the currently installed version.
#[tauri::command]
pub fn plan_package_update(state: State<AppState>, new_app_package_id: String) -> AppResult<PackageUpdateDiff> {
    let conn = state.conn.lock().unwrap();
    industry_package_service::plan_update(&conn, &require_workspace_id(&conn)?, &new_app_package_id)
}

/// Applies a newer package version over the currently installed one.
#[tauri::command]
pub fn apply_package_update(state: State<AppState>, new_app_package_id: String) -> AppResult<InstalledApp> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    industry_package_service::apply_update(&conn, &workspace_id, &new_app_package_id, current_actor(&state).as_deref())
}
