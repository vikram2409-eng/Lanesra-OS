//! Raw CRUD for `integration_jobs`/`integration_job_runs` (migration
//! 0032) - see `services::integration_job_service`.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::integration::{IntegrationJob, IntegrationJobRun};

fn map_job(row: &rusqlite::Row) -> rusqlite::Result<IntegrationJob> {
    Ok(IntegrationJob {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        external_object_id: row.get("external_object_id")?,
        target_object_key: row.get("target_object_key")?,
        match_key: row.get("match_key")?,
        cursor_field: row.get("cursor_field")?,
        cursor_value: row.get("cursor_value")?,
        interval_minutes: row.get("interval_minutes")?,
        status: row.get("status")?,
        last_run_at: row.get("last_run_at")?,
        last_run_status: row.get("last_run_status")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

fn map_run(row: &rusqlite::Row) -> rusqlite::Result<IntegrationJobRun> {
    Ok(IntegrationJobRun {
        id: row.get("id")?,
        job_id: row.get("job_id")?,
        workspace_id: row.get("workspace_id")?,
        started_at: row.get("started_at")?,
        finished_at: row.get("finished_at")?,
        status: row.get("status")?,
        records_processed: row.get("records_processed")?,
        records_failed: row.get("records_failed")?,
        error_message: row.get("error_message")?,
        cursor_before: row.get("cursor_before")?,
        cursor_after: row.get("cursor_after")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    name: &str,
    external_object_id: &str,
    target_object_key: &str,
    match_key: &str,
    cursor_field: Option<&str>,
    interval_minutes: i64,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<IntegrationJob> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO integration_jobs
            (id, workspace_id, name, external_object_id, target_object_key, match_key, cursor_field, interval_minutes, status, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?10, ?9, ?10)",
        rusqlite::params![id, workspace_id, name, external_object_id, target_object_key, match_key, cursor_field, interval_minutes, now, actor_user_id],
    )?;
    get(conn, id).map(|j| j.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<IntegrationJob>> {
    conn.query_row("SELECT * FROM integration_jobs WHERE id = ?1", [id], map_job).optional()
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<IntegrationJob>> {
    let mut stmt = conn.prepare("SELECT * FROM integration_jobs WHERE workspace_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map([workspace_id], map_job)?;
    rows.collect()
}

/// How many Jobs currently depend on this External Object - `Delete` is
/// blocked while any exist (`external_object_service::delete`'s own
/// dependency check).
pub fn count_by_external_object(conn: &Connection, external_object_id: &str) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM integration_jobs WHERE external_object_id = ?1", [external_object_id], |r| r.get(0))
}

/// Every active job whose `interval_minutes` has elapsed since its last
/// run (or that has never run at all) - what the scheduler loop (and the
/// desktop client-poll equivalent) actually iterates.
pub fn list_due(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<IntegrationJob>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM integration_jobs
         WHERE workspace_id = ?1 AND status = 'active'
           AND (last_run_at IS NULL OR datetime(last_run_at, '+' || interval_minutes || ' minutes') <= datetime('now'))
         ORDER BY name",
    )?;
    let rows = stmt.query_map([workspace_id], map_job)?;
    rows.collect()
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    conn: &Connection,
    id: &str,
    name: &str,
    external_object_id: &str,
    target_object_key: &str,
    match_key: &str,
    cursor_field: Option<&str>,
    interval_minutes: i64,
    status: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE integration_jobs SET name = ?1, external_object_id = ?2, target_object_key = ?3, match_key = ?4, cursor_field = ?5, interval_minutes = ?6, status = ?7, updated_at = ?8, updated_by = ?9 WHERE id = ?10",
        rusqlite::params![name, external_object_id, target_object_key, match_key, cursor_field, interval_minutes, status, now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM integration_job_runs WHERE job_id = ?1", [id])?;
    conn.execute("DELETE FROM integration_jobs WHERE id = ?1", [id])?;
    Ok(())
}

pub fn record_run_outcome(conn: &Connection, id: &str, status: &str, cursor_value: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE integration_jobs SET last_run_at = ?1, last_run_status = ?2, cursor_value = COALESCE(?3, cursor_value), updated_at = ?1 WHERE id = ?4",
        rusqlite::params![now_iso(), status, cursor_value, id],
    )?;
    Ok(())
}

pub fn insert_run_started(conn: &Connection, id: &str, job_id: &str, workspace_id: &str, cursor_before: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO integration_job_runs (id, job_id, workspace_id, started_at, status, cursor_before) VALUES (?1, ?2, ?3, ?4, 'running', ?5)",
        rusqlite::params![id, job_id, workspace_id, now_iso(), cursor_before],
    )?;
    Ok(())
}

pub fn finish_run(conn: &Connection, id: &str, status: &str, records_processed: i64, records_failed: i64, error_message: Option<&str>, cursor_after: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE integration_job_runs SET finished_at = ?1, status = ?2, records_processed = ?3, records_failed = ?4, error_message = ?5, cursor_after = ?6 WHERE id = ?7",
        rusqlite::params![now_iso(), status, records_processed, records_failed, error_message, cursor_after, id],
    )?;
    Ok(())
}

pub fn get_run(conn: &Connection, id: &str) -> rusqlite::Result<Option<IntegrationJobRun>> {
    conn.query_row("SELECT * FROM integration_job_runs WHERE id = ?1", [id], map_run).optional()
}

pub fn list_runs_for_job(conn: &Connection, job_id: &str, limit: i64) -> rusqlite::Result<Vec<IntegrationJobRun>> {
    let mut stmt = conn.prepare("SELECT * FROM integration_job_runs WHERE job_id = ?1 ORDER BY started_at DESC LIMIT ?2")?;
    let rows = stmt.query_map(rusqlite::params![job_id, limit], map_run)?;
    rows.collect()
}
