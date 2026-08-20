//! Dashboard customization Phase 1: multiple named dashboard layouts per
//! workspace, each an ordered list of widgets, with role-based assignment
//! and a required default fallback - structurally the same feature as
//! Screen/App Builder (`screen_layout_service`), just at the workspace
//! level instead of per entity_type. See that module's doc comment for
//! the shared design rationale (opaque widget config, draft/publish,
//! role resolution); this module mirrors it function-for-function.
//!
//! No published layout at all (the common case until an admin builds
//! one) resolves to `None` here - callers fall back to whatever the
//! Dashboard rendered before this feature existed (the workspace-wide
//! `dashboard_kpi_prefs` KPI selection from FR-KPI), exactly the same
//! "zero effect until published" contract `resolve_effective_layout`
//! already has for Screen layouts.

use rusqlite::Connection;
use serde::Serialize;

use crate::domain::{AppError, AppResult};
use crate::models::dashboard_layout::{DashboardLayout, DashboardLayoutInput, DashboardLayoutUpdate, DashboardWidget, DashboardWidgets};
use crate::repositories::{dashboard_layout_repo, user_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// Per-app scoped automation - see
/// `business_rule_service::require_valid_app_id`'s identical doc comment.
fn require_valid_app_id(conn: &Connection, workspace_id: &str, app_id: Option<&str>) -> AppResult<()> {
    let Some(app_id) = app_id else { return Ok(()) };
    let app = super::app_service::get(conn, app_id).map_err(|_| AppError::Validation("App not found".into()))?;
    if app.workspace_id != workspace_id {
        return Err(AppError::Validation("App not found".into()));
    }
    Ok(())
}

fn empty_widgets() -> DashboardWidgets {
    DashboardWidgets { widgets: vec![] }
}

fn seeded_widgets(initial_kpi_keys: &[String]) -> DashboardWidgets {
    DashboardWidgets {
        widgets: initial_kpi_keys
            .iter()
            .map(|key| DashboardWidget {
                id: crate::domain::ids::new_uuid(),
                kind: "kpi".into(),
                config: serde_json::json!({ "kpi_key": key }),
            })
            .collect(),
    }
}

fn hydrate(row: (DashboardLayout, String, String, Option<String>)) -> AppResult<DashboardLayout> {
    let (mut layout, roles_json, draft_json, published_json) = row;
    layout.roles = serde_json::from_str(&roles_json).unwrap_or_default();
    layout.draft = serde_json::from_str(&draft_json).unwrap_or_default();
    layout.published = published_json.and_then(|s| serde_json::from_str(&s).ok());
    Ok(layout)
}

/// Every dashboard layout for this workspace, default first -
/// auto-provisions a bare, unpublished Default layout (no widgets) the
/// first time this is called for a workspace with none yet, the same
/// "always has a Default to select" invariant `screen_layout_service`
/// keeps. Being unpublished, it has zero effect on the live Dashboard
/// until an admin actually adds widgets and publishes it.
pub fn list_layouts(conn: &Connection, workspace_id: &str) -> AppResult<Vec<DashboardLayout>> {
    if dashboard_layout_repo::count_for_workspace(conn, workspace_id)? == 0 {
        let id = dashboard_layout_repo::new_id();
        let draft_json = serde_json::to_string(&empty_widgets()).expect("DashboardWidgets always serializes");
        dashboard_layout_repo::create(conn, &id, workspace_id, "Default", true, "[]", &draft_json, None, None)?;
    }
    dashboard_layout_repo::list(conn, workspace_id)?.into_iter().map(hydrate).collect()
}

pub fn get_layout(conn: &Connection, id: &str) -> AppResult<DashboardLayout> {
    let row = dashboard_layout_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Dashboard layout".into()))?;
    hydrate(row)
}

pub fn create_layout(conn: &Connection, workspace_id: &str, input: &DashboardLayoutInput, actor_user_id: Option<&str>) -> AppResult<DashboardLayout> {
    require_admin(conn, actor_user_id)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Layout name is required".into()));
    }
    require_valid_app_id(conn, workspace_id, input.app_id.as_deref())?;
    let is_default = dashboard_layout_repo::count_for_workspace(conn, workspace_id)? == 0;
    let id = dashboard_layout_repo::new_id();
    let draft_json = serde_json::to_string(&seeded_widgets(&input.initial_kpi_keys)).expect("DashboardWidgets always serializes");
    dashboard_layout_repo::create(conn, &id, workspace_id, input.name.trim(), is_default, "[]", &draft_json, input.app_id.as_deref(), actor_user_id)?;
    super::solution_component_service::tag_local(conn, workspace_id, "dashboard_layout", &id, actor_user_id)?;
    get_layout(conn, &id)
}

