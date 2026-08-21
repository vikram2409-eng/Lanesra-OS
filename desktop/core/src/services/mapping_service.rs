//! Integration Hub (spec §14): reusable field Mappings - saved once from
//! a CSV import wizard run (or hand-built for a future Integration Job),
//! so the same source-column -> target-field/transform/default shape
//! doesn't need re-entering on every recurring import.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::integration::{Mapping, MappingInput};
use crate::repositories::integration_mapping_repo;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

const OPERATIONS: &[&str] = &["insert", "update", "upsert"];
const DUPLICATE_POLICIES: &[&str] = &["skip", "update_matched", "create_new"];

fn validate(input: &MappingInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Mapping name is required".into()));
    }
    if !OPERATIONS.contains(&input.operation.as_str()) {
        return Err(AppError::Validation(format!("Invalid operation '{}'", input.operation)));
    }
    if !DUPLICATE_POLICIES.contains(&input.duplicate_policy.as_str()) {
        return Err(AppError::Validation(format!("Invalid duplicate policy '{}'", input.duplicate_policy)));
    }
    if input.operation != "insert" && input.match_key.is_none() {
        return Err(AppError::Validation("An update/upsert mapping needs a match key".into()));
    }
    if input.field_map.is_empty() {
        return Err(AppError::Validation("At least one field mapping is required".into()));
    }
    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &MappingInput, actor_user_id: Option<&str>) -> AppResult<Mapping> {
    require_admin(conn, actor_user_id)?;
    validate(input)?;
    let field_map_json = serde_json::to_string(&input.field_map).unwrap_or_else(|_| "[]".to_string());
    Ok(integration_mapping_repo::insert(
        conn,
        &new_uuid(),
        workspace_id,
        input.name.trim(),
        &input.target_object_key,
        &input.operation,
        input.match_key.as_deref(),
        &field_map_json,
        &input.duplicate_policy,
        actor_user_id,
    )?)
}

fn get_owned(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<Mapping> {
    let mapping = integration_mapping_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Mapping".into()))?;
    if mapping.workspace_id != workspace_id {
        return Err(AppError::NotFound("Mapping".into()));
    }
    Ok(mapping)
}

pub fn get(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<Mapping> {
    get_owned(conn, workspace_id, id)
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Mapping>> {
    Ok(integration_mapping_repo::list_for_workspace(conn, workspace_id)?)
}

pub fn delete(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    Ok(integration_mapping_repo::delete(conn, id)?)
}

/// The one shared column-value transform every mapping-consuming caller
/// (CSV import, and later Integration Jobs) applies before writing a
/// field - spec §12/§14's transform vocabulary. `"date"` is left as-is
/// rather than guessing a source format/locale - real date normalization
/// needs a per-mapping format hint this MVP doesn't collect yet, stated
/// plainly rather than silently mis-parsing.
pub fn apply_transform(transform: Option<&str>, raw: &str) -> String {
    match transform {
        Some("trim") => raw.trim().to_string(),
        Some("uppercase") => raw.trim().to_uppercase(),
        Some("lowercase") => raw.trim().to_lowercase(),
        Some("numeric") => raw.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect(),
        _ => raw.to_string(),
    }
}
