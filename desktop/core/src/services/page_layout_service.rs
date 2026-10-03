//! Screen Builder 2.0 (issue #195, 5a): a page composer alongside the
//! Phase 1-3 `screen_layout_service` (which this mirrors function-for-
//! function for the draft/publish/revert/default/role lifecycle - see
//! that module's own doc comment and the migration's header comment for
//! why this is a second system, not an evolution of the first).
//!
//! The one genuinely new concern this layer owns that `screen_layout_service`
//! doesn't: validating a `PageDefinition` tree against a fixed component-type
//! registry (`COMPONENT_TYPES`/`CONTAINER_COMPONENT_TYPES`) before it's ever
//! saved, so a malformed or unknown `component_type` can't reach the
//! frontend's canvas/inspector/renderer - those all trust `component_type`
//! to be one of these without re-checking it themselves.

use rusqlite::Connection;
use serde::Serialize;

use crate::domain::{AppError, AppResult};
use crate::models::page_layout::{NodeLayout, PageDefinition, PageLayout, PageLayoutInput, PageLayoutUpdate, PageNode};
use crate::repositories::{page_layout_repo, user_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// The full component-type allowlist, grouped exactly as the issue's own
/// spec (§10.2) groups them. Honestly scoped for 5a: every Layout and
/// Record type the spec names is here (those are structural - any real
/// page needs them), plus a representative subset of Data and Actions
/// (the types with the clearest distinct value: a list, a table, a KPI
/// number, a chart; a plain button, a multi-button bar, a quick action,
/// and an agent action since "orchestrate agents" is this product's own
/// positioning) - Card List/Timeline/Activity Feed and Voice Action are
/// deliberately not in this list yet, named on the roadmap instead of
/// silently absent. The spec left Content/Navigation/Utility unitemized;
/// one representative, safe type covers each (rich text; an in-page
/// anchor link; a static admin-authored note) rather than guessing at a
/// longer list the spec never actually asked for.
pub const COMPONENT_TYPES: &[&str] = &[
    // Layout
    "section", "grid", "columns", "tabs", "divider", "spacer", "sticky_panel",
    // Record
    "field", "field_group", "record_header", "status_badge", "owner", "record_number",
    // Data
    "related_list", "table", "kpi", "chart",
    // Actions
    "button", "command_bar", "quick_action", "agent_action",
    // Content / Navigation / Utility
    "rich_text", "anchor_link", "note",
];

/// The subset of `COMPONENT_TYPES` allowed to have `children` - everything
/// else is a leaf. A leaf node with a non-empty `children` fails
/// validation rather than being silently flattened, so a bug writing bad
/// data is caught at save time, not discovered later as a rendering
/// mystery.
pub const CONTAINER_COMPONENT_TYPES: &[&str] = &["section", "grid", "columns", "tabs", "sticky_panel", "field_group"];

fn validate_node_layout(layout: &NodeLayout) -> AppResult<()> {
    for (label, span) in [
        ("column_span", Some(layout.column_span)),
        ("tablet_column_span", layout.tablet_column_span),
        ("mobile_column_span", layout.mobile_column_span),
    ] {
        if let Some(span) = span {
            if !(1..=12).contains(&span) {
                return Err(AppError::Validation(format!("{label} must be between 1 and 12, got {span}")));
            }
        }
    }
    Ok(())
}

fn validate_node(node: &PageNode) -> AppResult<()> {
    if !COMPONENT_TYPES.contains(&node.component_type.as_str()) {
        return Err(AppError::Validation(format!("Unknown component type '{}'", node.component_type)));
    }
    if !node.children.is_empty() && !CONTAINER_COMPONENT_TYPES.contains(&node.component_type.as_str()) {
        return Err(AppError::Validation(format!("'{}' can't contain other components", node.component_type)));
    }
    validate_node_layout(&node.layout)?;
    for child in &node.children {
        validate_node(child)?;
    }
    Ok(())
}

fn validate_page(page: &PageDefinition) -> AppResult<()> {
    for node in &page.root {
        validate_node(node)?;
    }
    Ok(())
}

fn empty_page() -> PageDefinition {
    PageDefinition { root: vec![] }
}

fn hydrate(row: (PageLayout, String, String, Option<String>)) -> AppResult<PageLayout> {
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

/// Every page layout for this entity type, default first - auto-provisions
/// a bare, unpublished Default layout (empty root) the first time this is
/// called for an entity type with none yet, same "always has a Default to
/// select" invariant `screen_layout_service::list_layouts` keeps. Being
/// unpublished and empty, it has zero effect on anything until an admin
/// actually composes and publishes it - and 5a has no runtime reader of
/// `published` yet regardless (that's 5b).
pub fn list_layouts(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<Vec<PageLayout>> {
    validate_entity_type(conn, workspace_id, entity_type)?;
    if page_layout_repo::count_for_entity(conn, workspace_id, entity_type)? == 0 {
        let id = page_layout_repo::new_id();
        let draft_json = serde_json::to_string(&empty_page()).expect("PageDefinition always serializes");
        page_layout_repo::create(conn, &id, workspace_id, entity_type, "Default", true, "[]", &draft_json, None)?;
    }
    page_layout_repo::list(conn, workspace_id, entity_type)?.into_iter().map(hydrate).collect()
}

pub fn get_layout(conn: &Connection, id: &str) -> AppResult<PageLayout> {
    let row = page_layout_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Page layout".into()))?;
    hydrate(row)
}

pub fn create_layout(conn: &Connection, workspace_id: &str, input: &PageLayoutInput, actor_user_id: Option<&str>) -> AppResult<PageLayout> {
    require_admin(conn, actor_user_id)?;
    validate_entity_type(conn, workspace_id, &input.entity_type)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Page name is required".into()));
    }
    let is_default = page_layout_repo::count_for_entity(conn, workspace_id, &input.entity_type)? == 0;
    let id = page_layout_repo::new_id();
    let draft_json = serde_json::to_string(&empty_page()).expect("PageDefinition always serializes");
    page_layout_repo::create(conn, &id, workspace_id, &input.entity_type, input.name.trim(), is_default, "[]", &draft_json, actor_user_id)?;
    super::solution_component_service::tag_local(conn, workspace_id, "page_layout", &id, actor_user_id)?;
    get_layout(conn, &id)
}

pub fn update_layout(conn: &Connection, id: &str, update: &PageLayoutUpdate, actor_user_id: Option<&str>) -> AppResult<PageLayout> {
    require_admin(conn, actor_user_id)?;
    if update.name.trim().is_empty() {
        return Err(AppError::Validation("Page name is required".into()));
    }
    validate_page(&update.draft)?;
    let roles_json = serde_json::to_string(&update.roles).expect("Vec<String> always serializes");
    let draft_json = serde_json::to_string(&update.draft).expect("PageDefinition always serializes");
    page_layout_repo::update_meta_and_draft(conn, id, update.name.trim(), &roles_json, &draft_json, actor_user_id)?;
    get_layout(conn, id)
}

pub fn publish_layout(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<PageLayout> {
    require_admin(conn, actor_user_id)?;
    page_layout_repo::publish(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

pub fn unpublish_layout(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<PageLayout> {
    require_admin(conn, actor_user_id)?;
    page_layout_repo::unpublish(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

pub fn revert_layout_draft(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<PageLayout> {
    require_admin(conn, actor_user_id)?;
    page_layout_repo::revert_draft_to_published(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

/// Moves the Default flag onto `id` - see
/// `page_layout_repo::clear_default`'s own comment on why this is two
/// sequential updates.
pub fn make_default(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<PageLayout> {
    require_admin(conn, actor_user_id)?;
    let layout = get_layout(conn, id)?;
    page_layout_repo::clear_default(conn, &layout.workspace_id, &layout.entity_type, id)?;
    page_layout_repo::set_default(conn, id, actor_user_id)?;
    get_layout(conn, id)
}

/// The default page layout can never be deleted, and a workspace always
/// keeps at least one once one exists - same reasoning (and same
/// redundant-but-clearer-error double check) as
/// `screen_layout_service::delete_layout`.
pub fn delete_layout(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let layout = get_layout(conn, id)?;
    if layout.is_default {
        return Err(AppError::Validation("The default page can't be deleted - make a different page the default first".into()));
    }
    let remaining = page_layout_repo::count_for_entity(conn, &layout.workspace_id, &layout.entity_type)?;
    if remaining <= 1 {
        return Err(AppError::Validation("The last page for an object can't be deleted".into()));
    }
    page_layout_repo::delete(conn, id)?;
    Ok(())
}

/// The published page to render for `entity_type` for the given actor, or
/// `None` if nothing's published - same role-resolution semantics as
/// `screen_layout_service::resolve_effective_layout`: a non-default
/// published layout whose roles intersect the actor's wins, else the
/// Default layout's published tree, else `None`. Exposed in 5a for the
/// builder's own "what would actually show" preview; nothing outside this
/// service calls it to render a live record yet - that wiring is 5b.
pub fn resolve_effective_page(conn: &Connection, workspace_id: &str, entity_type: &str, actor_user_id: Option<&str>) -> AppResult<Option<PageDefinition>> {
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

/// Serialized alongside the resolved page for a Tauri/HTTP round trip -
/// same rationale as `screen_layout_service::EffectiveLayout`.
#[derive(Debug, Clone, Serialize)]
pub struct EffectivePage {
    pub page: Option<PageDefinition>,
}