pub fn update_layout(conn: &Connection, id: &str, update: &DashboardLayoutUpdate, actor_user_id: Option<&str>) -> AppResult<DashboardLayout> {
    require_admin(conn, actor_user_id)?;
    if update.name.trim().is_empty() {
        return Err(AppError::Validation("Layout name is required".into()));
    }
    let existing = get_layout(conn, id)?;
    require_valid_app_id(conn, &existing.workspace_id, update.app_id.as_deref())?;
    let roles_json = serde_json::to_string(&update.roles).expect("Vec<String> always serializes");
    let draft_json = serde_json::to_string(&update.draft).expect("DashboardWidgets always serializes");
    dashboard_layout_repo::update_meta_and_draft(conn, id, update.name.trim(), &roles_json, &draft_json, update.app_id.as_deref(), actor_user_id)?;
    get_layout(conn, id)
}

pub fn publish_layout(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<DashboardLayout> {
    require_admin(conn, actor_user_id)?;
    dashboard_layout_repo::publish(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

pub fn unpublish_layout(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<DashboardLayout> {
    require_admin(conn, actor_user_id)?;
    dashboard_layout_repo::unpublish(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

pub fn revert_layout_draft(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<DashboardLayout> {
    require_admin(conn, actor_user_id)?;
    dashboard_layout_repo::revert_draft_to_published(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

/// Moves the Default flag onto `id` - see
/// `dashboard_layout_repo::clear_default`'s own comment on why this is
/// two sequential updates.
pub fn make_default(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<DashboardLayout> {
    require_admin(conn, actor_user_id)?;
    let layout = get_layout(conn, id)?;
    dashboard_layout_repo::clear_default(conn, &layout.workspace_id, id)?;
    dashboard_layout_repo::set_default(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

/// The default layout can never be deleted (it's the fallback every other
/// layout's role assignment depends on existing) - make a different
/// layout the default first. A workspace also always keeps at least one
/// layout once one exists, so the last remaining layout is always the
/// default and this second check is actually implied by the first, but
/// is checked explicitly too since "last layout" is the more intuitive
/// error for an admin to read - same reasoning as
/// `screen_layout_service::delete_layout`.
pub fn delete_layout(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let layout = get_layout(conn, id)?;
    if layout.is_default {
        return Err(AppError::Validation("The default layout can't be deleted - make a different layout the default first".into()));
    }
    let remaining = dashboard_layout_repo::count_for_workspace(conn, &layout.workspace_id)?;
    if remaining <= 1 {
        return Err(AppError::Validation("The last dashboard layout can't be deleted".into()));
    }
    dashboard_layout_repo::delete(conn, id)?;
    Ok(())
}

/// The widgets to render on the live Dashboard for the given actor, or
/// `None` if nothing published is in effect (the caller falls back to
/// the pre-this-feature Dashboard, driven by `dashboard_kpi_prefs`).
///
/// Resolution: any non-default, *published* layout whose roles include
/// one the actor holds wins, checked in `list_layouts`' own order
/// (default first, then alphabetical) - so if an actor's roles are
/// claimed by more than one layout, the alphabetically-first one wins,
/// same one reasonable tie-break `resolve_effective_layout` picks. No
/// match (or no roles - an unauthenticated caller) falls back to the
/// workspace's Default layout's published widgets.
pub fn resolve_effective_dashboard(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Option<DashboardWidgets>> {
    let actor_roles: Vec<String> = match actor_user_id {
        Some(uid) => user_repo::roles_for_user(conn, uid)?,
        None => Vec::new(),
    };
    let layouts = list_layouts(conn, workspace_id)?;
    let claimed = layouts
        .iter()
        .find(|l| !l.is_default && l.published.is_some() && l.roles.iter().any(|r| actor_roles.contains(r)));
    if let Some(l) = claimed {
        return Ok(l.published.clone());
    }
    Ok(layouts.into_iter().find(|l| l.is_default).and_then(|l| l.published))
}

/// Serialized alongside the resolved widgets for a Tauri/HTTP round trip -
/// see `screen_layout_service::EffectiveLayout`'s identical reasoning.
#[derive(Debug, Clone, Serialize)]
pub struct EffectiveDashboard {
    pub widgets: Option<DashboardWidgets>,
}
