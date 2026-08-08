use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::user::{NewUser, PasswordChange, User, UserUpdate};
use crate::repositories::{audit_repo, user_repo};
use crate::services::auth_service;

pub(crate) fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation(
            "Only an Administrator can manage users".into(),
        ));
    }
    Ok(())
}

fn validate_roles(roles: &[String]) -> AppResult<()> {
    if roles.is_empty() {
        return Err(AppError::Validation("A user needs at least one role".into()));
    }
    for role in roles {
        if !user_repo::ROLES.contains(&role.as_str()) {
            return Err(AppError::Validation(format!("Invalid role '{role}'")));
        }
    }
    Ok(())
}

fn to_public(conn: &Connection, id: &str) -> AppResult<User> {
    let record = user_repo::find_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("User".into()))?;
    let roles = user_repo::roles_for_user(conn, id)?;
    Ok(user_repo::to_public(record, roles))
}

/// Any authenticated user can list the directory (needed to assign task
/// owners); only an Administrator can create, edit or deactivate accounts.
pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<User>> {
    user_repo::list(conn, workspace_id)?
        .into_iter()
        .map(|record| {
            let roles = user_repo::roles_for_user(conn, &record.id)?;
            Ok(user_repo::to_public(record, roles))
        })
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

pub fn create(
    conn: &Connection,
    workspace_id: &str,
    input: &NewUser,
    actor_user_id: Option<&str>,
) -> AppResult<User> {
    require_admin(conn, actor_user_id)?;
    if input.username.trim().is_empty() || input.display_name.trim().is_empty() {
        return Err(AppError::Validation("Username and display name are required".into()));
    }
    if input.password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters".into()));
    }
    validate_roles(&input.roles)?;

    let password_hash = auth_service::hash_password(&input.password)?;
    let record = user_repo::create(
        conn,
        workspace_id,
        &input.username,
        &input.display_name,
        &password_hash,
        &input.roles,
    )?;

    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "user_admin",
        Some("user"),
        Some(&record.id),
        &format!("Created user '{}'", record.username),
        None,
    )?;

    to_public(conn, &record.id)
}

pub fn update(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    input: &UserUpdate,
    actor_user_id: Option<&str>,
) -> AppResult<User> {
    require_admin(conn, actor_user_id)?;
    if input.display_name.trim().is_empty() {
        return Err(AppError::Validation("Display name is required".into()));
    }
    validate_roles(&input.roles)?;

    let current = user_repo::find_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("User".into()))?;
    let current_roles = user_repo::roles_for_user(conn, id)?;
    let was_active_admin = current.is_active && current_roles.iter().any(|r| r == "Administrator");
    let will_be_active_admin = input.is_active && input.roles.iter().any(|r| r == "Administrator");

    if was_active_admin && !will_be_active_admin {
        let admin_count = user_repo::count_active_administrators(conn, workspace_id)?;
        if admin_count <= 1 {
            return Err(AppError::Validation(
                "Cannot remove or deactivate the last active administrator".into(),
            ));
        }
    }

    user_repo::update(conn, id, &input.display_name, input.is_active)?;
    user_repo::set_roles(conn, id, &input.roles)?;

    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "user_admin",
        Some("user"),
        Some(id),
        &format!("Updated user '{}'", current.username),
        None,
    )?;

    to_public(conn, id)
}

pub fn set_password(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    input: &PasswordChange,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    if input.new_password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters".into()));
    }
    let user = user_repo::find_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("User".into()))?;
    let password_hash = auth_service::hash_password(&input.new_password)?;
    user_repo::set_password(conn, id, &password_hash)?;

    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "user_admin",
        Some("user"),
        Some(id),
        &format!("Reset password for user '{}'", user.username),
        None,
    )?;

    Ok(())
}
