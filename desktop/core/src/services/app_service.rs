//! App Builder Phase 1: group a set of already-existing objects, their
//! screens and a dashboard into one named, publishable application. Every
//! primitive an app assembles - Custom Objects, Screen/App Builder layouts,
//! Dashboards - already ships and works; this module is purely the
//! packaging/visibility layer on top, plus a genuinely new capability
//! nothing else in the product has: per-app access grants to a specific
//! *user*, not only a role (see migration 0025's own doc comment).
//!
//! Phase 2 (`require_object_write_access` below) closes the gap Phase 1's
//! doc comment used to describe here: an app's Viewer/Editor grant now
//! actually gates that object's own create/update/archive commands, not
//! just resolving a level for the frontend to read. Scope stays bounded to
//! those three verbs on the entity itself - a status-lifecycle command
//! with its own name (issue_invoice, convert_quote_to_order,
//! set_opportunity_products, ...) is unaffected, a later increment, not
//! silently promised here either.

use rusqlite::Connection;
use std::collections::HashMap;

use crate::domain::{AppError, AppResult};
use crate::models::app_definition::{
    AccessibleApp, AppDefinition, AppDefinitionInput, AppDefinitionUpdate, AppPermission, AppPermissionInput,
    APP_PERMISSION_LEVELS, APP_PERMISSION_PRINCIPAL_TYPES,
};
use crate::repositories::{app_definition_repo, dashboard_layout_repo, user_repo};
use crate::repositories::user_repo::ROLES;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn hydrate(row: (AppDefinition, String)) -> AppResult<AppDefinition> {
    let (mut app, object_keys_json) = row;
    app.object_keys = serde_json::from_str(&object_keys_json).unwrap_or_default();
    Ok(app)
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<AppDefinition>> {
    app_definition_repo::list(conn, workspace_id)?.into_iter().map(hydrate).collect()
}

pub fn get(conn: &Connection, id: &str) -> AppResult<AppDefinition> {
    let row = app_definition_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("App".into()))?;
    hydrate(row)
}

pub fn create(conn: &Connection, workspace_id: &str, input: &AppDefinitionInput, actor_user_id: Option<&str>) -> AppResult<AppDefinition> {
    require_admin(conn, actor_user_id)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("App name is required".into()));
    }
    let id = app_definition_repo::new_id();
    app_definition_repo::create(conn, &id, workspace_id, input.name.trim(), &input.icon, input.description.as_deref(), actor_user_id)?;
    get(conn, &id)
}

/// Every `object_keys` entry must be a built-in entity type or an active
/// custom object for this workspace (same validity check custom reports
/// and custom fields already use); `dashboard_id`, if set, must reference
/// an existing dashboard layout in the same workspace.
fn validate_object_keys_and_dashboard(
    conn: &Connection,
    workspace_id: &str,
    object_keys: &[String],
    dashboard_id: Option<&str>,
) -> AppResult<()> {
    for key in object_keys {
        if !super::custom_object_service::is_valid_dynamic_entity_type(conn, workspace_id, key)? {
            return Err(AppError::Validation(format!("'{key}' is not a valid object for this workspace")));
        }
    }
    if let Some(dash_id) = dashboard_id {
        let (dashboard, ..) = dashboard_layout_repo::get(conn, dash_id)?.ok_or_else(|| AppError::Validation("Dashboard not found".into()))?;
        if dashboard.workspace_id != workspace_id {
            return Err(AppError::Validation("Dashboard not found".into()));
        }
    }
    Ok(())
}

pub fn update(conn: &Connection, id: &str, update: &AppDefinitionUpdate, actor_user_id: Option<&str>) -> AppResult<AppDefinition> {
    require_admin(conn, actor_user_id)?;
    if update.name.trim().is_empty() {
        return Err(AppError::Validation("App name is required".into()));
    }
    let existing = get(conn, id)?;
    validate_object_keys_and_dashboard(conn, &existing.workspace_id, &update.object_keys, update.dashboard_id.as_deref())?;
    let object_keys_json = serde_json::to_string(&update.object_keys).expect("Vec<String> always serializes");
    app_definition_repo::update(
        conn, id, update.name.trim(), &update.icon, update.description.as_deref(), &object_keys_json, update.dashboard_id.as_deref(), actor_user_id,
    )?;
    get(conn, id)
}

