//! Integration Hub (spec §8): inbound API clients - the credentials an
//! external caller presents to the REST API (`server/src/routes.rs`'s new
//! `/api/v1` router). Issued key shape is `"{client_id}.{secret}"`; only
//! `sha256(secret)` is ever stored (`secret_service::hash_api_secret`) -
//! Lanesra only ever needs to verify a presented key, never re-send it
//! anywhere, unlike a Connection's own auth secret which genuinely must
//! be recoverable (see `secret_service`'s own doc comment for that
//! distinction).

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::integration::{ApiClient, ApiClientInput, IssuedApiClient};
use crate::repositories::integration_api_client_repo;

/// Spec §8.2's suggested scope vocabulary.
pub const VALID_SCOPES: &[&str] = &[
    "objects.read",
    "objects.write",
    "metadata.read",
    "search.read",
    "bulk.read",
    "bulk.write",
    "webhooks.manage",
    "events.read",
    "admin.integration.read",
    "admin.integration.manage",
];

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn validate_scopes(scopes: &[String]) -> AppResult<()> {
    for scope in scopes {
        if !VALID_SCOPES.contains(&scope.as_str()) {
            return Err(AppError::Validation(format!("Unknown scope '{scope}'")));
        }
    }
    Ok(())
}

fn generate_client_id() -> String {
    format!("client_{}", super::secret_service::generate_random_secret(8))
}

pub fn create(conn: &Connection, workspace_id: &str, input: &ApiClientInput, actor_user_id: Option<&str>) -> AppResult<IssuedApiClient> {
    require_admin(conn, actor_user_id)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("API client name is required".into()));
    }
    validate_scopes(&input.scopes)?;
    let id = new_uuid();
    let client_id = generate_client_id();
    let scopes_json = serde_json::to_string(&input.scopes).unwrap_or_else(|_| "[]".into());
    let client = integration_api_client_repo::insert(
        conn,
        &id,
        workspace_id,
        input.name.trim(),
        &client_id,
        &scopes_json,
        input.allowed_cidr.as_deref(),
        input.owner_user_id.as_deref(),
        actor_user_id,
    )?;
    let secret = super::secret_service::generate_random_secret(24);
    let hash = super::secret_service::hash_api_secret(&secret);
    integration_api_client_repo::insert_credential(conn, &new_uuid(), workspace_id, &id, &hash)?;
    Ok(IssuedApiClient { client, api_key: format!("{client_id}.{secret}") })
}

/// Issues a fresh secret for an already-existing client, invalidating the
/// old one - spec table 22's "Rotate Secret".
pub fn rotate_secret(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<IssuedApiClient> {
    require_admin(conn, actor_user_id)?;
    let client = get_owned(conn, workspace_id, id)?;
    let secret = super::secret_service::generate_random_secret(24);
    let hash = super::secret_service::hash_api_secret(&secret);
    integration_api_client_repo::insert_credential(conn, &new_uuid(), workspace_id, id, &hash)?;
    Ok(IssuedApiClient { api_key: format!("{}.{secret}", client.client_id), client })
}

fn get_owned(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<ApiClient> {
    let client = integration_api_client_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("API client".into()))?;
    if client.workspace_id != workspace_id {
        return Err(AppError::NotFound("API client".into()));
    }
    Ok(client)
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<ApiClient>> {
    Ok(integration_api_client_repo::list_for_workspace(conn, workspace_id)?)
}

pub fn revoke(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    Ok(integration_api_client_repo::set_status(conn, id, "revoked", actor_user_id)?)
}

pub fn reactivate(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    Ok(integration_api_client_repo::set_status(conn, id, "active", actor_user_id)?)
}

pub fn delete(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    Ok(integration_api_client_repo::delete(conn, id)?)
}

/// Verifies a presented `Authorization: Bearer {client_id}.{secret}`
/// value against this workspace's active clients, returning the matched
/// client and its scopes - what the new `/api/v1` axum auth middleware
/// calls on every request. Touches `last_used_at` on success.
pub fn authenticate(conn: &Connection, workspace_id: &str, presented_key: &str) -> AppResult<ApiClient> {
    let (client_id, secret) = presented_key
        .split_once('.')
        .ok_or_else(|| AppError::Validation("Malformed API key".into()))?;
    let client = integration_api_client_repo::get_by_client_id(conn, client_id)?.ok_or_else(|| AppError::Validation("Invalid API key".into()))?;
    if client.workspace_id != workspace_id {
        return Err(AppError::Validation("Invalid API key".into()));
    }
    if client.status != "active" {
        return Err(AppError::Validation(format!("This API client is {}", client.status)));
    }
    let expected_hash = integration_api_client_repo::current_hash_for(conn, &client.id)?.ok_or_else(|| AppError::Validation("Invalid API key".into()))?;
    if super::secret_service::hash_api_secret(secret) != expected_hash {
        return Err(AppError::Validation("Invalid API key".into()));
    }
    integration_api_client_repo::touch_last_used(conn, &client.id)?;
    Ok(client)
}

pub fn has_scope(client: &ApiClient, scope: &str) -> bool {
    client.scopes.iter().any(|s| s == scope)
}
