//! Raw CRUD for `integration_settings` (migration 0032) - one row per
//! workspace, created lazily with defaults on first read. See
//! `services::integration_log_service::get_settings`/`update_settings`.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::integration::IntegrationSettings;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<IntegrationSettings> {
    let allow_insecure: i64 = row.get("allow_insecure_connections")?;
    Ok(IntegrationSettings {
        workspace_id: row.get("workspace_id")?,
        api_rate_limit_per_minute: row.get("api_rate_limit_per_minute")?,
        global_rate_limit_per_minute: row.get("global_rate_limit_per_minute")?,
        log_retention_days: row.get("log_retention_days")?,
        file_retention_days: row.get("file_retention_days")?,
        allow_insecure_connections: allow_insecure != 0,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn get(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Option<IntegrationSettings>> {
    conn.query_row("SELECT * FROM integration_settings WHERE workspace_id = ?1", [workspace_id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// Creates the default row for a workspace that has never touched
/// Integration Hub settings before - called lazily rather than at
/// workspace-creation time, so every existing workspace picks up sane
/// defaults without a data migration.
pub fn ensure_default(conn: &Connection, workspace_id: &str) -> rusqlite::Result<IntegrationSettings> {
    if let Some(existing) = get(conn, workspace_id)? {
        return Ok(existing);
    }
    conn.execute(
        "INSERT INTO integration_settings (workspace_id, updated_at) VALUES (?1, ?2)
         ON CONFLICT (workspace_id) DO NOTHING",
        rusqlite::params![workspace_id, now_iso()],
    )?;
    get(conn, workspace_id).map(|s| s.expect("just ensured"))
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    conn: &Connection,
    workspace_id: &str,
    api_rate_limit_per_minute: i64,
    global_rate_limit_per_minute: i64,
    log_retention_days: i64,
    file_retention_days: i64,
    allow_insecure_connections: bool,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<IntegrationSettings> {
    conn.execute(
        "UPDATE integration_settings SET
            api_rate_limit_per_minute = ?1, global_rate_limit_per_minute = ?2,
            log_retention_days = ?3, file_retention_days = ?4, allow_insecure_connections = ?5,
            updated_at = ?6, updated_by = ?7
         WHERE workspace_id = ?8",
        rusqlite::params![
            api_rate_limit_per_minute,
            global_rate_limit_per_minute,
            log_retention_days,
            file_retention_days,
            allow_insecure_connections as i64,
            now_iso(),
            actor_user_id,
            workspace_id,
        ],
    )?;
    get(conn, workspace_id).map(|s| s.expect("just updated"))
}
