//! Solution Packages & Admin IA design spec, Phase 3: component-tagging.
//! See migration 0030's own comment for the full design (why a single
//! `tag_local` + `retag` pair, called from exactly 10 existing creation
//! functions plus `industry_package_service::run_install`, is enough to
//! give every component in a workspace a real owner with zero signature
//! changes anywhere else).

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::AppResult;
use crate::models::solution_component::{LocalWorkspaceSummary, SolutionComponent, WorkspaceComponent};
use crate::repositories::solution_component_repo;

/// Tags a just-created component with this workspace's `local` publisher.
/// Called unconditionally at the end of every one of the 10 component-
/// creation call sites (see migration 0030) - the ordinary admin-UI path
/// has no other context to tag with, and a package install that reuses
/// the same function immediately corrects this via `retag` right after.
/// Idempotent and cheap: `ensure_defaults` is a single indexed SELECT
/// once the workspace's publishers already exist (the overwhelmingly
/// common case).
pub fn tag_local(conn: &Connection, workspace_id: &str, artifact_type: &str, metadata_id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    super::publisher_service::ensure_defaults(conn, workspace_id)?;
    let publishers = super::publisher_service::list(conn, workspace_id)?;
    let local = publishers.into_iter().find(|p| p.is_local).expect("ensure_defaults just seeded 'local'");
    solution_component_repo::upsert(conn, &new_uuid(), workspace_id, artifact_type, metadata_id, &local.id, None, actor_user_id)?;
    Ok(())
}

/// Overwrites an already-`tag_local`-tagged component's owner with the
/// installing package's real publisher, and records which install owns
/// it - called by `industry_package_service::run_install` for every
/// artifact it buffers, right where it already writes `package_artifacts`
/// rows for the same `(artifact_type, metadata_id)` pairs.
pub fn retag(
    conn: &Connection,
    workspace_id: &str,
    artifact_type: &str,
    metadata_id: &str,
    publisher_id: &str,
    installed_app_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    solution_component_repo::upsert(conn, &new_uuid(), workspace_id, artifact_type, metadata_id, publisher_id, Some(installed_app_id), actor_user_id)?;
    Ok(())
}

/// Every component in the workspace, joined for display - the "Components"
/// tab's data source, superseding the narrower
/// `industry_package_service::list_artifacts_for_workspace` (which only
/// ever saw package-installed artifacts) now that hand-built components
/// are tagged too.
pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<WorkspaceComponent>> {
    let rows = solution_component_repo::list_for_workspace(conn, workspace_id)?;
    Ok(rows
        .into_iter()
        .map(|(component, publisher_key, publisher_name, is_local, installed_app_name)| WorkspaceComponent {
            component,
            publisher_key,
            publisher_name,
            is_local,
            installed_app_name,
        })
        .collect())
}

/// Every component still owned by this workspace's `local` publisher -
/// the "Local Workspace" Unmanaged grouping's contents and what
/// `industry_package_service::export_local_workspace` reads back into a
/// manifest.
pub fn list_local(conn: &Connection, workspace_id: &str) -> AppResult<Vec<SolutionComponent>> {
    super::publisher_service::ensure_defaults(conn, workspace_id)?;
    let publishers = super::publisher_service::list(conn, workspace_id)?;
    let local = publishers.into_iter().find(|p| p.is_local).expect("ensure_defaults just seeded 'local'");
    Ok(solution_component_repo::list_for_publisher(conn, workspace_id, &local.id)?)
}

/// The Managed/Unmanaged distinction's Unmanaged half, made visible
/// without ever writing a fake `app_packages` row: a count of everything
/// still owned by `local`, broken down by type - what the Solution
/// Packages tab renders as a synthetic "Local Workspace" row alongside
/// real installed (Managed) packages. An empty summary (a workspace that
/// has only ever installed reference packages, never built anything by
/// hand) is a legitimate, common result, not an error.
pub fn local_workspace_summary(conn: &Connection, workspace_id: &str) -> AppResult<LocalWorkspaceSummary> {
    super::publisher_service::ensure_defaults(conn, workspace_id)?;
    let publishers = super::publisher_service::list(conn, workspace_id)?;
    let local = publishers.into_iter().find(|p| p.is_local).expect("ensure_defaults just seeded 'local'");
    let components = solution_component_repo::list_for_publisher(conn, workspace_id, &local.id)?;
    let mut by_type: Vec<(String, i64)> = Vec::new();
    for component in &components {
        match by_type.iter_mut().find(|(t, _)| *t == component.artifact_type) {
            Some((_, count)) => *count += 1,
            None => by_type.push((component.artifact_type.clone(), 1)),
        }
    }
    by_type.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(LocalWorkspaceSummary { publisher_id: local.id, component_count: components.len() as i64, components_by_type: by_type })
}
