//! UX/UI Modernization, Phase A (issue #191): raw CRUD + versioning for
//! `workspace_themes` (migration `0066_workspace_theme_studio.sql`). See
//! `services::theme_service` for preset definitions, contrast validation
//! and the publish-gate logic built on top of this.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::workspace_theme::{ThemeTokens, WorkspaceTheme, WorkspaceThemeInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<WorkspaceTheme> {
    let tokens_json: String = row.get("tokens_json")?;
    let tokens: ThemeTokens = serde_json::from_str(&tokens_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(WorkspaceTheme {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        status: row.get("status")?,
        version: row.get("version")?,
        preset_key: row.get("preset_key")?,
        tokens,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        published_at: row.get("published_at")?,
        published_by: row.get("published_by")?,
    })
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<WorkspaceTheme>> {
    conn.query_row("SELECT * FROM workspace_themes WHERE id = ?1", [id], map_row).optional()
}

pub fn get_published(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Option<WorkspaceTheme>> {
    conn.query_row(
        "SELECT * FROM workspace_themes WHERE workspace_id = ?1 AND status = 'published'",
        [workspace_id],
        map_row,
    )
    .optional()
}

pub fn list_versions(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<WorkspaceTheme>> {
    let mut stmt = conn.prepare("SELECT * FROM workspace_themes WHERE workspace_id = ?1 ORDER BY version DESC")?;
    let rows = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}

fn next_version(conn: &Connection, workspace_id: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM workspace_themes WHERE workspace_id = ?1",
        [workspace_id],
        |r| r.get(0),
    )
}

/// Always creates a new Draft version (never mutates a Published row) -
/// the same "a new draft, never edit history" rule agent versioning uses.
pub fn create_draft(
    conn: &Connection,
    workspace_id: &str,
    input: &WorkspaceThemeInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<WorkspaceTheme> {
    let id = new_uuid();
    let now = now_iso();
    let version = next_version(conn, workspace_id)?;
    let tokens_json = serde_json::to_string(&input.tokens).unwrap_or_default();
    conn.execute(
        "INSERT INTO workspace_themes (id, workspace_id, name, status, version, preset_key, tokens_json, created_at, created_by)
         VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, workspace_id, input.name, version, input.preset_key, tokens_json, now, actor_user_id],
    )?;
    Ok(get(conn, &id)?.expect("just inserted"))
}

/// Overwrites an existing Draft row's own tokens/name in place (editing a
/// Draft is a normal save, not a new version) - the caller is responsible
/// for confirming `status == "draft"` first.
pub fn update_draft(conn: &Connection, id: &str, input: &WorkspaceThemeInput) -> rusqlite::Result<()> {
    let tokens_json = serde_json::to_string(&input.tokens).unwrap_or_default();
    conn.execute(
        "UPDATE workspace_themes SET name = ?1, preset_key = ?2, tokens_json = ?3 WHERE id = ?4 AND status = 'draft'",
        rusqlite::params![input.name, input.preset_key, tokens_json, id],
    )?;
    Ok(())
}

/// Archives whichever row is currently published (if any) and marks
/// `id` published - mirrors agent_version_service::transition_status's
/// own "deprecate previous published, then set this one published"
/// sequential-statement shape (this codebase's established convention;
/// the single-writer desktop connection makes an explicit transaction
/// unnecessary here same as everywhere else).
pub fn publish(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<WorkspaceTheme> {
    let now = now_iso();
    let workspace_id: String = conn.query_row("SELECT workspace_id FROM workspace_themes WHERE id = ?1", [id], |r| r.get(0))?;
    conn.execute(
        "UPDATE workspace_themes SET status = 'archived' WHERE workspace_id = ?1 AND status = 'published'",
        [&workspace_id],
    )?;
    conn.execute(
        "UPDATE workspace_themes SET status = 'published', published_at = ?1, published_by = ?2 WHERE id = ?3",
        rusqlite::params![now, actor_user_id, id],
    )?;
    Ok(get(conn, id)?.expect("just published"))
}

pub fn delete_draft(conn: &Connection, id: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM workspace_themes WHERE id = ?1 AND status = 'draft'", [id])
}
