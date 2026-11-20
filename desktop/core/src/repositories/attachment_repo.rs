//! Storage for files attached to a lane run (WP-0711, REQ-LANE-018).
//!
//! The rows live in the 0001 `attachments` table with
//! `entity_type = "lane_run"` and the run id as `entity_id`; the bytes live
//! on disk under the data dir (the service writes them, this repo only
//! records the row). Nothing in code read or wrote this table before this
//! work package.

use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::lane_run_note::LaneRunAttachment;

/// The entity type under which lane-run attachments are stored.
pub const LANE_RUN_ENTITY_TYPE: &str = "lane_run";

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<LaneRunAttachment> {
    Ok(LaneRunAttachment {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        run_id: row.get("entity_id")?,
        file_name: row.get("file_name")?,
        relative_path: row.get("relative_path")?,
        mime_type: row.get("mime_type")?,
        size_bytes: row.get("size_bytes")?,
        checksum: row.get("checksum")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
    })
}

/// Inserts one attachment row for a run. The caller writes the bytes first
/// and passes the path relative to the data dir.
#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    workspace_id: &str,
    run_id: &str,
    file_name: &str,
    relative_path: &str,
    mime_type: Option<&str>,
    size_bytes: i64,
    checksum: &str,
    created_by: Option<&str>,
) -> rusqlite::Result<LaneRunAttachment> {
    let id = new_uuid();
    let created_at = now_iso();
    conn.execute(
        "INSERT INTO attachments (id, workspace_id, entity_type, entity_id, file_name, relative_path, mime_type, size_bytes, checksum, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        (
            &id,
            workspace_id,
            LANE_RUN_ENTITY_TYPE,
            run_id,
            file_name,
            relative_path,
            mime_type,
            size_bytes,
            checksum,
            &created_at,
            created_by,
        ),
    )?;
    Ok(LaneRunAttachment {
        id,
        workspace_id: workspace_id.to_string(),
        run_id: run_id.to_string(),
        file_name: file_name.to_string(),
        relative_path: relative_path.to_string(),
        mime_type: mime_type.map(String::from),
        size_bytes: Some(size_bytes),
        checksum: Some(checksum.to_string()),
        created_at,
        created_by: created_by.map(String::from),
    })
}

/// All attachments on a run, oldest first.
pub fn list_for_run(conn: &Connection, run_id: &str) -> rusqlite::Result<Vec<LaneRunAttachment>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM attachments WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY created_at ASC, id ASC",
    )?;
    let rows = stmt.query_map((LANE_RUN_ENTITY_TYPE, run_id), map_row)?;
    rows.collect()
}

/// One attachment on a run, or None. The run id is part of the lookup so an
/// attachment can only be read through its own run (AC-3).
pub fn get_for_run(
    conn: &Connection,
    run_id: &str,
    attachment_id: &str,
) -> rusqlite::Result<Option<LaneRunAttachment>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM attachments WHERE entity_type = ?1 AND entity_id = ?2 AND id = ?3",
    )?;
    let mut rows = stmt.query_map((LANE_RUN_ENTITY_TYPE, run_id, attachment_id), map_row)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}
