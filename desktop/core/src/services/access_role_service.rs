//! Access Control v1: CRUD for Access Roles and their per-object grants, and
//! assigning/removing a role to/from a user. Managing Access Roles
//! themselves stays gated by the existing legacy Administrator check - not
//! a new self-referential "manage access roles" capability, which would let
//! a misconfigured role revoke its own administrators' ability to fix it.
//! Deliberate seam, same style as `ownership_service::require_assign_capability`'s
//! own placeholder note in Phase 1.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::access_role::{AccessRole, AccessRoleGrant, AccessRoleGrantInput, AccessRoleInput, AccessRoleUpdate, CAPABILITIES, RECORD_SCOPES};
use crate::repositories::{access_role_repo, user_access_role_repo, user_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage Access Roles".into()));
    }
    Ok(())
}

fn validate_grant_shape(input: &AccessRoleGrantInput) -> AppResult<()> {
    if input.object_key.trim().is_empty() {
        return Err(AppError::Validation("Object key is required (use '*' for the default grant)".into()));
    }
    if !RECORD_SCOPES.contains(&input.record_scope.as_str()) {
        return Err(AppError::Validation(format!("Record scope must be one of: {}", RECORD_SCOPES.join(", "))));
    }
    Ok(())
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<AccessRole>> {
    Ok(access_role_repo::list(conn, workspace_id)?)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<AccessRole>> {
    Ok(access_role_repo::get(conn, id)?)
}

pub fn create(conn: &Connection, workspace_id: &str, input: &AccessRoleInput, actor_user_id: Option<&str>) -> AppResult<AccessRole> {
    require_admin(conn, actor_user_id)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name is required".into()));
    }
    let id = new_uuid();
    Ok(access_role_repo::create(conn, &id, workspace_id, input, false, actor_user_id)?)
}

pub fn update(conn: &Connection, id: &str, input: &AccessRoleUpdate, actor_user_id: Option<&str>) -> AppResult<AccessRole> {
    require_admin(conn, actor_user_id)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name is required".into()));
    }
    Ok(access_role_repo::update(conn, id, input, actor_user_id)?)
}

pub fn delete(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let role = access_role_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Access Role".into()))?;
    if role.is_system {
        return Err(AppError::Validation(format!("'{}' is a system role and can't be deleted", role.name)));
    }
    let assignees = user_access_role_repo::count_assignees(conn, id)?;
    if assignees > 0 {
        return Err(AppError::Validation(format!("'{}' is still assigned to {} user(s) - remove them first", role.name, assignees)));
    }
    Ok(access_role_repo::delete(conn, id)?)
}

pub fn list_grants(conn: &Connection, access_role_id: &str) -> AppResult<Vec<AccessRoleGrant>> {
    Ok(access_role_repo::list_grants(conn, access_role_id)?)
}

pub fn upsert_grant(conn: &Connection, access_role_id: &str, input: &AccessRoleGrantInput, actor_user_id: Option<&str>) -> AppResult<AccessRoleGrant> {
    require_admin(conn, actor_user_id)?;
    validate_grant_shape(input)?;
    access_role_repo::get(conn, access_role_id)?.ok_or_else(|| AppError::NotFound("Access Role".into()))?;
    let id = new_uuid();
    Ok(access_role_repo::upsert_grant(conn, &id, access_role_id, input)?)
}

pub fn delete_grant(conn: &Connection, access_role_id: &str, object_key: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    Ok(access_role_repo::delete_grant(conn, access_role_id, object_key)?)
}

pub fn list_assignee_user_ids(conn: &Connection, access_role_id: &str) -> AppResult<Vec<String>> {
    Ok(user_access_role_repo::list_assignee_user_ids(conn, access_role_id)?)
}

pub fn list_roles_for_user(conn: &Connection, user_id: &str) -> AppResult<Vec<AccessRole>> {
    Ok(user_access_role_repo::list_for_user(conn, user_id)?)
}

pub fn assign_to_user(conn: &Connection, user_id: &str, access_role_id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    user_repo::find_by_id(conn, user_id)?.ok_or_else(|| AppError::NotFound("User".into()))?;
    access_role_repo::get(conn, access_role_id)?.ok_or_else(|| AppError::NotFound("Access Role".into()))?;
    Ok(user_access_role_repo::assign(conn, user_id, access_role_id)?)
}

pub fn remove_from_user(conn: &Connection, user_id: &str, access_role_id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    Ok(user_access_role_repo::remove(conn, user_id, access_role_id)?)
}

