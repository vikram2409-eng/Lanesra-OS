//! Storage for `models::lane_run_note::LaneRunNote` (WP-0711, REQ-LANE-018).

use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::lane_run_note::LaneRunNote;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<LaneRunNote> {
    Ok(LaneRunNote {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        run_id: row.get("run_id")?,
        step_id: row.get("step_id")?,
        author_user_id: row.get("author_user_id")?,
        author_display_name: None,
        body: row.get("body")?,
        created_at: row.get("created_at")?,
    })
}

/// Inserts one note on a run. The caller validates the run first; this
/// function only stores the row.
pub fn create(
    conn: &Connection,
    workspace_id: &str,
    run_id: &str,
    step_id: Option<&str>,
    author_user_id: Option<&str>,
    body: &str,
) -> rusqlite::Result<LaneRunNote> {
    let id = new_uuid();
    let created_at = now_iso();
    conn.execute(
        "INSERT INTO process_lane_run_notes (id, workspace_id, run_id, step_id, author_user_id, body, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            &id,
            workspace_id,
            run_id,
            step_id,
            author_user_id,
            body,
            &created_at,
        ),
    )?;
    Ok(LaneRunNote {
        id,
        workspace_id: workspace_id.to_string(),
        run_id: run_id.to_string(),
        step_id: step_id.map(String::from),
        author_user_id: author_user_id.map(String::from),
        author_display_name: None,
        body: body.to_string(),
        created_at,
    })
}

/// All notes on a run, oldest first (the order the runner shows them).
pub fn list_for_run(conn: &Connection, run_id: &str) -> rusqlite::Result<Vec<LaneRunNote>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM process_lane_run_notes WHERE run_id = ?1 ORDER BY created_at ASC, id ASC",
    )?;
    let rows = stmt.query_map((run_id,), map_row)?;
    rows.collect()
}
