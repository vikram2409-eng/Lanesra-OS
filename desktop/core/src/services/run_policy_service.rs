//! WP-0141: the interrupted-run policy (ADR-0007 §5, ADR-0013/ADR-0014,
//! owner decision 2026-10-16).
//!
//! When the runtime finds a run that was left `running` with an expired
//! worker lease - the process that held it stopped mid-step - this setting
//! decides what happens: `resume_if_safe` (the default) re-takes the run and
//! continues it, safe because every write is idempotent (WP-0141's ledger);
//! `always_pause` leaves it paused for a person to look at. The setting is
//! per workspace; only an administrator may change it, and every change is
//! audited. (A per-Team override arrives with Teams in Phase 3.)

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::repositories::audit_repo;

pub const RESUME_IF_SAFE: &str = "resume_if_safe";
pub const ALWAYS_PAUSE: &str = "always_pause";
const SETTING_KEY: &str = "interrupted_run_policy";

/// The workspace's interrupted-run policy, defaulting to `resume_if_safe`.
pub fn get(conn: &Connection, workspace_id: &str) -> AppResult<String> {
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE workspace_id = ?1 AND key = ?2",
            rusqlite::params![workspace_id, SETTING_KEY],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value
        .filter(|v| is_valid(v))
        .unwrap_or_else(|| RESUME_IF_SAFE.to_string()))
}

/// Sets the policy. Administrator only; the value must be one of the two
/// known policies; the change is audited (ADR-0007 §5).
pub fn set(
    conn: &Connection,
    workspace_id: &str,
    policy: &str,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)?;
    if !is_valid(policy) {
        return Err(AppError::Validation(format!(
            "interrupted_run_policy must be '{RESUME_IF_SAFE}' or '{ALWAYS_PAUSE}', not '{policy}'"
        )));
    }
    conn.execute(
        "INSERT INTO settings (id, workspace_id, key, value) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (workspace_id, key) DO UPDATE SET value = excluded.value",
        rusqlite::params![new_uuid(), workspace_id, SETTING_KEY, policy],
    )?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "run_policy",
        Some("workspace"),
        Some(workspace_id),
        &format!("Interrupted-run policy set to '{policy}'"),
        None,
    )?;
    Ok(())
}

fn is_valid(policy: &str) -> bool {
    policy == RESUME_IF_SAFE || policy == ALWAYS_PAUSE
}