/// Validates a capability string from the frontend - shared with the Tauri
/// command layer so both `check_capability` and `inspect_access` reject the
/// same bad input the same way.
pub fn parse_capability_str(s: &str) -> AppResult<crate::models::access_role::Capability> {
    crate::models::access_role::Capability::from_str(s)
        .ok_or_else(|| AppError::Validation(format!("Capability must be one of: {}", CAPABILITIES.join(", "))))
}

/// Gives a newly created user a starting Access Role - "Full Access" if
/// their legacy roles include Administrator, else "Standard User" - the
/// same bridge the 0056 migration's own backfill applies at upgrade time,
/// so a user created after this phase shipped never starts with zero
/// Access Roles. Mirrors `ownership_service::set_default_owner_on_create`'s
/// own stance: never blocks the create itself, silently no-ops if the
/// expected system role has been renamed or removed.
pub fn assign_default_role_for_new_user(conn: &Connection, workspace_id: &str, user_id: &str, legacy_roles: &[String]) -> AppResult<()> {
    ensure_system_roles_seeded(conn, workspace_id)?;
    let system_role_name = if legacy_roles.iter().any(|r| r == "Administrator") { "Full Access" } else { "Standard User" };
    if let Some(role) = access_role_repo::get_by_name(conn, workspace_id, system_role_name)? {
        user_access_role_repo::assign(conn, user_id, &role.id)?;
    }
    Ok(())
}

/// Idempotent - the same "bootstrap on first access" pattern
/// `organization_service::get_or_bootstrap` uses for a workspace's root
/// Organization Unit. The 0056 migration only seeds "Full Access"/"Standard
/// User" for workspaces that already existed when it ran; a workspace
/// created afterward (`workspace_service::first_run_setup` creates its
/// workspace row itself, post-migration) has neither until something calls
/// this - so every capability check does, before it looks anything up.
pub fn ensure_system_roles_seeded(conn: &Connection, workspace_id: &str) -> AppResult<()> {
    if access_role_repo::get_by_name(conn, workspace_id, "Full Access")?.is_some() {
        return Ok(());
    }
    let full_access = access_role_repo::create(
        conn,
        &new_uuid(),
        workspace_id,
        &AccessRoleInput { name: "Full Access".into(), description: "Every capability, organization-wide - the default for anyone who already holds the legacy Administrator role.".into() },
        true,
        None,
    )?;
    access_role_repo::upsert_grant(
        conn,
        &new_uuid(),
        &full_access.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: true, can_read: true, can_update: true, can_delete: true, can_assign: true, record_scope: "ORGANIZATION".into() },
    )?;
    let standard_user = access_role_repo::create(
        conn,
        &new_uuid(),
        workspace_id,
        &AccessRoleInput { name: "Standard User".into(), description: "Ordinary create/read/update/delete, same as every workspace already behaves today - the one thing this role does not grant by default is Assign (reassigning a record's owner), which stays scoped to whoever an admin explicitly grants it to.".into() },
        true,
        None,
    )?;
    // Deliberately Organization scope, not Owner - see 0056_access_roles.sql's
    // own comment on this exact grant for why: ordinary CRUD has never been
    // owner-restricted in this product, only Assign was ever actually
    // enforced. Seeding Owner scope here would silently lock every
    // non-Administrator user out of records they don't personally own.
    access_role_repo::upsert_grant(
        conn,
        &new_uuid(),
        &standard_user.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: true, can_read: true, can_update: true, can_delete: true, can_assign: false, record_scope: "ORGANIZATION".into() },
    )?;
    Ok(())
}

/// Idempotent per-user counterpart: a user who predates this phase (an
/// admin created via `workspace_service::first_run_setup`, which bypasses
/// `user_service::create`'s own default-role assignment) holds zero Access
/// Roles until something checks. Called by `access_service` before every
/// capability evaluation, same reasoning as `ensure_system_roles_seeded`.
pub fn ensure_user_has_access_role(conn: &Connection, workspace_id: &str, user_id: &str) -> AppResult<()> {
    ensure_system_roles_seeded(conn, workspace_id)?;
    if !user_access_role_repo::list_for_user(conn, user_id)?.is_empty() {
        return Ok(());
    }
    let legacy_roles = user_repo::roles_for_user(conn, user_id)?;
    assign_default_role_for_new_user(conn, workspace_id, user_id, &legacy_roles)
}
