//! Screen/App Builder Phase 1: multiple named layouts per object (built-in
//! or custom), each with its own tabs of drag-ordered field sections, and
//! role-based assignment with a required default fallback. See the
//! migration's header comment for why the tabs structure is stored (and
//! handled here) as opaque field-key strings rather than coupled to any
//! specific field registry - this service never needs to know what a
//! field is, only where an admin put it and who should see it.
//!
//! Mirrors the online demo's equivalent (app.js's layoutsTab/ensureLayouts
//! and friends), with one real difference: the demo has no signed-in
//! user, so its live form always renders the Default layout; here,
//! `resolve_effective_layout` resolves against the real signed-in user's
//! roles.

use rusqlite::Connection;
use serde::Serialize;

use crate::domain::{AppError, AppResult};
use crate::models::screen_layout::{LayoutTab, LayoutTabs, ScreenLayout, ScreenLayoutInput, ScreenLayoutUpdate};
use crate::repositories::{screen_layout_repo, user_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn empty_tabs() -> LayoutTabs {
    LayoutTabs { tabs: vec![LayoutTab { id: crate::domain::ids::new_uuid(), title: "Details".into(), sections: vec![] }] }
}

fn seeded_tabs(initial_fields: &[String]) -> LayoutTabs {
    use crate::models::screen_layout::LayoutSection;
    LayoutTabs {
        tabs: vec![LayoutTab {
            id: crate::domain::ids::new_uuid(),
            title: "Details".into(),
            sections: vec![LayoutSection {
                id: crate::domain::ids::new_uuid(),
                title: "Details".into(),
                fields: initial_fields.to_vec(),
            }],
        }],
    }
}

fn hydrate(row: (ScreenLayout, String, String, Option<String>)) -> AppResult<ScreenLayout> {
    let (mut layout, roles_json, draft_json, published_json) = row;
    layout.roles = serde_json::from_str(&roles_json).unwrap_or_default();
    layout.draft = serde_json::from_str(&draft_json).unwrap_or_default();
    layout.published = published_json.and_then(|s| serde_json::from_str(&s).ok());
    Ok(layout)
}

fn validate_entity_type(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<()> {
    if !super::custom_object_service::is_valid_dynamic_entity_type(conn, workspace_id, entity_type)? {
        return Err(AppError::Validation(format!("Invalid entity type '{entity_type}'")));
    }
    Ok(())
}

/// Every layout for this entity type, default first - auto-provisions a
/// bare, unpublished Default layout (one empty tab/section) the first
/// time this is called for an entity type with none yet, the same
/// "always has a Default to select" invariant the demo's `ensureLayouts`
/// keeps. Being unpublished, it has zero effect on the live form until
/// an admin actually adds fields and publishes it.
pub fn list_layouts(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<Vec<ScreenLayout>> {
    validate_entity_type(conn, workspace_id, entity_type)?;
    if screen_layout_repo::count_for_entity(conn, workspace_id, entity_type)? == 0 {
        let id = screen_layout_repo::new_id();
        let draft_json = serde_json::to_string(&empty_tabs()).expect("LayoutTabs always serializes");
        screen_layout_repo::create(conn, &id, workspace_id, entity_type, "Default", true, "[]", &draft_json, None)?;
    }
    screen_layout_repo::list(conn, workspace_id, entity_type)?.into_iter().map(hydrate).collect()
}

pub fn get_layout(conn: &Connection, id: &str) -> AppResult<ScreenLayout> {
    let row = screen_layout_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Screen layout".into()))?;
    hydrate(row)
}

pub fn create_layout(conn: &Connection, workspace_id: &str, input: &ScreenLayoutInput, actor_user_id: Option<&str>) -> AppResult<ScreenLayout> {
    require_admin(conn, actor_user_id)?;
    validate_entity_type(conn, workspace_id, &input.entity_type)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Layout name is required".into()));
    }
    let is_default = screen_layout_repo::count_for_entity(conn, workspace_id, &input.entity_type)? == 0;
    let id = screen_layout_repo::new_id();
    let draft_json = serde_json::to_string(&seeded_tabs(&input.initial_fields)).expect("LayoutTabs always serializes");
    screen_layout_repo::create(conn, &id, workspace_id, &input.entity_type, input.name.trim(), is_default, "[]", &draft_json, actor_user_id)?;
    get_layout(conn, &id)
}

pub fn update_layout(conn: &Connection, id: &str, update: &ScreenLayoutUpdate, actor_user_id: Option<&str>) -> AppResult<ScreenLayout> {
    require_admin(conn, actor_user_id)?;
    if update.name.trim().is_empty() {
        return Err(AppError::Validation("Layout name is required".into()));
    }
    let roles_json = serde_json::to_string(&update.roles).expect("Vec<String> always serializes");
    let draft_json = serde_json::to_string(&update.draft).expect("LayoutTabs always serializes");
    screen_layout_repo::update_meta_and_draft(conn, id, update.name.trim(), &roles_json, &draft_json, actor_user_id)?;
    get_layout(conn, id)
}

pub fn publish_layout(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<ScreenLayout> {
    require_admin(conn, actor_user_id)?;
    screen_layout_repo::publish(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

pub fn unpublish_layout(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<ScreenLayout> {
    require_admin(conn, actor_user_id)?;
    screen_layout_repo::unpublish(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

pub fn revert_layout_draft(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<ScreenLayout> {
    require_admin(conn, actor_user_id)?;
    screen_layout_repo::revert_draft_to_published(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

/// Moves the Default flag onto `id` (any layout can become the default,
/// not just the one auto-provisioned first) - see
/// `screen_layout_repo::clear_default`'s own comment on why this is two
/// sequential updates.
pub fn make_default(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<ScreenLayout> {
    require_admin(conn, actor_user_id)?;
    let layout = get_layout(conn, id)?;
    screen_layout_repo::clear_default(conn, &layout.workspace_id, &layout.entity_type, id)?;
    screen_layout_repo::set_default(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

/// The default layout can never be deleted (it's the fallback every
/// other layout's role assignment depends on existing) - make a
/// different layout the default first. A workspace also always keeps at
/// least one layout per entity type once one exists, so the last
/// remaining layout is always the default and this second check is
/// actually implied by the first, but is checked explicitly too since
/// "last layout" is the more intuitive error for an admin to read.
pub fn delete_layout(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let layout = get_layout(conn, id)?;
    if layout.is_default {
        return Err(AppError::Validation("The default layout can't be deleted - make a different layout the default first".into()));
    }
    let remaining = screen_layout_repo::count_for_entity(conn, &layout.workspace_id, &layout.entity_type)?;
    if remaining <= 1 {
        return Err(AppError::Validation("The last layout for an object can't be deleted".into()));
    }
    screen_layout_repo::delete(conn, id)?;
    Ok(())
}

/// The tabs to render on this entity type's live create/edit form for the
/// given actor, or `None` if nothing published is in effect (renders the
/// plain default field order, exactly as if this feature didn't exist).
///
/// Resolution: any non-default, *published* layout whose roles include
/// one the actor holds wins, checked in `list_layouts`' own order
/// (default first, then alphabetical) - so if an actor's roles are
/// claimed by more than one layout, the alphabetically-first one wins.
/// That's one reasonable tie-break, not the only one; a workspace this
/// ambiguity actually bites should just not assign the same role to two
/// layouts on one object. No match (or no roles - an unauthenticated
/// caller) falls back to the entity's Default layout's published tabs.
pub fn resolve_effective_layout(conn: &Connection, workspace_id: &str, entity_type: &str, actor_user_id: Option<&str>) -> AppResult<Option<LayoutTabs>> {
    let actor_roles: Vec<String> = match actor_user_id {
        Some(uid) => user_repo::roles_for_user(conn, uid)?,
        None => Vec::new(),
    };
    let layouts = list_layouts(conn, workspace_id, entity_type)?;
    let claimed = layouts
        .iter()
        .find(|l| !l.is_default && l.published.is_some() && l.roles.iter().any(|r| actor_roles.contains(r)));
    if let Some(l) = claimed {
        return Ok(l.published.clone());
    }
    Ok(layouts.into_iter().find(|l| l.is_default).and_then(|l| l.published))
}

/// Serialized alongside the resolved tabs for a Tauri/HTTP round trip -
/// `LayoutTabs` alone already derives `Serialize`, this exists only so
/// callers get a named, documented response shape instead of a bare
/// nullable struct.
#[derive(Debug, Clone, Serialize)]
pub struct EffectiveLayout {
    pub tabs: Option<LayoutTabs>,
}
