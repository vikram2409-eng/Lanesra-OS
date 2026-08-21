//! Integration Hub (spec §5): Connection References - the portable
//! logical name a Workflow "Call Connector Action" step or a Webhook
//! subscription actually points at, bound to a physical Connection an
//! admin picks per workspace. This is what makes a Solution Package
//! portable across Personal/Test/Production without ever packaging a
//! secret or a physical connection instance (spec table 6/7,
//! INT-AC-02/INT-AC-09) - `export_local_workspace`/`export_solution`
//! never touch this table at all, the same way they never touch
//! `integration_connections`/`integration_secrets`.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::integration::{ConnectionRef, ConnectionRefInput};
use crate::repositories::{integration_connection_ref_repo, integration_connection_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn validate_key(key: &str) -> AppResult<()> {
    let key = key.trim();
    if key.len() < 2 || key.len() > 64 {
        return Err(AppError::Validation("Reference key must be 2-64 characters".into()));
    }
    let mut chars = key.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_lowercase()) {
        return Err(AppError::Validation("Reference key must start with a lowercase letter".into()));
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.') {
        return Err(AppError::Validation("Reference key may only contain lowercase letters, digits, underscores and dots".into()));
    }
    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &ConnectionRefInput, actor_user_id: Option<&str>) -> AppResult<ConnectionRef> {
    require_admin(conn, actor_user_id)?;
    if input.reference_name.trim().is_empty() {
        return Err(AppError::Validation("Reference name is required".into()));
    }
    let key = input.reference_key.trim().to_lowercase();
    validate_key(&key)?;
    if integration_connection_ref_repo::get_by_key(conn, workspace_id, &key)?.is_some() {
        return Err(AppError::Conflict(format!("A connection reference with key '{key}' already exists in this workspace")));
    }
    if let Some(connection_id) = &input.connection_id {
        let connection = integration_connection_repo::get(conn, connection_id)?.ok_or_else(|| AppError::NotFound("Connection".into()))?;
        if connection.workspace_id != workspace_id {
            return Err(AppError::NotFound("Connection".into()));
        }
        if connection.connection_type != input.expected_connection_type {
            return Err(AppError::Validation(format!(
                "This reference expects a '{}' connection, but the selected connection is '{}'",
                input.expected_connection_type, connection.connection_type
            )));
        }
    }
    Ok(integration_connection_ref_repo::insert(
        conn,
        &new_uuid(),
        workspace_id,
        input.reference_name.trim(),
        &key,
        &input.expected_connection_type,
        input.connection_id.as_deref(),
        actor_user_id,
    )?)
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<ConnectionRef>> {
    Ok(integration_connection_ref_repo::list_for_workspace(conn, workspace_id)?)
}

fn get_owned(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<ConnectionRef> {
    let reference = integration_connection_ref_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Connection reference".into()))?;
    if reference.workspace_id != workspace_id {
        return Err(AppError::NotFound("Connection reference".into()));
    }
    Ok(reference)
}

/// Binds (or unbinds, with `connection_id: None`) this reference to a
/// physical Connection - what an admin does after importing a package
/// whose references arrived unresolved (spec table 7's "Import prompts
/// administrator to bind unresolved references before activating
/// dependent automation").
pub fn bind(conn: &Connection, workspace_id: &str, id: &str, connection_id: Option<&str>, actor_user_id: Option<&str>) -> AppResult<ConnectionRef> {
    require_admin(conn, actor_user_id)?;
    let reference = get_owned(conn, workspace_id, id)?;
    if let Some(connection_id) = connection_id {
        let connection = integration_connection_repo::get(conn, connection_id)?.ok_or_else(|| AppError::NotFound("Connection".into()))?;
        if connection.workspace_id != workspace_id {
            return Err(AppError::NotFound("Connection".into()));
        }
        if connection.connection_type != reference.expected_connection_type {
            return Err(AppError::Validation(format!(
                "This reference expects a '{}' connection, but the selected connection is '{}'",
                reference.expected_connection_type, connection.connection_type
            )));
        }
    }
    integration_connection_ref_repo::bind(conn, id, connection_id, actor_user_id)?;
    Ok(integration_connection_ref_repo::get(conn, id)?.expect("just bound"))
}

pub fn delete(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    Ok(integration_connection_ref_repo::delete(conn, id)?)
}

/// The physical Connection this reference currently resolves to, or a
/// clear "unresolved" error - what a Workflow "Call Connector Action"
/// step or Webhook delivery calls before it can actually make a request.
pub(crate) fn resolve(conn: &Connection, workspace_id: &str, reference_key: &str) -> AppResult<String> {
    let reference = integration_connection_ref_repo::get_by_key(conn, workspace_id, reference_key)?
        .ok_or_else(|| AppError::NotFound(format!("Connection reference '{reference_key}'")))?;
    reference.connection_id.ok_or_else(|| {
        AppError::Validation(format!(
            "Connection reference '{reference_key}' isn't bound to a connection yet - bind it under Integration Hub → Connection References first"
        ))
    })
}
