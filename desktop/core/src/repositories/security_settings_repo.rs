//! Raw CRUD for `security_settings` (migration 0089) — one row per
//! workspace, created lazily with defaults on first read, like
//! `integration_settings_repo`. See `services::security_settings_service`.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::security::SecuritySettings;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<SecuritySettings> {
    Ok(SecuritySettings {
        workspace_id: row.get("workspace_id")?,
        lockout_max_failures: row.get("lockout_max_failures")?,
        lockout_minutes: row.get("lockout_minutes")?,
        session_lifetime_hours: row.get("session_lifetime_hours")?,
        session_idle_minutes: row.get("session_idle_minutes")?,
        public_base_url: row.get("public_base_url")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn get(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Option<SecuritySettings>> {
    conn.query_row(
        "SELECT * FROM security_settings WHERE workspace_id = ?1",
        [workspace_id],
        map_row,
    )
    .map(Some)
    .or_else(|e| {
        if e == rusqlite::Error::QueryReturnedNoRows {
            Ok(None)
        } else {
            Err(e)
        }
    })
}

/// Creates the default row for a workspace that has never touched security
/// settings — called lazily, so existing workspaces pick up the defaults
/// (migration 0089) with no data migration.
pub fn ensure_default(conn: &Connection, workspace_id: &str) -> rusqlite::Result<SecuritySettings> {
    if let Some(existing) = get(conn, workspace_id)? {
        return Ok(existing);
    }
    conn.execute(
        "INSERT INTO security_settings (workspace_id, updated_at) VALUES (?1, ?2)
         ON CONFLICT (workspace_id) DO NOTHING",
        rusqlite::params![workspace_id, now_iso()],
    )?;
    get(conn, workspace_id).map(|s| s.expect("just ensured"))
}

pub fn update(
    conn: &Connection,
    workspace_id: &str,
    lockout_max_failures: i64,
    lockout_minutes: i64,
    session_lifetime_hours: i64,
    session_idle_minutes: i64,
    public_base_url: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<SecuritySettings> {
    conn.execute(
        "UPDATE security_settings SET
            lockout_max_failures = ?1, lockout_minutes = ?2,
            session_lifetime_hours = ?3, session_idle_minutes = ?4,
            public_base_url = ?5, updated_at = ?6, updated_by = ?7
         WHERE workspace_id = ?8",
        rusqlite::params![
            lockout_max_failures,
            lockout_minutes,
            session_lifetime_hours,
            session_idle_minutes,
            public_base_url,
            now_iso(),
            actor_user_id,
            workspace_id,
        ],
    )?;
    get(conn, workspace_id).map(|s| s.expect("just updated"))
}
