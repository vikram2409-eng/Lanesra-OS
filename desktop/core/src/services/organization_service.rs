//! Enterprise Access Foundation, Phase 1: "Organization" is a view over
//! `workspaces` plus the org-specific columns 0051_organization_and_org_units.sql
//! added, not a separate table - see that migration's own doc comment.
//! `get_or_bootstrap` is the one place that guarantees every workspace has
//! a real org_code and a root Organization Unit, whether it's a
//! pre-existing workspace the migration already backfilled or a brand new
//! one created after this code shipped.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::org_unit::OrgUnitInput;
use crate::models::organization::{Organization, OrganizationUpdate};
use crate::repositories::{organization_repo, org_unit_repo, user_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage the Organization".into()));
    }
    Ok(())
}

/// Idempotent: returns the existing Organization if this workspace already
/// has one (the normal case - the migration backfills every pre-existing
/// workspace), otherwise synthesizes org_code and a root Organization Unit
/// on the fly. Every other Access Foundation service calls this first so
/// the invariant "every workspace has an Organization and a root
/// Organization Unit" always holds by the time anything else runs.
pub fn get_or_bootstrap(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Organization> {
    let mut org = organization_repo::get(conn, workspace_id)?.ok_or_else(|| AppError::NotFound("Workspace".into()))?;
    if org.root_org_unit_id.is_empty() {
        let root_name = org.name.clone();
        let root_input = OrgUnitInput {
            name: root_name,
            unit_type: "Division".to_string(),
            parent_org_unit_id: None,
            manager_user_id: None,
            effective_from: None,
            effective_to: None,
        };
        let id = crate::domain::ids::new_uuid();
        let path = format!("/{id}/");
        let root_unit = org_unit_repo::create(conn, &id, workspace_id, &root_input, &path, 0, actor_user_id)?;
        organization_repo::set_root_org_unit(conn, workspace_id, &root_unit.id)?;
        org.root_org_unit_id = root_unit.id;
    }
    if org.code.is_empty() {
        let code = format!("ORG-{}", &crate::domain::ids::new_uuid()[..8].to_uppercase());
        org = organization_repo::update(conn, workspace_id, &OrganizationUpdate { code, status: org.status })?;
    }
    Ok(org)
}

pub fn get(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Organization> {
    get_or_bootstrap(conn, workspace_id, actor_user_id)
}

pub fn update(conn: &Connection, workspace_id: &str, input: &OrganizationUpdate, actor_user_id: Option<&str>) -> AppResult<Organization> {
    require_admin(conn, actor_user_id)?;
    if input.code.trim().is_empty() {
        return Err(AppError::Validation("Organization code is required".into()));
    }
    if !["Active", "Inactive"].contains(&input.status.as_str()) {
        return Err(AppError::Validation("Status must be Active or Inactive".into()));
    }
    get_or_bootstrap(conn, workspace_id, actor_user_id)?;
    Ok(organization_repo::update(conn, workspace_id, input)?)
}
