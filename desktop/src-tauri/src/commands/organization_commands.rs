use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::org_unit::{OrgUnit, OrgUnitInput, OrgUnitMoveImpact, OrgUnitUpdate};
use lanesra_core::models::organization::{Organization, OrganizationUpdate};
use lanesra_core::models::ownership::{OwnerRef, OwnershipTransferDryRun, RecordOwnership};
use lanesra_core::models::work_team::{TeamMembership, WorkTeam, WorkTeamInput, WorkTeamUpdate};
use lanesra_core::services::bulk_action_service::BulkActionResult;
use lanesra_core::services::{org_unit_service, organization_service, ownership_service, work_team_service};

#[tauri::command]
pub fn get_organization(state: State<AppState>) -> AppResult<Organization> {
    let conn = state.conn.lock().unwrap();
    organization_service::get(&conn, &require_workspace_id(&conn)?, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_organization(state: State<AppState>, input: OrganizationUpdate) -> AppResult<Organization> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    organization_service::update(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_org_units(state: State<AppState>) -> AppResult<Vec<OrgUnit>> {
    let conn = state.conn.lock().unwrap();
    org_unit_service::list_tree(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_org_unit(state: State<AppState>, input: OrgUnitInput) -> AppResult<OrgUnit> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    org_unit_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_org_unit(state: State<AppState>, id: String, input: OrgUnitUpdate) -> AppResult<OrgUnit> {
    let conn = state.conn.lock().unwrap();
    org_unit_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn preview_move_org_unit(state: State<AppState>, id: String, new_parent_id: String) -> AppResult<OrgUnitMoveImpact> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    org_unit_service::preview_move(&conn, &workspace_id, &id, &new_parent_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn move_org_unit(state: State<AppState>, id: String, new_parent_id: String) -> AppResult<OrgUnit> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    org_unit_service::move_unit(&conn, &workspace_id, &id, &new_parent_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_org_unit(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    org_unit_service::delete(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_work_teams(state: State<AppState>) -> AppResult<Vec<WorkTeam>> {
    let conn = state.conn.lock().unwrap();
    work_team_service::list(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_work_team(state: State<AppState>, input: WorkTeamInput) -> AppResult<WorkTeam> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    work_team_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_work_team(state: State<AppState>, id: String, input: WorkTeamUpdate) -> AppResult<WorkTeam> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    work_team_service::update(&conn, &workspace_id, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_work_team(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    work_team_service::delete(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_team_members(state: State<AppState>, team_id: String) -> AppResult<Vec<TeamMembership>> {
    let conn = state.conn.lock().unwrap();
    work_team_service::list_members(&conn, &team_id)
}

#[tauri::command]
pub fn add_team_member(state: State<AppState>, team_id: String, user_id: String, role_in_team: Option<String>) -> AppResult<TeamMembership> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    work_team_service::add_member(&conn, &workspace_id, &team_id, &user_id, role_in_team.as_deref(), current_actor(&state).as_deref())
}

#[tauri::command]
pub fn end_team_membership(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    work_team_service::end_membership(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn get_record_owner(state: State<AppState>, object_key: String, id: String) -> AppResult<Option<RecordOwnership>> {
    let conn = state.conn.lock().unwrap();
    ownership_service::get_owner(&conn, &object_key, &id)
}

#[tauri::command]
pub fn set_record_owner(state: State<AppState>, object_key: String, id: String, owner: OwnerRef, owning_org_unit_id: Option<String>) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ownership_service::set_owner(&conn, &workspace_id, &object_key, &id, &owner, owning_org_unit_id.as_deref(), current_actor(&state).as_deref())
}

#[tauri::command]
pub fn bulk_transfer_ownership_dry_run(state: State<AppState>, object_key: String, ids: Vec<String>, new_owner: OwnerRef) -> AppResult<OwnershipTransferDryRun> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ownership_service::bulk_transfer_dry_run(&conn, &workspace_id, &object_key, &ids, &new_owner)
}

#[tauri::command]
pub fn bulk_transfer_ownership_commit(
    state: State<AppState>,
    object_key: String,
    ids: Vec<String>,
    new_owner: OwnerRef,
    owning_org_unit_id: Option<String>,
) -> AppResult<Vec<BulkActionResult>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ownership_service::bulk_transfer_commit(&conn, &workspace_id, &object_key, &ids, &new_owner, owning_org_unit_id.as_deref(), current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_user_org_units(state: State<AppState>, user_id: String) -> AppResult<Vec<String>> {
    let conn = state.conn.lock().unwrap();
    Ok(lanesra_core::repositories::user_repo::list_additional_org_units(&conn, &user_id)?)
}

#[tauri::command]
pub fn set_user_primary_org_unit(state: State<AppState>, user_id: String, org_unit_id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    Ok(lanesra_core::repositories::user_repo::set_primary_org_unit(&conn, &user_id, &org_unit_id)?)
}

#[tauri::command]
pub fn add_user_org_unit(state: State<AppState>, user_id: String, org_unit_id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    Ok(lanesra_core::repositories::user_repo::add_additional_org_unit(&conn, &workspace_id, &user_id, &org_unit_id)?)
}

#[tauri::command]
pub fn remove_user_org_unit(state: State<AppState>, user_id: String, org_unit_id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    Ok(lanesra_core::repositories::user_repo::remove_additional_org_unit(&conn, &user_id, &org_unit_id)?)
}
