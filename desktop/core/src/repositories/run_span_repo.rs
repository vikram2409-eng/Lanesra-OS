//! Raw CRUD for `ai_run_spans`/`ai_run_span_bodies` (migration 0085).
//! See `services::trace_service`.

use rusqlite::{Connection, OptionalExtension};

use crate::models::run_trace::{RunSpan, SpanBody};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<RunSpan> {
    let attributes_json: String = row.get("attributes_json")?;
    Ok(RunSpan {
        span_id: row.get("span_id")?,
        trace_id: row.get("trace_id")?,
        parent_span_id: row.get("parent_span_id")?,
        run_kind: row.get("run_kind")?,
        run_id: row.get("run_id")?,
        seq: row.get("seq")?,
        kind: row.get("kind")?,
        name: row.get("name")?,
        status: row.get("status")?,
        started_at: row.get("started_at")?,
        ended_at: row.get("ended_at")?,
        duration_ms: row.get("duration_ms")?,
        attributes: serde_json::from_str(&attributes_json).unwrap_or_default(),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    span_id: &str,
    trace_id: &str,
    parent_span_id: Option<&str>,
    workspace_id: &str,
    run_kind: &str,
    run_id: &str,
    seq: i64,
    kind: &str,
    name: &str,
    started_at: &str,
    attributes_json: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_run_spans (span_id, trace_id, parent_span_id, workspace_id, run_kind, run_id, seq, kind, name, started_at, attributes_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            span_id,
            trace_id,
            parent_span_id,
            workspace_id,
            run_kind,
            run_id,
            seq,
            kind,
            name,
            started_at,
            attributes_json
        ],
    )?;
    Ok(())
}

pub fn finish(
    conn: &Connection,
    span_id: &str,
    status: &str,
    ended_at: &str,
    duration_ms: i64,
    attributes_json: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ai_run_spans SET status = ?2, ended_at = ?3, duration_ms = ?4, attributes_json = ?5 WHERE span_id = ?1",
        rusqlite::params![span_id, status, ended_at, duration_ms, attributes_json],
    )?;
    Ok(())
}

pub fn put_body(conn: &Connection, span_id: &str, part: &str, body: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO ai_run_span_bodies (span_id, part, body) VALUES (?1, ?2, ?3)",
        rusqlite::params![span_id, part, body],
    )?;
    Ok(())
}

/// The run's root span, if it was traced before (a resumed run continues
/// its trace).
pub fn root_for_run(
    conn: &Connection,
    run_kind: &str,
    run_id: &str,
) -> rusqlite::Result<Option<RunSpan>> {
    conn.query_row(
        "SELECT * FROM ai_run_spans WHERE run_kind = ?1 AND run_id = ?2 AND kind = 'run' ORDER BY seq LIMIT 1",
        [run_kind, run_id],
        map_row,
    )
    .optional()
}

pub fn max_seq(conn: &Connection, run_kind: &str, run_id: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(seq), 0) FROM ai_run_spans WHERE run_kind = ?1 AND run_id = ?2",
        [run_kind, run_id],
        |r| r.get(0),
    )
}

pub fn list_for_run(
    conn: &Connection,
    workspace_id: &str,
    run_kind: &str,
    run_id: &str,
) -> rusqlite::Result<Vec<RunSpan>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM ai_run_spans WHERE workspace_id = ?1 AND run_kind = ?2 AND run_id = ?3 ORDER BY seq",
    )?;
    let rows = stmt.query_map([workspace_id, run_kind, run_id], map_row)?;
    rows.collect()
}

pub fn bodies_for_span(
    conn: &Connection,
    workspace_id: &str,
    span_id: &str,
) -> rusqlite::Result<Vec<SpanBody>> {
    let mut stmt = conn.prepare(
        "SELECT b.span_id, b.part, b.body FROM ai_run_span_bodies b
         JOIN ai_run_spans s ON s.span_id = b.span_id
         WHERE s.workspace_id = ?1 AND b.span_id = ?2 ORDER BY b.part",
    )?;
    let rows = stmt.query_map([workspace_id, span_id], |row| {
        Ok(SpanBody {
            span_id: row.get(0)?,
            part: row.get(1)?,
            body: row.get(2)?,
        })
    })?;
    rows.collect()
}
