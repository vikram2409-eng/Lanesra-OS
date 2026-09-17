//! Enterprise Access Foundation, Phase 1: Work Teams (spec §1.3) - the
//! first-class, record-owning security principal, distinct from a later
//! (Phase 2+) Access Group, which only ever groups principals for policies
//! and never owns anything itself.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::work_team::{TeamMembership, WorkTeam, WorkTeamInput, WorkTeamUpdate, WORK_TEAM_TYPES};
use crate::repositories::{org_unit_repo, team_membership_repo, user_repo, work_team_repo};
use crate::services::access_service;

/// Administrator always passes (unchanged); a non-Administrator additionally
/// passes with an explicit Access Role grant on "WorkTeam" - see
/// `access_service::require_admin_or_explicit_update`'s own doc comment.
fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    access_service::require_admin_or_explicit_update(conn, actor_user_id, "WorkTeam", "Only an Administrator can manage Work Teams")
}

fn validate_common(conn: &Connection, workspace_id: &str, input_name: &str, team_type: &str, primary_org_unit_id: &str) -> AppResult<()> {
    if input_name.trim().is_empty() {
        return Err(AppError::Validation("Name is required".into()));
    }
    if !WORK_TEAM_TYPES.contains(&team_type) {
        return Err(AppError::Validation(format!("Team type must be one of: {}", WORK_TEAM_TYPES.join(", "))));
    }
    let org_unit = org_unit_repo::get(conn, primary_org_unit_id)?.ok_or_else(|| AppError::Validation("Primary Organization Unit not found".into()))?;
    if org_unit.workspace_id != workspace_id {
        return Err(AppError::Validation("Primary Organization Unit belongs to a different workspace".into()));
    }
    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &WorkTeamInput, actor_user_id: Option<&str>) -> AppResult<WorkTeam> {
    require_admin(conn, actor_user_id)?;
    validate_common(conn, workspace_id, &input.name, &input.team_type, &input.primary_org_unit_id)?;
    if input.code.trim().is_empty() {
        return Err(AppError::Validation("Code is required".into()));
    }
    if work_team_repo::get_by_code(conn, workspace_id, &input.code)?.is_some() {
        return Err(AppError::Validation(format!("A team with code '{}' already exists", input.code)));
    }
    let id = crate::domain::ids::new_uuid();
    Ok(work_team_repo::create(conn, &id, workspace_id, input, actor_user_id)?)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<WorkTeam>> {
    Ok(work_team_repo::get(conn, id)?)
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<WorkTeam>> {
    Ok(work_team_repo::list(conn, workspace_id)?)
}

pub fn update(conn: &Connection, workspace_id: &str, id: &str, input: &WorkTeamUpdate, actor_user_id: Option<&str>) -> AppResult<WorkTeam> {
    require_admin(conn, actor_user_id)?;
    work_team_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Work Team".into()))?;
    validate_common(conn, workspace_id, &input.name, &input.team_type, &input.primary_org_unit_id)?;
    if !["Active", "Inactive"].contains(&input.status.as_str()) {
        return Err(AppError::Validation("Status must be Active or Inactive".into()));
    }
    Ok(work_team_repo::update(conn, id, input, actor_user_id)?)
}

/// Blocked while any active membership still exists, or any record still
/// carries this team as record_owner_id - deleting a team that owns
/// records would leave them orphaned; deactivate instead.
pub fn delete(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let team = work_team_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Work Team".into()))?;
    let members = team_membership_repo::list_active_members(conn, id)?;
    if !members.is_empty() {
        return Err(AppError::Validation(format!("Can't delete '{}' - {} active member(s) still belong to it. Remove them first.", team.name, members.len())));
    }
    Ok(work_team_repo::delete(conn, id)?)
}

pub fn list_members(conn: &Connection, team_id: &str) -> AppResult<Vec<TeamMembership>> {
    Ok(team_membership_repo::list_active_members(conn, team_id)?)
}

pub fn add_member(conn: &Connection, workspace_id: &str, team_id: &str, user_id: &str, role_in_team: Option<&str>, actor_user_id: Option<&str>) -> AppResult<TeamMembership> {
    require_admin(conn, actor_user_id)?;
    work_team_repo::get(conn, team_id)?.ok_or_else(|| AppError::NotFound("Work Team".into()))?;
    user_repo::find_by_id(conn, user_id)?.ok_or_else(|| AppError::NotFound("User".into()))?;
    let id = crate::domain::ids::new_uuid();
    let now = crate::domain::ids::now_iso();
    Ok(team_membership_repo::add(conn, &id, workspace_id, team_id, user_id, role_in_team, &now, actor_user_id)?)
}

/// Ends the membership (stamps effective_to) - never deletes the row and
/// never touches any record this team owns. See 0052_work_teams.sql's own
/// doc comment: membership and ownership are deliberately independent.
pub fn end_membership(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    team_membership_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Team membership".into()))?;
    Ok(team_membership_repo::end_membership(conn, id)?)
}
