use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::user::{ChangeOwnPassword, Credentials, User};
use crate::repositories::{audit_repo, user_repo};

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Validation(format!("could not hash password: {e}")))
}

fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

pub fn login(conn: &Connection, workspace_id: &str, credentials: &Credentials) -> AppResult<User> {
    let record = user_repo::find_by_username(conn, workspace_id, &credentials.username)?;

    let record = match record {
        Some(r) if r.is_active && verify_password(&credentials.password, &r.password_hash) => r,
        Some(_) => {
            audit_repo::record(
                conn,
                workspace_id,
                None,
                "failed_login",
                Some("user"),
                None,
                &format!("Failed login attempt for '{}'", credentials.username),
                None,
            )?;
            return Err(AppError::Validation("Invalid username or password".into()));
        }
        None => {
            audit_repo::record(
                conn,
                workspace_id,
                None,
                "failed_login",
                Some("user"),
                None,
                &format!("Failed login attempt for '{}'", credentials.username),
                None,
            )?;
            return Err(AppError::Validation("Invalid username or password".into()));
        }
    };

    let roles = user_repo::roles_for_user(conn, &record.id)?;
    let user_id = record.id.clone();
    let user = user_repo::to_public(record, roles);

    audit_repo::record(
        conn,
        workspace_id,
        Some(&user_id),
        "login",
        Some("user"),
        Some(&user_id),
        &format!("User '{}' logged in", user.username),
        None,
    )?;

    Ok(user)
}

/// Lets any authenticated user change their own password, given they can
/// prove they know the current one. Unlike `user_service::set_password`
/// (an Administrator resetting someone else's account), this needs no
/// Administrator role - it's the self-service counterpart the README has
/// been listing as deferred.
pub fn change_own_password(
    conn: &Connection,
    workspace_id: &str,
    actor_user_id: Option<&str>,
    input: &ChangeOwnPassword,
) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    if input.new_password.len() < 8 {
        return Err(AppError::Validation("New password must be at least 8 characters".into()));
    }

    let record = user_repo::find_by_id(conn, actor_id)?.ok_or_else(|| AppError::NotFound("User".into()))?;
    if !verify_password(&input.current_password, &record.password_hash) {
        return Err(AppError::Validation("Current password is incorrect".into()));
    }

    let new_hash = hash_password(&input.new_password)?;
    user_repo::set_password(conn, actor_id, &new_hash)?;

    audit_repo::record(
        conn,
        workspace_id,
        Some(actor_id),
        "password_change",
        Some("user"),
        Some(actor_id),
        &format!("User '{}' changed their own password", record.username),
        None,
    )?;

    Ok(())
}

/// Resolves a session's user_id into the public User shape, or None if the
/// account no longer exists (e.g. deleted between session creation and use).
pub fn resolve_user(conn: &Connection, user_id: &str) -> AppResult<Option<User>> {
    let Some(record) = user_repo::find_by_id(conn, user_id)? else {
        return Ok(None);
    };
    let roles = user_repo::roles_for_user(conn, user_id)?;
    Ok(Some(user_repo::to_public(record, roles)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_and_verifies_round_trip() {
        let hash = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password("correct horse battery staple", &hash));
        assert!(!verify_password("wrong password", &hash));
    }
}
