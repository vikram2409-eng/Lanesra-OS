//! Solution Packages & Admin IA design spec, Phase 2: a real Publisher
//! registry and the publisher/namespace scope the user explicitly asked
//! to "build it now" - not stub. See migration 0029's own comment for
//! the schema and the two auto-seeded publishers every workspace gets.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::publisher::{Publisher, PublisherInput};
use crate::repositories::publisher_repo;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// `lanesra` owns every bundled reference package; `local` is the
/// implicit home for hand-built customizations (see migration 0029).
/// Reserved so an admin can never register a lookalike, and so
/// `import_package` can rely on `lanesra` always resolving for the
/// bundled reference packages with zero setup.
const RESERVED_KEYS: &[&str] = &["lanesra", "local"];

/// Lowercase ascii letters/digits/underscore, must start with a letter,
/// 2-32 characters - matches `ManifestObject`/`ManifestField`'s own
/// "explicit, deterministic key" reasoning: this becomes a literal
/// dotted-namespace prefix on every package_id under it
/// (`"<key>.<name>"`), so it needs the same no-surprises shape a URL
/// path segment would.
pub fn validate_key(key: &str) -> AppResult<()> {
    let key = key.trim();
    if key.len() < 2 || key.len() > 32 {
        return Err(AppError::Validation("Publisher key must be 2-32 characters".into()));
    }
    let mut chars = key.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_lowercase() {
        return Err(AppError::Validation("Publisher key must start with a lowercase letter".into()));
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(AppError::Validation(
            "Publisher key may only contain lowercase letters, digits and underscores".into(),
        ));
    }
    Ok(())
}

fn seed_if_missing(
    conn: &Connection,
    workspace_id: &str,
    key: &str,
    name: &str,
    description: &str,
    is_official: bool,
    is_local: bool,
) -> AppResult<()> {
    if publisher_repo::get_by_key(conn, workspace_id, key)?.is_none() {
        publisher_repo::insert(conn, &new_uuid(), workspace_id, key, name, Some(description), is_official, is_local, None)?;
    }
    Ok(())
}

/// Idempotent - safe to call on every request that touches publishers,
/// not just at workspace setup, so a workspace created before migration
/// 0029 self-heals the first time it needs either default rather than
/// requiring a data migration to backfill existing rows.
pub fn ensure_defaults(conn: &Connection, workspace_id: &str) -> AppResult<()> {
    seed_if_missing(
        conn,
        workspace_id,
        "lanesra",
        "Lanesra OS",
        "The official publisher of every bundled industry reference package.",
        true,
        false,
    )?;
    seed_if_missing(
        conn,
        workspace_id,
        "local",
        "Local Workspace",
        "The implicit home for whatever you build by hand in this workspace, rather than install from a package.",
        false,
        true,
    )?;
    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &PublisherInput, actor_user_id: Option<&str>) -> AppResult<Publisher> {
    require_admin(conn, actor_user_id)?;
    ensure_defaults(conn, workspace_id)?;
    let key = input.key.trim().to_lowercase();
    validate_key(&key)?;
    if RESERVED_KEYS.contains(&key.as_str()) {
        return Err(AppError::Validation(format!("'{key}' is a reserved publisher key")));
    }
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Publisher name is required".into()));
    }
    if publisher_repo::get_by_key(conn, workspace_id, &key)?.is_some() {
        return Err(AppError::Conflict(format!("A publisher with key '{key}' already exists in this workspace")));
    }
    Ok(publisher_repo::insert(
        conn,
        &new_uuid(),
        workspace_id,
        &key,
        input.name.trim(),
        input.description.as_deref(),
        false,
        false,
        actor_user_id,
    )?)
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Publisher>> {
    ensure_defaults(conn, workspace_id)?;
    Ok(publisher_repo::list_for_workspace(conn, workspace_id)?)
}

/// Resolves a `package_id`'s namespace prefix (the text before its first
/// `.`) to a registered `Publisher` in this workspace - the enforcement
/// point `industry_package_service::import_package` calls so every
/// import is actually backed by a real, registered publisher instead of
/// an unvalidated string convention.
pub fn resolve_for_package_id(conn: &Connection, workspace_id: &str, package_id: &str) -> AppResult<Publisher> {
    let key = package_id.split('.').next().filter(|k| !k.is_empty()).ok_or_else(|| {
        AppError::Validation(format!("Package id '{package_id}' must be namespaced as '<publisher-key>.<name>' (e.g. 'acme.inspection')"))
    })?;
    publisher_repo::get_by_key(conn, workspace_id, key)?.ok_or_else(|| {
        AppError::Validation(format!(
            "'{key}' isn't a registered publisher in this workspace yet - register it under Admin → Deployment Management → Publishers before importing a package under that namespace"
        ))
    })
}
