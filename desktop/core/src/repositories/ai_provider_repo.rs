//! Raw CRUD for `ai_providers` (migration 0041) - named AI provider
//! connections, the multi-row sibling of the single `ai_settings` row.
//! See `services::ai_provider_service` for admin-gated validation and
//! `services::ai_gateway_service` for how an agent's routing policy
//! actually resolves one of these into a real dispatch.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::ai::{AiProvider, AiProviderInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<AiProvider> {
    let secret_id: Option<String> = row.get("secret_id")?;
    Ok(AiProvider {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        provider: row.get("provider")?,
        base_url: row.get("base_url")?,
        model: row.get("model")?,
        has_key: secret_id.is_some(),
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<AiProvider>> {
    conn.query_row("SELECT * FROM ai_providers WHERE id = ?1", [id], map_row).optional()
}

/// The raw `secret_id`, not exposed on the public model - same reason
/// `ai_settings_repo::get_secret_id` exists (deciding rotate-vs-insert,
/// and resolving the real key for a dispatch).
pub fn get_secret_id(conn: &Connection, id: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT secret_id FROM ai_providers WHERE id = ?1", [id], |row| row.get(0)).optional().map(|o| o.flatten())
}

pub fn list(conn: &Connection, workspace_id: &str, active_only: bool) -> rusqlite::Result<Vec<AiProvider>> {
    let sql = if active_only {
        "SELECT * FROM ai_providers WHERE workspace_id = ?1 AND is_active = 1 ORDER BY name"
    } else {
        "SELECT * FROM ai_providers WHERE workspace_id = ?1 ORDER BY name"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &AiProviderInput, secret_id: Option<&str>, actor_user_id: Option<&str>) -> rusqlite::Result<AiProvider> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_providers (id, workspace_id, name, provider, base_url, model, secret_id, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9, ?8, ?9)",
        rusqlite::params![id, workspace_id, input.name, input.provider, input.base_url, input.model, secret_id, now, actor_user_id],
    )?;
    get(conn, id).map(|p| p.expect("just inserted"))
}

pub fn update(conn: &Connection, id: &str, input: &AiProviderInput, secret_id: Option<&str>, actor_user_id: Option<&str>) -> rusqlite::Result<AiProvider> {
    let now = now_iso();
    conn.execute(
        "UPDATE ai_providers SET name = ?1, provider = ?2, base_url = ?3, model = ?4, secret_id = ?5, updated_at = ?6, updated_by = ?7 WHERE id = ?8",
        rusqlite::params![input.name, input.provider, input.base_url, input.model, secret_id, now, actor_user_id, id],
    )?;
    get(conn, id).map(|p| p.expect("just updated"))
}

pub fn set_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> rusqlite::Result<AiProvider> {
    let now = now_iso();
    conn.execute("UPDATE ai_providers SET is_active = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4", rusqlite::params![is_active, now, actor_user_id, id])?;
    get(conn, id).map(|p| p.expect("just updated"))
}
