//! Saved Views & Bulk Actions (product backlog): a saved view is a
//! persisted filter/sort/column/grouping preference for one `object_key`
//! - see migration 0034's own comment for why `filters`/`columns` are
//! plain JSON rather than a modeled query.
//!
//! Scope is deliberately broader than Bulk Actions (`bulk_action_service`):
//! any object `api_object_service::list_object_keys` knows about (every
//! readable built-in plus every active Custom Object) can have saved
//! views, since a view is a read/browse concern, not a write one -
//! `object_key` isn't otherwise validated beyond that, matching how a
//! Custom Object can be deactivated and reactivated without losing what
//! was built against its key elsewhere in this codebase.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::saved_view::{SavedView, SavedViewInput};
use crate::repositories::saved_view_repo;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn require_actor<'a>(actor_user_id: Option<&'a str>) -> AppResult<&'a str> {
    actor_user_id.ok_or_else(|| AppError::Validation("A signed-in user is required".into()))
}

fn validate_input(input: &SavedViewInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("View name is required".into()));
    }
    if input.object_key.trim().is_empty() {
        return Err(AppError::Validation("An object is required".into()));
    }
    if input.visibility != "private" && input.visibility != "shared" {
        return Err(AppError::Validation("Visibility must be 'private' or 'shared'".into()));
    }
    if input.sort_direction != "asc" && input.sort_direction != "desc" {
        return Err(AppError::Validation("Sort direction must be 'asc' or 'desc'".into()));
    }
    Ok(())
}

/// Only the owner or an admin may edit/delete/set-default a view - a
/// `shared` visibility makes it usable by everyone, not editable by
/// everyone, the same "anyone can read, one owner (or an admin) can
/// write" line every other admin-authored resource in this codebase
/// draws (Business Rules, Workflow Automation, Dashboards, ...).
fn require_owner_or_admin(conn: &Connection, view: &SavedView, actor_user_id: Option<&str>) -> AppResult<()> {
    if Some(view.owner_user_id.as_str()) == actor_user_id {
        return Ok(());
    }
    require_admin(conn, actor_user_id)
}

pub fn create(conn: &Connection, workspace_id: &str, input: &SavedViewInput, actor_user_id: Option<&str>) -> AppResult<SavedView> {
    validate_input(input)?;
    let owner = require_actor(actor_user_id)?;
    Ok(saved_view_repo::insert(
        conn,
        &new_uuid(),
        workspace_id,
        input.object_key.trim(),
        input.name.trim(),
        owner,
        &input.visibility,
        &input.filters,
        input.sort_field.as_deref(),
        &input.sort_direction,
        input.columns.as_deref(),
        input.group_by_field.as_deref(),
    )?)
}

/// Every view `actor_user_id` may use for `object_key` - their own
/// private views plus every shared view in the workspace, with the
/// workspace's current default (if any) listed first regardless of who
/// owns it, matching `saved_view_repo::list_usable`'s own ordering.
pub fn list_for_object(conn: &Connection, workspace_id: &str, object_key: &str, actor_user_id: Option<&str>) -> AppResult<Vec<SavedView>> {
    let user = require_actor(actor_user_id)?;
    Ok(saved_view_repo::list_usable(conn, workspace_id, object_key, user)?)
}

fn get_owned(conn: &Connection, id: &str) -> AppResult<SavedView> {
    saved_view_repo::get_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("Saved view".into()))
}

/// Fetches a saved view by id for use as a dashboard record-list widget's
/// data source - deliberately no owner/visibility check here: an admin who
/// wired a widget to this view already chose it (same trust boundary
/// `dashboard_layout_service` itself operates under for the rest of a
/// widget's config), and only the view's `filters` ever leave this call,
/// never the view row itself.
pub fn get(conn: &Connection, id: &str) -> AppResult<Option<SavedView>> {
    Ok(saved_view_repo::get_by_id(conn, id)?)
}

pub fn update(conn: &Connection, id: &str, input: &SavedViewInput, actor_user_id: Option<&str>) -> AppResult<SavedView> {
    validate_input(input)?;
    let existing = get_owned(conn, id)?;
    require_owner_or_admin(conn, &existing, actor_user_id)?;
    Ok(saved_view_repo::update(
        conn,
        id,
        input.name.trim(),
        &input.visibility,
        &input.filters,
        input.sort_field.as_deref(),
        &input.sort_direction,
        input.columns.as_deref(),
        input.group_by_field.as_deref(),
    )?)
}

pub fn delete(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let existing = get_owned(conn, id)?;
    require_owner_or_admin(conn, &existing, actor_user_id)?;
    Ok(saved_view_repo::delete(conn, id)?)
}

/// Sets `id` as the object-wide default every user lands on when they
/// first open this object's list with no view of their own selected yet
/// - admin-only (a default is a workspace-wide policy choice, not a
/// personal preference), and unsets whatever was previously the default
/// for the same `object_key` first, so at most one row ever has
/// `is_object_default = 1` per (workspace_id, object_key) - the
/// invariant migration 0034's own comment documents as enforced here
/// rather than by a schema constraint.
pub fn set_default(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<SavedView> {
    require_admin(conn, actor_user_id)?;
    let view = get_owned(conn, id)?;
    saved_view_repo::clear_default_for_object(conn, &view.workspace_id, &view.object_key)?;
    saved_view_repo::set_default(conn, id)?;
    get_owned(conn, id)
}

pub fn clear_default(conn: &Connection, workspace_id: &str, object_key: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    Ok(saved_view_repo::clear_default_for_object(conn, workspace_id, object_key)?)
}

pub fn get_default(conn: &Connection, workspace_id: &str, object_key: &str) -> AppResult<Option<SavedView>> {
    Ok(saved_view_repo::find_default(conn, workspace_id, object_key)?)
}
