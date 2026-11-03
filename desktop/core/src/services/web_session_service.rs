//! WP-0106: admin view and control of web sessions — list the active
//! sessions and end them all ("sign out all sessions"). Session lifetime
//! and idle timeout are enforced in `session_repo`; this is the admin
//! surface over them.

use rusqlite::Connection;

use crate::domain::AppResult;
use crate::models::security::ActiveSession;
use crate::repositories::{audit_repo, session_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// Lists the workspace's active sessions, newest-seen first. `current_id`
/// (the requesting session's id) is flagged so the UI can label "this
/// device".
pub fn list_active_sessions(
    conn: &Connection,
    workspace_id: &str,
    actor_user_id: Option<&str>,
    current_id: Option<&str>,
) -> AppResult<Vec<ActiveSession>> {
    require_admin(conn, actor_user_id)?;
    let rows = session_repo::list_active(conn, workspace_id)?;
    Ok(rows
        .into_iter()
        .map(
            |(id, user_id, username, display_name, created_at, last_seen_at, expires_at)| {
                let current = current_id == Some(id.as_str());
                ActiveSession {
                    id,
                    user_id,
                    username,
                    display_name,
                    created_at,
                    last_seen_at,
                    expires_at,
                    current,
                }
            },
        )
        .collect())
}

/// Ends every session in the workspace, including the caller's. Returns the
/// number ended.
pub fn sign_out_all(
    conn: &Connection,
    workspace_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<usize> {
    require_admin(conn, actor_user_id)?;
    let ended = session_repo::delete_all_for_workspace(conn, workspace_id)?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "sessions_signed_out_all",
        Some("session"),
        None,
        &format!("Signed out all sessions ({ended} ended)"),
        None,
    )?;
    Ok(ended)
}
