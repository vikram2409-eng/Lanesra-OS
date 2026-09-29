//! AI Agent Platform v2, Phase 2: raw CRUD for `ai_tool_registry`
//! (migration `0061_agent_policy_engine.sql`) - workspace-scoped risk-level
//! overrides. See `services::tool_registry_service` for the built-in
//! default classification a missing row falls back to.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::ai_tool_registry::{AiToolRegistryOverride, AiToolRegistryOverrideInput, RiskLevel};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<AiToolRegistryOverride> {
    let risk_level: String = row.get("risk_level")?;
    Ok(AiToolRegistryOverride {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        tool_name: row.get("tool_name")?,
        risk_level: RiskLevel::from_str(&risk_level).unwrap_or(RiskLevel::Write),
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

/// Most-recently-updated first.
pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<AiToolRegistryOverride>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_tool_registry WHERE workspace_id = ?1 ORDER BY updated_at DESC")?;
    let rows = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}

pub fn get_by_tool_name(conn: &Connection, workspace_id: &str, tool_name: &str) -> rusqlite::Result<Option<AiToolRegistryOverride>> {
    conn.query_row(
        "SELECT * FROM ai_tool_registry WHERE workspace_id = ?1 AND tool_name = ?2",
        rusqlite::params![workspace_id, tool_name],
        map_row,
    )
    .optional()
}

/// Upsert-by-`(workspace_id, tool_name)`, matching the table's own
/// `UNIQUE` constraint.
pub fn set(conn: &Connection, workspace_id: &str, input: &AiToolRegistryOverrideInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiToolRegistryOverride> {
    let now = now_iso();
    let existing = get_by_tool_name(conn, workspace_id, &input.tool_name)?;
    if let Some(existing) = existing {
        conn.execute(
            "UPDATE ai_tool_registry SET risk_level = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4",
            rusqlite::params![input.risk_level.as_str(), now, actor_user_id, existing.id],
        )?;
        Ok(get_by_tool_name(conn, workspace_id, &input.tool_name)?.expect("just updated"))
    } else {
        let id = new_uuid();
        conn.execute(
            "INSERT INTO ai_tool_registry (id, workspace_id, tool_name, risk_level, created_at, created_by, updated_at, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?5, ?6)",
            rusqlite::params![id, workspace_id, input.tool_name, input.risk_level.as_str(), now, actor_user_id],
        )?;
        Ok(get_by_tool_name(conn, workspace_id, &input.tool_name)?.expect("just inserted"))
    }
}

pub fn clear(conn: &Connection, workspace_id: &str, tool_name: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_tool_registry WHERE workspace_id = ?1 AND tool_name = ?2", rusqlite::params![workspace_id, tool_name])?;
    Ok(())
}
