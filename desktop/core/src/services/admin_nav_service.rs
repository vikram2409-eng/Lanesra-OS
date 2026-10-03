//! Admin Control Center Modernization (issue #197): a thin read/write
//! wrapper over `admin_nav_repo` - recording a visit is fire-and-forget
//! bookkeeping (gated only on "a real, authenticated user", the same
//! light bar `audit_service::list_for_entity` itself uses), and every
//! list is scoped to the caller's own history - there's no "whose
//! Recently Viewed" admin concept, unlike the workspace-wide Recent
//! Changes feed `audit_service::list_recent` exposes.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::admin_nav::AdminNavItem;
use crate::repositories::admin_nav_repo;

fn require_user(actor_user_id: Option<&str>) -> AppResult<&str> {
    actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))
}

pub fn record_visit(conn: &Connection, workspace_id: &str, admin_tab: &str, label: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let user_id = require_user(actor_user_id)?;
    Ok(admin_nav_repo::record_visit(conn, workspace_id, user_id, admin_tab, label)?)
}

pub fn set_pinned(conn: &Connection, workspace_id: &str, admin_tab: &str, pinned: bool, actor_user_id: Option<&str>) -> AppResult<()> {
    let user_id = require_user(actor_user_id)?;
    Ok(admin_nav_repo::set_pinned(conn, workspace_id, user_id, admin_tab, pinned)?)
}

pub fn list_recent(conn: &Connection, workspace_id: &str, limit: i64, actor_user_id: Option<&str>) -> AppResult<Vec<AdminNavItem>> {
    let user_id = require_user(actor_user_id)?;
    Ok(admin_nav_repo::list_recent(conn, workspace_id, user_id, limit.clamp(1, 50))?)
}

pub fn list_pinned(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<AdminNavItem>> {
    let user_id = require_user(actor_user_id)?;
    Ok(admin_nav_repo::list_pinned(conn, workspace_id, user_id)?)
}
