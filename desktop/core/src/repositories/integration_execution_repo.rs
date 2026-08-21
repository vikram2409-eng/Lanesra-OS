//! Raw CRUD for `integration_executions` (migration 0032) - the unified
//! execution log every Integration Hub subsystem writes into (spec §23):
//! inbound/outbound REST API calls, Connector Action invocations, webhook
//! deliveries, CSV/Bulk import-export runs, and Integration Job runs. See
//! `services::integration_log_service` for the `record`/`finish` pair
//! every caller actually uses, and its Overview KPI aggregation.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::integration::IntegrationExecution;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<IntegrationExecution> {
    Ok(IntegrationExecution {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        execution_type: row.get("execution_type")?,
        correlation_id: row.get("correlation_id")?,
        ref_id: row.get("ref_id")?,
        direction: row.get("direction")?,
        started_at: row.get("started_at")?,
        ended_at: row.get("ended_at")?,
        duration_ms: row.get("duration_ms")?,
        status: row.get("status")?,
        http_status: row.get("http_status")?,
        records_read: row.get("records_read")?,
        records_written: row.get("records_written")?,
        records_skipped: row.get("records_skipped")?,
        records_failed: row.get("records_failed")?,
        retry_count: row.get("retry_count")?,
        error_category: row.get("error_category")?,
        error_message: row.get("error_message")?,
        actor_user_id: row.get("actor_user_id")?,
    })
}

/// Opens a new execution row in `status = "running"` - `finish` closes it.
#[allow(clippy::too_many_arguments)]
pub fn start(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    execution_type: &str,
    correlation_id: Option<&str>,
    ref_id: Option<&str>,
    direction: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO integration_executions
            (id, workspace_id, execution_type, correlation_id, ref_id, direction, started_at, status, actor_user_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'running', ?8)",
        rusqlite::params![id, workspace_id, execution_type, correlation_id, ref_id, direction, now_iso(), actor_user_id],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn finish(
    conn: &Connection,
    id: &str,
    status: &str,
    http_status: Option<i64>,
    records_read: i64,
    records_written: i64,
    records_skipped: i64,
    records_failed: i64,
    retry_count: i64,
    error_category: Option<&str>,
    error_message: Option<&str>,
) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "UPDATE integration_executions SET
            status = ?1, ended_at = ?2,
            duration_ms = CAST((julianday(?2) - julianday(started_at)) * 86400000 AS INTEGER),
            http_status = ?3, records_read = ?4, records_written = ?5, records_skipped = ?6, records_failed = ?7,
            retry_count = ?8, error_category = ?9, error_message = ?10
         WHERE id = ?11",
        rusqlite::params![status, now, http_status, records_read, records_written, records_skipped, records_failed, retry_count, error_category, error_message, id],
    )?;
    Ok(())
}

pub struct ExecutionFilter {
    pub execution_type: Option<String>,
    pub status: Option<String>,
    pub correlation_id: Option<String>,
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str, filter: &ExecutionFilter, limit: i64) -> rusqlite::Result<Vec<IntegrationExecution>> {
    let mut sql = "SELECT * FROM integration_executions WHERE workspace_id = ?1".to_string();
    if filter.execution_type.is_some() {
        sql.push_str(" AND execution_type = ?2");
    }
    if filter.status.is_some() {
        sql.push_str(" AND status = ?3");
    }
    if filter.correlation_id.is_some() {
        sql.push_str(" AND correlation_id = ?4");
    }
    sql.push_str(" ORDER BY started_at DESC LIMIT ?5");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params![workspace_id, filter.execution_type, filter.status, filter.correlation_id, limit],
        map_row,
    )?;
    rows.collect()
}

pub fn count_since(conn: &Connection, workspace_id: &str, execution_type: Option<&str>, status: Option<&str>, since_iso: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM integration_executions
         WHERE workspace_id = ?1 AND started_at >= ?2
           AND (?3 IS NULL OR execution_type = ?3)
           AND (?4 IS NULL OR status = ?4)",
        rusqlite::params![workspace_id, since_iso, execution_type, status],
        |r| r.get(0),
    )
}

/// Deletes execution rows older than the workspace's configured retention
/// window (spec §22: "Log retention (days), configurable"). Returns how
/// many rows were purged.
pub fn purge_older_than(conn: &Connection, workspace_id: &str, cutoff_iso: &str) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM integration_executions WHERE workspace_id = ?1 AND started_at < ?2",
        rusqlite::params![workspace_id, cutoff_iso],
    )
}