pub fn publish(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<AppDefinition> {
    require_admin(conn, actor_user_id)?;
    let app = get(conn, id)?;
    if app.object_keys.is_empty() {
        return Err(AppError::Validation("Add at least one object before publishing this app".into()));
    }
    app_definition_repo::set_published(conn, id, true, actor_user_id)?;
    get(conn, id)
}

pub fn unpublish(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<AppDefinition> {
    require_admin(conn, actor_user_id)?;
    app_definition_repo::set_published(conn, id, false, actor_user_id)?;
    get(conn, id)
}

pub fn delete(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get(conn, id)?; // 404s cleanly instead of a silent no-op delete
    app_definition_repo::delete(conn, id)?;
    Ok(())
}

// ---- Permissions ------------------------------------------------------

pub fn list_permissions(conn: &Connection, app_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<AppPermission>> {
    require_admin(conn, actor_user_id)?;
    get(conn, app_id)?;
    Ok(app_definition_repo::list_permissions(conn, app_id)?)
}

pub fn grant_permission(conn: &Connection, app_id: &str, input: &AppPermissionInput, actor_user_id: Option<&str>) -> AppResult<AppPermission> {
    require_admin(conn, actor_user_id)?;
    let app = get(conn, app_id)?;
    if !APP_PERMISSION_PRINCIPAL_TYPES.contains(&input.principal_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid principal type '{}'", input.principal_type)));
    }
    if !APP_PERMISSION_LEVELS.contains(&input.level.as_str()) {
        return Err(AppError::Validation(format!("Invalid access level '{}'", input.level)));
    }
    match input.principal_type.as_str() {
        "role" => {
            if !ROLES.contains(&input.principal_id.as_str()) {
                return Err(AppError::Validation(format!("'{}' is not a role", input.principal_id)));
            }
        }
        "user" => {
            let user = user_repo::find_by_id(conn, &input.principal_id)?.ok_or_else(|| AppError::Validation("User not found".into()))?;
            if user.workspace_id != app.workspace_id {
                return Err(AppError::Validation("User not found".into()));
            }
        }
        _ => unreachable!("validated above"),
    }
    let id = app_definition_repo::new_id();
    app_definition_repo::upsert_permission(conn, &id, app_id, &input.principal_type, &input.principal_id, &input.level, actor_user_id)?;
    // The upsert may have updated an existing row under a different id than
    // the one just minted (a re-grant) - re-read by (app_id, principal) to
    // return the row that actually exists now.
    app_definition_repo::list_permissions(conn, app_id)?
        .into_iter()
        .find(|p| p.principal_type == input.principal_type && p.principal_id == input.principal_id)
        .ok_or_else(|| AppError::NotFound("App permission".into()))
}

pub fn revoke_permission(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    app_definition_repo::delete_permission(conn, id)?;
    Ok(())
}

/// The higher of two access levels - "editor" beats "viewer" when more
/// than one grant (e.g. two different role grants) applies to the same
/// actor on the same app.
fn stronger(a: &str, b: &str) -> String {
    if a == "editor" || b == "editor" { "editor".into() } else { "viewer".into() }
}

/// Every published app the actor can see, with their resolved access
/// level on each. An Administrator sees every published app at "editor" -
/// same "the workspace's own admins bypass everything else" precedent
/// `require_admin` already sets everywhere else in this codebase, not a
/// new exception invented for apps. Everyone else needs an explicit grant:
/// a user-specific grant wins outright over any role grant (more specific
/// intent), otherwise the strongest of whatever role grants match one of
/// the actor's roles; no matching grant at all means the app doesn't
/// appear for them, unpublished or not.
pub fn list_accessible(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<AccessibleApp>> {
    let published: Vec<AppDefinition> = list(conn, workspace_id)?.into_iter().filter(|a| a.is_published).collect();
    let Some(actor_id) = actor_user_id else { return Ok(Vec::new()) };
    let actor_roles = user_repo::roles_for_user(conn, actor_id)?;
    if actor_roles.iter().any(|r| r == "Administrator") {
        return Ok(published.into_iter().map(|app| AccessibleApp { app, level: "editor".into() }).collect());
    }

    let all_permissions = app_definition_repo::list_all_permissions_for_workspace(conn, workspace_id)?;
    let mut by_app: HashMap<&str, Vec<&AppPermission>> = HashMap::new();
    for p in &all_permissions {
        by_app.entry(p.app_id.as_str()).or_default().push(p);
    }

    let mut out = Vec::new();
    for app in published {
        let grants = by_app.get(app.id.as_str());
        let Some(grants) = grants else { continue };
        if let Some(user_grant) = grants.iter().find(|p| p.principal_type == "user" && p.principal_id == actor_id) {
            out.push(AccessibleApp { level: user_grant.level.clone(), app });
            continue;
        }
        let role_level = grants
            .iter()
            .filter(|p| p.principal_type == "role" && actor_roles.contains(&p.principal_id))
            .fold(None::<String>, |acc, p| Some(match acc { Some(a) => stronger(&a, &p.level), None => p.level.clone() }));
        if let Some(level) = role_level {
            out.push(AccessibleApp { level, app });
        }
    }
    Ok(out)
}

/// App Builder Phase 2: called at the top of an entity's own
/// create/update/archive service function (see each call site) to gate it
/// on the actor's App Builder access, the same way `require_admin` gates
/// admin-only operations elsewhere in this codebase.
///
/// Only engages once `entity_type` (the same capitalized-singular strings
/// `CUSTOM_FIELD_ENTITY_TYPES` uses for built-ins, or a custom object's own
/// key) is actually placed in a *published* app - an entity type in zero
/// published apps is unaffected, exactly the pre-App-Builder behavior. Once
/// it is, this reuses `list_accessible`'s exact resolution (Administrator
/// bypass, a user grant beating a role grant, the strongest of several
/// matching role grants) rather than a second copy of that logic, folding
/// across every published app that contains this entity type with
/// `stronger` the same way `list_accessible` folds multiple role grants on
/// one app. "Editor" on at least one such app is enough to write; "viewer"
/// everywhere it matters, or no access to any app that claims this entity
/// type at all, is not.
pub fn require_object_write_access(conn: &Connection, workspace_id: &str, entity_type: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let accessible = list_accessible(conn, workspace_id, actor_user_id)?;
    let level = accessible
        .iter()
        .filter(|a| a.app.object_keys.iter().any(|k| k == entity_type))
        .fold(None::<String>, |acc, a| Some(match acc { Some(l) => stronger(&l, &a.level), None => a.level.clone() }));
    match level.as_deref() {
        Some("editor") => Ok(()),
        Some(_) => Err(AppError::Validation(format!(
            "You have view-only access to {entity_type} records through an app - ask an Administrator for Editor access to make changes"
        ))),
        // No accessible app claims this entity type - either none does (the
        // common case, entirely unaffected by App Builder) or the actor
        // simply can't see any of the ones that do, which still needs
        // telling apart so an unscoped object stays silently unaffected.
        None => {
            let scoped = list(conn, workspace_id)?
                .into_iter()
                .any(|a| a.is_published && a.object_keys.iter().any(|k| k == entity_type));
            if scoped {
                Err(AppError::Validation(format!(
                    "You don't have access to {entity_type} records through any app - ask an Administrator for access"
                )))
            } else {
                Ok(())
            }
        }
    }
}
