//! Raw CRUD for `graph_pending_runs` (migration 0067) - see that
//! migration's own doc comment. Mirrors `ai_agent_pending_run_repo.rs`
//! exactly, same reasoning (a record-save context must never block on a
//! real graph run).

use rusqlite::Connection;

use crate::domain::ids::now_iso;

pub struct PendingGraphRun {
    pub id: String,
    pub workspace_id: String,
    pub graph_id: String,
    pub trigger_input: String,
    pub triggered_by: Option<String>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<PendingGraphRun> {
    Ok(PendingGraphRun {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        graph_id: row.get("graph_id")?,
        trigger_input: row.get("trigger_input")?,
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
    graph_id: &str,
    trigger_input: &str,
    triggered_by: Option<&str>,
    source_entity_type: Option<&str>,
    source_entity_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO graph_pending_runs (id, workspace_id, graph_id, trigger_input, triggered_by, source_entity_type, source_entity_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (id, workspace_id, graph_id, trigger_input, triggered_by, source_entity_type, source_entity_id, now_iso()),
    )?;
    Ok(())
}

pub fn list_batch(conn: &Connection, workspace_id: &str, limit: i64) -> rusqlite::Result<Vec<PendingGraphRun>> {
    let mut stmt = conn.prepare("SELECT * FROM graph_pending_runs WHERE workspace_id = ?1 ORDER BY created_at LIMIT ?2")?;
    let rows = stmt.query_map(rusqlite::params![workspace_id, limit], map_row)?;
    rows.collect()
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM graph_pending_runs WHERE id = ?1", [id])?;
    Ok(())
}
