//! Raw reads and writes for `ai_run_events` (migration 0087). See
//! `services::run_event_service`.

use rusqlite::{Connection, OptionalExtension};

use crate::models::run_event::{RunEvent, RUN_EVENT_SCHEMA, RUN_EVENT_VERSION};

fn map(row: &rusqlite::Row) -> rusqlite::Result<RunEvent> {
    let data: String = row.get("data_json")?;
    Ok(RunEvent {
        schema: RUN_EVENT_SCHEMA.into(),
        v: RUN_EVENT_VERSION,
        seq: row.get("seq")?,
        event_type: row.get("type")?,
        at: row.get("at")?,
        workspace_id: row.get("workspace_id")?,
        run_kind: row.get("run_kind")?,
        run_id: row.get("run_id")?,
        step_id: row.get("step_id")?,
        span_id: row.get("span_id")?,
        parent_span_id: row.get("parent_span_id")?,
        data: serde_json::from_str(&data).unwrap_or_default(),
        cursor: row.get("id")?,
    })
}

/// Stores `event` and returns its cursor.
pub fn insert(conn: &Connection, event: &RunEvent) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO ai_run_events (workspace_id, run_kind, run_id, seq, type, at, step_id, span_id, parent_span_id, data_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            event.workspace_id,
            event.run_kind,
            event.run_id,
            event.seq,
            event.event_type,
            event.at,
            event.step_id,
            event.span_id,
            event.parent_span_id,
            event.data.to_string(),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn max_seq(conn: &Connection, run_kind: &str, run_id: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(seq), 0) FROM ai_run_events WHERE run_kind = ?1 AND run_id = ?2",
        [run_kind, run_id],
        |r| r.get(0),
    )
}

/// A run's events after `after_seq`, in order.
pub fn list_for_run(
    conn: &Connection,
    workspace_id: &str,
    run_id: &str,
    after_seq: i64,
) -> rusqlite::Result<Vec<RunEvent>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM ai_run_events WHERE workspace_id = ?1 AND run_id = ?2 AND seq > ?3 ORDER BY seq",
    )?;
    let rows = stmt.query_map(rusqlite::params![workspace_id, run_id, after_seq], map)?;
    rows.collect()
}

/// Events of any of `run_ids` stored after the cursor `after_id`, in the
/// order they were stored (which is `seq` order within each run).
pub fn list_after_cursor(
    conn: &Connection,
    workspace_id: &str,
    run_ids: &[String],
    after_id: i64,
) -> rusqlite::Result<Vec<RunEvent>> {
    let mut out = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT * FROM ai_run_events WHERE workspace_id = ?1 AND run_id = ?2 AND id > ?3 ORDER BY id",
    )?;
    for run_id in run_ids {
        let rows = stmt.query_map(rusqlite::params![workspace_id, run_id, after_id], map)?;
        for row in rows {
            out.push(row?);
        }
    }
    out.sort_by_key(|e| e.cursor);
    Ok(out)
}

/// `pipeline` or `graph` for a run of this workspace, if it exists.
pub fn run_kind_of(
    conn: &Connection,
    workspace_id: &str,
    run_id: &str,
) -> rusqlite::Result<Option<&'static str>> {
    let pipeline = conn
        .query_row(
            "SELECT 1 FROM ai_agent_runs WHERE workspace_id = ?1 AND id = ?2",
            [workspace_id, run_id],
            |_| Ok(()),
        )
        .optional()?;
    if pipeline.is_some() {
        return Ok(Some("pipeline"));
    }
    let graph = conn
        .query_row(
            "SELECT 1 FROM ai_runs WHERE workspace_id = ?1 AND id = ?2",
            [workspace_id, run_id],
            |_| Ok(()),
        )
        .optional()?;
    Ok(graph.map(|_| "graph"))
}
