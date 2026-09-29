//! Raw CRUD for `ai_memory_items` (migration `0063_memory_architecture.sql`)
//! - the three itemized memory types (Session/Working/Entity). See
//! `services::ai_memory_service` for the write-side policy gate and the
//! context-scoped read this repo's `list_for_context` exists for.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::ai_memory::MemoryItem;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<MemoryItem> {
    Ok(MemoryItem {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        memory_type: row.get("memory_type")?,
        agent_id: row.get("agent_id")?,
        session_key: row.get("session_key")?,
        run_id: row.get("run_id")?,
        entity_type: row.get("entity_type")?,
        entity_id: row.get("entity_id")?,
        content: row.get("content")?,
        source: row.get("source")?,
        confidence: row.get("confidence")?,
        classification: row.get("classification")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        expires_at: row.get("expires_at")?,
        last_used_at: row.get("last_used_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    workspace_id: &str,
    memory_type: &str,
    agent_id: Option<&str>,
    session_key: Option<&str>,
    run_id: Option<&str>,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    content: &str,
    source: &str,
    confidence: Option<f64>,
    classification: &str,
    created_by: Option<&str>,
    expires_at: Option<&str>,
) -> rusqlite::Result<MemoryItem> {
    let id = new_uuid();
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_memory_items (id, workspace_id, memory_type, agent_id, session_key, run_id, entity_type, entity_id, content, source, confidence, classification, created_at, created_by, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        rusqlite::params![id, workspace_id, memory_type, agent_id, session_key, run_id, entity_type, entity_id, content, source, confidence, classification, now, created_by, expires_at],
    )?;
    get(conn, &id).map(|r| r.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<MemoryItem>> {
    conn.query_row("SELECT * FROM ai_memory_items WHERE id = ?1", [id], map_row).optional()
}

/// Non-expired Entity Memory for one record - every agent's items, since
/// a durable fact about a record isn't scoped to whichever agent first
/// observed it.
pub fn list_entity(conn: &Connection, workspace_id: &str, entity_type: &str, entity_id: &str) -> rusqlite::Result<Vec<MemoryItem>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM ai_memory_items WHERE workspace_id = ?1 AND memory_type = 'entity' AND entity_type = ?2 AND entity_id = ?3
         AND (expires_at IS NULL OR expires_at > ?4) ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(rusqlite::params![workspace_id, entity_type, entity_id, now_iso()], map_row)?.collect();
    rows
}

/// Non-expired Session Memory for one agent's ongoing relationship with
/// one user - see `models::ai_memory::AgentMemoryContext`'s own doc
/// comment on why `session_key` is that pair, not a real conversation id.
pub fn list_session(conn: &Connection, agent_id: &str, session_key: &str, limit: i64) -> rusqlite::Result<Vec<MemoryItem>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM ai_memory_items WHERE memory_type = 'session' AND agent_id = ?1 AND session_key = ?2
         AND (expires_at IS NULL OR expires_at > ?3) ORDER BY created_at DESC LIMIT ?4",
    )?;
    let rows = stmt.query_map(rusqlite::params![agent_id, session_key, now_iso(), limit], map_row)?.collect();
    rows
}

/// Non-expired Working Memory for one real run.
pub fn list_working(conn: &Connection, agent_id: &str, run_id: &str) -> rusqlite::Result<Vec<MemoryItem>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM ai_memory_items WHERE memory_type = 'working' AND agent_id = ?1 AND run_id = ?2
         AND (expires_at IS NULL OR expires_at > ?3) ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(rusqlite::params![agent_id, run_id, now_iso()], map_row)?.collect();
    rows
}

/// Every memory item in a workspace, optionally narrowed by type and/or
/// entity - the admin Memory Inspector's own listing, unfiltered by
/// expiry so an admin can see (and clean up) an already-expired item too.
pub fn list_all(conn: &Connection, workspace_id: &str, memory_type: Option<&str>, entity_type: Option<&str>, entity_id: Option<&str>) -> rusqlite::Result<Vec<MemoryItem>> {
    let mut sql = "SELECT * FROM ai_memory_items WHERE workspace_id = ?1".to_string();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(workspace_id.to_string())];
    if let Some(t) = memory_type {
        sql.push_str(&format!(" AND memory_type = ?{}", params.len() + 1));
        params.push(Box::new(t.to_string()));
    }
    if let Some(et) = entity_type {
        sql.push_str(&format!(" AND entity_type = ?{}", params.len() + 1));
        params.push(Box::new(et.to_string()));
    }
    if let Some(eid) = entity_id {
        sql.push_str(&format!(" AND entity_id = ?{}", params.len() + 1));
        params.push(Box::new(eid.to_string()));
    }
    sql.push_str(" ORDER BY created_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), map_row)?.collect();
    rows
}

pub fn delete(conn: &Connection, workspace_id: &str, id: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM ai_memory_items WHERE id = ?1 AND workspace_id = ?2", rusqlite::params![id, workspace_id])
}

pub fn touch_last_used(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE ai_memory_items SET last_used_at = ?1 WHERE id = ?2", rusqlite::params![now_iso(), id])?;
    Ok(())
}

/// Every already-expired item in a workspace, regardless of type -
/// `ai_memory_service::sweep_expired` deletes what this returns, the same
/// TTL-reclaim shape `workflow_service::run_scheduled`'s own sweep uses.
pub fn sweep_expired(conn: &Connection, workspace_id: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM ai_memory_items WHERE workspace_id = ?1 AND expires_at IS NOT NULL AND expires_at <= ?2", rusqlite::params![workspace_id, now_iso()])
}
