//! Solution Packages & Admin IA design spec, Phase 4: named, scoped
//! Solutions - the Dynamics-365-style "build a solution in test, export
//! it, import it in prod" workflow. See migration 0031's own comment for
//! why this needed no new "environment" concept (a workspace already is
//! one) and `industry_package_service::export_solution` for how a
//! Solution's curated membership turns into the same `.lanesra`-shaped
//! manifest the existing import/install pipeline already knows how to
//! consume in a second workspace.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::solution::{Solution, SolutionDetail, SolutionInput, SolutionMember, SolutionMemberInput, SolutionUpdate};
use crate::repositories::{solution_component_repo, solution_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

const DEFAULT_VERSION: &str = "1.0.0.0";

fn validate_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("Solution name is required".into()));
    }
    if name.len() > 128 {
        return Err(AppError::Validation("Solution name must be 128 characters or fewer".into()));
    }
    Ok(name.to_string())
}

fn normalize_version(version: Option<&str>) -> String {
    match version.map(str::trim) {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => DEFAULT_VERSION.to_string(),
    }
}

/// A `publisher_id` the caller supplied must actually be a registered
/// publisher in this workspace - the same ownership check
/// `industry_package_service::get_owned_package` does for packages,
/// applied here so a Solution can never be mislabeled under a publisher
/// from a different workspace. `None` resolves to this workspace's own
/// `local` publisher (see `publisher_service::ensure_defaults`), which is
/// the sensible default for a Solution being assembled from hand-built
/// customizations - the same publisher `export_local_workspace` already
/// uses for the equivalent whole-workspace export.
fn resolve_publisher_id(conn: &Connection, workspace_id: &str, publisher_id: Option<&str>) -> AppResult<String> {
    match publisher_id {
        Some(id) => {
            let publisher =
                super::publisher_service::list(conn, workspace_id)?.into_iter().find(|p| p.id == id).ok_or_else(|| {
                    AppError::Validation("That publisher isn't registered in this workspace".into())
                })?;
            Ok(publisher.id)
        }
        None => {
            super::publisher_service::ensure_defaults(conn, workspace_id)?;
            let local = super::publisher_service::list(conn, workspace_id)?
                .into_iter()
                .find(|p| p.is_local)
                .expect("ensure_defaults just seeded 'local'");
            Ok(local.id)
        }
    }
}

fn get_owned(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<Solution> {
    let solution = solution_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Solution".into()))?;
    if solution.workspace_id != workspace_id {
        return Err(AppError::NotFound("Solution".into()));
    }
    Ok(solution)
}

pub fn create(conn: &Connection, workspace_id: &str, input: &SolutionInput, actor_user_id: Option<&str>) -> AppResult<Solution> {
    require_admin(conn, actor_user_id)?;
    let name = validate_name(&input.name)?;
    let version = normalize_version(input.version.as_deref());
    let publisher_id = resolve_publisher_id(conn, workspace_id, input.publisher_id.as_deref())?;
    if solution_repo::get_by_name(conn, workspace_id, &name)?.is_some() {
        return Err(AppError::Conflict(format!("A solution named '{name}' already exists in this workspace")));
    }
    Ok(solution_repo::insert(
        conn,
        &new_uuid(),
        workspace_id,
        &name,
        input.description.as_deref(),
        &version,
        Some(&publisher_id),
        actor_user_id,
    )?)
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Solution>> {
    Ok(solution_repo::list_for_workspace(conn, workspace_id)?)
}

/// Ownership-checked single lookup - what
/// `industry_package_service::export_solution` calls for the solution's
/// own name/version/id before reading its members.
pub fn get(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<Solution> {
    get_owned(conn, workspace_id, id)
}

/// The Solutions tab's detail view: the `Solution` itself plus every
/// curated member resolved to the same display shape the Components tab
/// uses (`WorkspaceComponent`) - reuses
/// `solution_component_service::list_for_workspace` rather than a second
/// query, since a Solution's members are always a subset of that list (a
/// membership row whose component has since been deleted just doesn't
/// match anything and is silently dropped from the resolved view, the
/// same tolerance `export_solution` applies at export time).
pub fn get_detail(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<SolutionDetail> {
    let solution = get_owned(conn, workspace_id, id)?;
    let member_rows = solution_repo::list_members(conn, id)?;
    let member_keys: std::collections::HashSet<(String, String)> =
        member_rows.iter().map(|m| (m.artifact_type.clone(), m.metadata_id.clone())).collect();
    let all_components = super::solution_component_service::list_for_workspace(conn, workspace_id)?;
    let members = all_components
        .into_iter()
        .filter(|c| member_keys.contains(&(c.component.artifact_type.clone(), c.component.metadata_id.clone())))
        .collect();
    Ok(SolutionDetail { solution, members })
}

pub fn update(conn: &Connection, workspace_id: &str, id: &str, input: &SolutionUpdate, actor_user_id: Option<&str>) -> AppResult<Solution> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    let name = validate_name(&input.name)?;
    let version = normalize_version(Some(&input.version));
    let publisher_id = resolve_publisher_id(conn, workspace_id, input.publisher_id.as_deref())?;
    if let Some(existing) = solution_repo::get_by_name(conn, workspace_id, &name)? {
        if existing.id != id {
            return Err(AppError::Conflict(format!("A solution named '{name}' already exists in this workspace")));
        }
    }
    Ok(solution_repo::update(conn, id, &name, input.description.as_deref(), &version, Some(&publisher_id), actor_user_id)?)
}

pub fn delete(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    Ok(solution_repo::delete(conn, id)?)
}

/// Curates a component into a Solution - only ever a component that
/// already exists and is tagged in `solution_components` (migration
/// 0030), whether hand-built (owned by `local`) or brought in from an
/// installed package. Rejecting an unknown `(artifact_type, metadata_id)`
/// here, rather than silently recording a membership row that would
/// never resolve to anything, is the same "fail closed on a bad
/// reference" stance `export_local_workspace`/`apply_update` take for a
/// dangling relationship reference.
pub fn add_component(conn: &Connection, workspace_id: &str, solution_id: &str, input: &SolutionMemberInput, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, solution_id)?;
    if solution_component_repo::get(conn, workspace_id, &input.artifact_type, &input.metadata_id)?.is_none() {
        return Err(AppError::NotFound("Component".into()));
    }
    solution_repo::add_member(conn, &new_uuid(), solution_id, &input.artifact_type, &input.metadata_id, actor_user_id)?;
    Ok(())
}

pub fn remove_component(conn: &Connection, workspace_id: &str, solution_id: &str, artifact_type: &str, metadata_id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, solution_id)?;
    Ok(solution_repo::remove_member(conn, solution_id, artifact_type, metadata_id)?)
}

/// The raw `(artifact_type, metadata_id)` pairs a Solution currently
/// curates - what `industry_package_service::export_solution` reads to
/// build a scoped manifest, deliberately not going through the
/// display-oriented `get_detail`/`WorkspaceComponent` resolution export
/// doesn't need.
pub(crate) fn list_member_refs(conn: &Connection, solution_id: &str) -> AppResult<Vec<SolutionMember>> {
    Ok(solution_repo::list_members(conn, solution_id)?)
}
