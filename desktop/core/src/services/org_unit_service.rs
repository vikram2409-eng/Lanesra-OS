//! Enterprise Access Foundation, Phase 1: Organization Units (spec §1.2).
//! Hierarchy is a materialized path (`/root-id/child-id/.../`) + depth, not
//! a closure table - see 0051_organization_and_org_units.sql's own doc
//! comment for why. Every workspace has exactly one root unit, created by
//! `organization_service::get_or_bootstrap`; this service's own `create`
//! always requires a parent, so a second top-level unit can never be
//! created through the public API - the hierarchy stays single-rooted.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::org_unit::{OrgUnit, OrgUnitInput, OrgUnitMoveImpact, OrgUnitUpdate, ORG_UNIT_TYPES};
use crate::repositories::org_unit_repo;
use crate::services::{access_service, custom_object_service, ownership_service};

/// Administrator always passes (unchanged); a non-Administrator additionally
/// passes with an explicit Access Role grant on "OrgUnit" - see
/// `access_service::require_admin_or_explicit_update`'s own doc comment.
fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    access_service::require_admin_or_explicit_update(conn, actor_user_id, "OrgUnit", "Only an Administrator can manage Organization Units")
}

fn validate_shape(input_name: &str, unit_type: &str) -> AppResult<()> {
    if input_name.trim().is_empty() {
        return Err(AppError::Validation("Name is required".into()));
    }
    if !ORG_UNIT_TYPES.contains(&unit_type) {
        return Err(AppError::Validation(format!("Unit type must be one of: {}", ORG_UNIT_TYPES.join(", "))));
    }
    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &OrgUnitInput, actor_user_id: Option<&str>) -> AppResult<OrgUnit> {
    require_admin(conn, actor_user_id)?;
    validate_shape(&input.name, &input.unit_type)?;
    let parent_id = input.parent_org_unit_id.as_deref().ok_or_else(|| {
        AppError::Validation("A new Organization Unit must have a parent - every workspace has exactly one root unit already".into())
    })?;
    let parent = org_unit_repo::get(conn, parent_id)?.ok_or_else(|| AppError::NotFound("Parent Organization Unit".into()))?;
    if parent.workspace_id != workspace_id {
        return Err(AppError::Validation("Parent Organization Unit belongs to a different workspace".into()));
    }
    let id = crate::domain::ids::new_uuid();
    let path = format!("{}{id}/", parent.path);
    org_unit_repo::create(conn, &id, workspace_id, input, &path, parent.depth + 1, actor_user_id).map_err(AppError::from)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<OrgUnit>> {
    Ok(org_unit_repo::get(conn, id)?)
}

/// Flat list ordered by path - a parent always sorts before its own
/// descendants, so the frontend builds the tree straight from this without
/// a second pass or a separate tree-shaped response. Bootstraps the
/// workspace's Organization (and its root unit) first, the same as
/// `organization_service::get` does - a fresh workspace has no root unit
/// until *something* calls `get_or_bootstrap`, and an admin who opens
/// "Organization Units" before ever opening "Organization" must still see
/// (and be able to nest under) a root, not an empty list with no valid
/// parent to create anything under.
pub fn list_tree(conn: &Connection, workspace_id: &str) -> AppResult<Vec<OrgUnit>> {
    super::organization_service::get_or_bootstrap(conn, workspace_id, None)?;
    Ok(org_unit_repo::list(conn, workspace_id)?)
}

pub fn update(conn: &Connection, id: &str, input: &OrgUnitUpdate, actor_user_id: Option<&str>) -> AppResult<OrgUnit> {
    require_admin(conn, actor_user_id)?;
    validate_shape(&input.name, &input.unit_type)?;
    org_unit_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Organization Unit".into()))?;
    if !["Active", "Inactive"].contains(&input.status.as_str()) {
        return Err(AppError::Validation("Status must be Active or Inactive".into()));
    }
    Ok(org_unit_repo::update(conn, id, input, actor_user_id)?)
}

/// Every object_key this workspace can own a record under - the fixed
/// built-ins plus every active Custom Object - used by the move-impact
/// scan. Not cached: an admin previewing/committing a move is a rare,
/// deliberate action, not a hot path.
fn all_owned_object_keys(conn: &Connection, workspace_id: &str) -> AppResult<Vec<String>> {
    let mut keys: Vec<String> = ownership_service::BUILTIN_OWNED_OBJECT_KEYS.iter().map(|s| s.to_string()).collect();
    for def in custom_object_service::list(conn, workspace_id, true)? {
        keys.push(def.key);
    }
    Ok(keys)
}

/// True when `candidate_id` is `unit_id` itself or a descendant of it -
/// the cycle check a move must reject (you can't move a unit under its own
/// child).
fn is_self_or_descendant(unit: &OrgUnit, candidate: &OrgUnit) -> bool {
    candidate.id == unit.id || candidate.path.starts_with(&unit.path)
}

fn validate_move(conn: &Connection, workspace_id: &str, id: &str, new_parent_id: &str) -> AppResult<(OrgUnit, OrgUnit)> {
    let unit = org_unit_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Organization Unit".into()))?;
    if unit.parent_org_unit_id.is_none() {
        return Err(AppError::Validation("The root Organization Unit can't be moved".into()));
    }
    let new_parent = org_unit_repo::get(conn, new_parent_id)?.ok_or_else(|| AppError::NotFound("New parent Organization Unit".into()))?;
    if new_parent.workspace_id != workspace_id {
        return Err(AppError::Validation("New parent Organization Unit belongs to a different workspace".into()));
    }
    if is_self_or_descendant(&unit, &new_parent) {
        return Err(AppError::Validation("Can't move an Organization Unit under itself or one of its own descendants".into()));
    }
    if new_parent.id == unit.parent_org_unit_id.clone().unwrap_or_default() {
        return Err(AppError::Validation("This is already the parent".into()));
    }
    Ok((unit, new_parent))
}

pub fn preview_move(conn: &Connection, workspace_id: &str, id: &str, new_parent_id: &str, actor_user_id: Option<&str>) -> AppResult<OrgUnitMoveImpact> {
    require_admin(conn, actor_user_id)?;
    let (unit, _new_parent) = validate_move(conn, workspace_id, id, new_parent_id)?;
    let subtree = org_unit_repo::list_descendants_by_path(conn, workspace_id, &unit.path)?;
    let descendant_unit_count = (subtree.len() as i64) - 1; // exclude the unit itself
    let object_keys = all_owned_object_keys(conn, workspace_id)?;
    let mut owned_record_counts = Vec::new();
    for object_key in &object_keys {
        let mut total = 0i64;
        for su in &subtree {
            total += ownership_service::count_owned_in_org_unit(conn, object_key, &su.id)?;
        }
        if total > 0 {
            owned_record_counts.push((object_key.clone(), total));
        }
    }
    Ok(OrgUnitMoveImpact { descendant_unit_count, owned_record_counts })
}

/// Re-validates (never trusts a stale preview) then commits: rewrites
/// path/depth for the whole moved subtree in one bounded UPDATE, then
/// points the moved unit's own parent_org_unit_id at its new parent.
pub fn move_unit(conn: &Connection, workspace_id: &str, id: &str, new_parent_id: &str, actor_user_id: Option<&str>) -> AppResult<OrgUnit> {
    require_admin(conn, actor_user_id)?;
    let (unit, new_parent) = validate_move(conn, workspace_id, id, new_parent_id)?;
    let old_prefix = unit.path.clone();
    let new_prefix = format!("{}{id}/", new_parent.path);
    let depth_delta = (new_parent.depth + 1) - unit.depth;
    org_unit_repo::update_path_for_subtree(conn, &old_prefix, &new_prefix, depth_delta, actor_user_id)?;
    org_unit_repo::set_parent_and_path(conn, id, Some(new_parent_id), &new_prefix, new_parent.depth + 1, actor_user_id)?;
    crate::repositories::audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "org_unit_moved",
        Some("OrgUnit"),
        Some(id),
        &format!("Moved '{}' under '{}'", unit.name, new_parent.name),
        None,
    )?;
    org_unit_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Organization Unit".into()))
}

/// Blocked while any child Organization Unit still points at this one, or
/// any Work Team's primary_org_unit_id does - the same "block delete when
/// dependents exist" rule every other entity in this product follows.
pub fn delete(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let unit = org_unit_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Organization Unit".into()))?;
    if unit.parent_org_unit_id.is_none() {
        return Err(AppError::Validation("The root Organization Unit can't be deleted".into()));
    }
    let children = org_unit_repo::list_children(conn, id)?;
    if !children.is_empty() {
        return Err(AppError::Validation(format!("Can't delete '{}' - {} child Organization Unit(s) still exist. Move or delete them first.", unit.name, children.len())));
    }
    Ok(org_unit_repo::delete(conn, id)?)
}
