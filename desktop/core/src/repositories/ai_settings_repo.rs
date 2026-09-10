//! Raw CRUD for `ai_settings` (migration 0035) - one row per workspace,
//! created lazily with defaults on first read, the same shape
//! `integration_settings_repo` already uses. See
//! `services::ai_service::get_settings`/`save_settings`.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::ai::AiSettings;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<AiSettings> {
    let secret_id: Option<String> = row.get("secret_id")?;
    Ok(AiSettings {
        workspace_id: row.get("workspace_id")?,
        provider: row.get("provider")?,
        base_url: row.get("base_url")?,
        model: row.get("model")?,
        has_key: secret_id.is_some(),
        status: row.get("status")?,
        last_test_message: row.get("last_test_message")?,
        last_tested_at: row.get("last_tested_at")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn get(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Option<AiSettings>> {
    conn.query_row("SELECT * FROM ai_settings WHERE workspace_id = ?1", [workspace_id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// The `secret_id` this workspace's settings row currently points at, if
/// any - `ai_service::save_settings` needs this raw value (not on the
/// public `AiSettings` model, which only ever exposes `has_key`) to decide
/// whether a new key rotates the existing secret in place or is a brand
/// new one.
pub fn get_secret_id(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT secret_id FROM ai_settings WHERE workspace_id = ?1", [workspace_id], |row| row.get(0))
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// Creates the default (unconfigured) row for a workspace that has never
/// touched AI settings before - called lazily rather than at
/// workspace-creation time, so every existing workspace picks up sane
/// defaults with no data migration.
pub fn ensure_default(conn: &Connection, workspace_id: &str) -> rusqlite::Result<AiSettings> {
    if let Some(existing) = get(conn, workspace_id)? {
        return Ok(existing);
    }
    conn.execute(
        "INSERT INTO ai_settings (workspace_id, updated_at) VALUES (?1, ?2)
         ON CONFLICT (workspace_id) DO NOTHING",
        rusqlite::params![workspace_id, now_iso()],
    )?;
    get(conn, workspace_id).map(|s| s.expect("just ensured"))
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    conn: &Connection,
    workspace_id: &str,
    provider: &str,
    base_url: Option<&str>,
    model: &str,
    secret_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<AiSettings> {
    conn.execute(
        "UPDATE ai_settings SET
            provider = ?1, base_url = ?2, model = ?3, secret_id = ?4,
            status = 'unconfigured', last_test_message = NULL, last_tested_at = NULL,
            updated_at = ?5, updated_by = ?6
         WHERE workspace_id = ?7",
        rusqlite::params![provider, base_url, model, secret_id, now_iso(), actor_user_id, workspace_id],
    )?;
    get(conn, workspace_id).map(|s| s.expect("just updated"))
}

/// Persists the outcome of `ai_service::test_key` - does not touch any of
/// the configuration columns above.
pub fn set_test_result(conn: &Connection, workspace_id: &str, status: &str, message: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ai_settings SET status = ?1, last_test_message = ?2, last_tested_at = ?3 WHERE workspace_id = ?4",
        rusqlite::params![status, message, now_iso(), workspace_id],
    )?;
    Ok(())
}
