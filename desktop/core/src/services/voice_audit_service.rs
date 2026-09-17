//! Voice-First Mode, PR 1: read-side access to Voice's own audit trail
//! (spec §21) - "My Voice Activity" (any user, their own commands) and the
//! admin Voice Activity search (any workspace command). Mirrors
//! `audit_service.rs`'s own "thin read-side, real repo underneath" shape:
//! no new enforcement logic here, `voice_repo`'s SQL already scopes by
//! user_id/workspace_id.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::voice::VoiceActivityEntry;
use crate::repositories::{user_repo, voice_repo};

pub fn list_my_voice_activity(conn: &Connection, user_id: &str, limit: i64) -> AppResult<Vec<VoiceActivityEntry>> {
    Ok(voice_repo::list_activity_for_user(conn, user_id, limit)?)
}

/// Same Administrator-only gate every other admin-only read in this
/// codebase defines for itself - Voice Activity across every user in the
/// workspace is a security/audit surface, not something any authenticated
/// user should be able to browse.
fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can search Voice Activity".into()));
    }
    Ok(())
}

pub fn search_voice_activity(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>, limit: i64) -> AppResult<Vec<VoiceActivityEntry>> {
    require_admin(conn, actor_user_id)?;
    Ok(voice_repo::search_activity_for_workspace(conn, workspace_id, limit)?)
}
