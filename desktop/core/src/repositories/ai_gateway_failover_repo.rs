//! Raw storage for `ai_gateway_failover_events` (migration 0041) - one row
//! per gateway dispatch that didn't succeed on its primary tier. See
//! `services::ai_gateway_service` for when a row is written and the
//! Gateway health view (Admin -> LLM & MCP -> Gateway) for where it's read.

use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::ai::AiGatewayFailoverEvent;

pub fn record(conn: &Connection, workspace_id: &str, agent_id: &str, served_by: &str, reason: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_gateway_failover_events (id, workspace_id, agent_id, served_by, reason, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![new_uuid(), workspace_id, agent_id, served_by, reason, now_iso()],
    )?;
    Ok(())
}

pub fn list_recent(conn: &Connection, workspace_id: &str, limit: i64) -> rusqlite::Result<Vec<AiGatewayFailoverEvent>> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.agent_id, a.name AS agent_name, e.served_by, e.reason, e.created_at
         FROM ai_gateway_failover_events e
         JOIN ai_agents a ON a.id = e.agent_id
         WHERE e.workspace_id = ?1
         ORDER BY e.created_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(rusqlite::params![workspace_id, limit], |row| {
        Ok(AiGatewayFailoverEvent {
            id: row.get("id")?,
            agent_id: row.get("agent_id")?,
            agent_name: row.get("agent_name")?,
            served_by: row.get("served_by")?,
            reason: row.get("reason")?,
            created_at: row.get("created_at")?,
        })
    })?;
    rows.collect()
}
