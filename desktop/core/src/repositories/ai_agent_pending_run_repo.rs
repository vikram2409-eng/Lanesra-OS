//! Raw CRUD for `ai_agent_pending_runs` (migration 0040) - the queue a
//! due schedule Trigger, an inbound webhook, and the new
//! `run_ai_agent`/`run_ai_agent_pipeline` Workflow Automation action all
//! write into; `services::ai_orchestration_service::drain_pending_runs`
//! is the async drain that actually runs each one. Mirrors
//! `integration_pending_action_repo.rs` exactly - same reasoning as that
//! module's own doc comment (a record-save or scheduler-tick context
//! must never block on a real LLM call).

use rusqlite::Connection;

use crate::domain::ids::now_iso;

pub struct PendingRun {
    pub id: String,
    pub workspace_id: String,
    pub target_type: String,
    pub target_id: String,
    pub resolved_input_text: String,
    pub triggered_by: Option<String>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<PendingRun> {
    Ok(PendingRun {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        target_type: row.get("target_type")?,
        target_id: row.get("target_id")?,
        resolved_input_text: row.get("resolved_input_text")?,
        triggered_by: row.get("triggered_by")?,
        source_entity_type: row.get("source_entity_type")?,
        source_entity_id: row.get("source_entity_id")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn enqueue(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    target_type: &str,
    target_id: &str,
    resolved_input_text: &str,
    triggered_by: Option<&str>,
    source_entity_type: Option<&str>,
    source_entity_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_agent_pending_runs (id, workspace_id, target_type, target_id, resolved_input_text, triggered_by, source_entity_type, source_entity_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        (id, workspace_id, target_type, target_id, resolved_input_text, triggered_by, source_entity_type, source_entity_id, now_iso()),
    )?;
    Ok(())
}

pub fn list_batch(conn: &Connection, workspace_id: &str, limit: i64) -> rusqlite::Result<Vec<PendingRun>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_agent_pending_runs WHERE workspace_id = ?1 ORDER BY created_at LIMIT ?2")?;
    let rows = stmt.query_map(rusqlite::params![workspace_id, limit], map_row)?;
    rows.collect()
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_agent_pending_runs WHERE id = ?1", [id])?;
    Ok(())
}
