//! Raw CRUD for `process_lanes`/`process_lane_versions` (migration 0095).
//! See `services::process_lane_service` for the Draft -> Published
//! lifecycle, the compile to an execution graph, and the run-start rules.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::process_lane::{ProcessLane, ProcessLaneVersion};

fn map_lane_row(row: &rusqlite::Row) -> rusqlite::Result<ProcessLane> {
    Ok(ProcessLane {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        lane_key: row.get("lane_key")?,
        name: row.get("name")?,
        description: row.get("description")?,
        object_key: row.get("object_key")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
        versions: Vec::new(), // filled in by `hydrate`
    })
}

fn map_version_row(row: &rusqlite::Row) -> rusqlite::Result<ProcessLaneVersion> {
    Ok(ProcessLaneVersion {
        id: row.get("id")?,
        lane_id: row.get("lane_id")?,
        workspace_id: row.get("workspace_id")?,
        version: row.get("version")?,
        status: row.get("status")?,
        definition_json: row.get("definition_json")?,
        graph_id: row.get("graph_id")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn list_versions(
    conn: &Connection,
    lane_id: &str,
) -> rusqlite::Result<Vec<ProcessLaneVersion>> {
    let mut stmt =
        conn.prepare("SELECT * FROM process_lane_versions WHERE lane_id = ?1 ORDER BY version")?;
    let rows = stmt.query_map([lane_id], map_version_row)?.collect();
    rows
}

fn hydrate(conn: &Connection, mut lane: ProcessLane) -> rusqlite::Result<ProcessLane> {
    lane.versions = list_versions(conn, &lane.id)?;
    Ok(lane)
}

pub fn create_lane(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    lane_key: &str,
    name: &str,
    description: Option<&str>,
    object_key: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<ProcessLane> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO process_lanes (id, workspace_id, lane_key, name, description, object_key, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?7, ?8)",
        (
            id, workspace_id, lane_key, name, description, object_key, now, actor_user_id,
        ),
    )?;
    get_lane(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn create_version(
    conn: &Connection,
    id: &str,
    lane_id: &str,
    workspace_id: &str,
    version: i64,
    status: &str,
    definition_json: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<ProcessLaneVersion> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO process_lane_versions (id, lane_id, workspace_id, version, status, definition_json, graph_id, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8, ?7, ?8)",
        (
            id,
            lane_id,
            workspace_id,
            version,
            status,
            definition_json,
            now,
            actor_user_id,
        ),
    )?;
    get_version_by_id(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get_lane(conn: &Connection, id: &str) -> rusqlite::Result<Option<ProcessLane>> {
    let row = conn
        .query_row(
            "SELECT * FROM process_lanes WHERE id = ?1",
            [id],
            map_lane_row,
        )
        .optional()?;
    row.map(|l| hydrate(conn, l)).transpose()
}

pub fn get_lane_by_key(
    conn: &Connection,
    workspace_id: &str,
    lane_key: &str,
) -> rusqlite::Result<Option<ProcessLane>> {
    let row = conn
        .query_row(
            "SELECT * FROM process_lanes WHERE workspace_id = ?1 AND lane_key = ?2",
            [workspace_id, lane_key],
            map_lane_row,
        )
        .optional()?;
    row.map(|l| hydrate(conn, l)).transpose()
}

pub fn list_lanes(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<ProcessLane>> {
    let mut stmt =
        conn.prepare("SELECT * FROM process_lanes WHERE workspace_id = ?1 ORDER BY name")?;
    let rows: Vec<ProcessLane> = stmt
        .query_map([workspace_id], map_lane_row)?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|l| hydrate(conn, l)).collect()
}

pub fn get_version_by_id(
    conn: &Connection,
    id: &str,
) -> rusqlite::Result<Option<ProcessLaneVersion>> {
    conn.query_row(
        "SELECT * FROM process_lane_versions WHERE id = ?1",
        [id],
        map_version_row,
    )
    .optional()
}

pub fn get_version(
    conn: &Connection,
    lane_id: &str,
    version: i64,
) -> rusqlite::Result<Option<ProcessLaneVersion>> {
    conn.query_row(
        "SELECT * FROM process_lane_versions WHERE lane_id = ?1 AND version = ?2",
        (lane_id, version),
        map_version_row,
    )
    .optional()
}

pub fn latest_version(
    conn: &Connection,
    lane_id: &str,
) -> rusqlite::Result<Option<ProcessLaneVersion>> {
    conn.query_row(
        "SELECT * FROM process_lane_versions WHERE lane_id = ?1 ORDER BY version DESC LIMIT 1",
        [lane_id],
        map_version_row,
    )
    .optional()
}

pub fn latest_published_version(
    conn: &Connection,
    lane_id: &str,
) -> rusqlite::Result<Option<ProcessLaneVersion>> {
    conn.query_row(
        "SELECT * FROM process_lane_versions WHERE lane_id = ?1 AND status = 'published' ORDER BY version DESC LIMIT 1",
        [lane_id],
        map_version_row,
    )
    .optional()
}

pub fn set_version_published(
    conn: &Connection,
    version_id: &str,
    graph_id: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE process_lane_versions SET status = 'published', graph_id = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4",
        (graph_id, now_iso(), actor_user_id, version_id),
    )?;
    Ok(())
}

/// The active run of a lane on one anchor record, if any - the one-active-
/// run rule (ADR-0024 §6). A run is active while its status is not terminal;
/// a run's lane is read from its graph's `source_id` (`{lane_key}@v{n}`),
/// which is how a run keeps its version (ADR-0024 §1).
pub fn find_active_run_for_lane_record(
    conn: &Connection,
    workspace_id: &str,
    lane_key: &str,
    anchor_record_id: &str,
) -> rusqlite::Result<Option<String>> {
    let pattern = format!("{lane_key}@v%");
    conn.query_row(
        "SELECT r.id FROM ai_runs r
          JOIN ai_execution_graphs g ON g.id = r.graph_id
         WHERE r.workspace_id = ?1
           AND g.source_kind = 'lane'
           AND g.source_id LIKE ?2
           AND r.anchor_record_id = ?3
           AND r.status NOT IN ('completed', 'completed_partial', 'failed', 'cancelled')
         ORDER BY r.started_at DESC LIMIT 1",
        (workspace_id, pattern, anchor_record_id),
        |r| r.get(0),
    )
    .optional()
}
