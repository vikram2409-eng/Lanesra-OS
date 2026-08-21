//! Integration Hub (spec §23): the unified execution log every subsystem
//! writes into - inbound/outbound REST API calls, Connector Action
//! invocations, Bulk API jobs, CSV import/export runs, webhook deliveries
//! and Integration Job runs all become one `integration_executions` row,
//! filterable from one screen instead of scattered per-feature logs (spec:
//! "single pane of glass for all integration activity").
//!
//! `start`/`finish` bracket one execution; callers that already know the
//! whole outcome up front (nothing here needs that yet, but a future
//! synchronous one-shot could) may call `finish` immediately after
//! `start`. Every call site is expected to swallow a log-write failure the
//! same way `event_hooks` does for its own writes - logging must never be
//! the reason a real operation fails.
//!
//! The Overview KPI row (`overview`) is a handful of real aggregate
//! queries over `integration_connections`/`integration_webhook_deliveries`/
//! `integration_executions`/`integration_jobs` - the same "plain
//! `conn.query_row` aggregates, no ORM" convention `dashboard_service`
//! already uses for its own KPI row. `integration_jobs`/`integration_job_runs`
//! (migration 0032) are now driven by `integration_job_service` - these
//! KPI columns were already real queries before that existed, so nothing
//! here needed to change once it did.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::AppResult;
use crate::models::integration::{IntegrationExecution, IntegrationOverview, IntegrationSettings, IntegrationSettingsUpdate};
use crate::repositories::integration_execution_repo::{self, ExecutionFilter};
use crate::repositories::{integration_connection_repo, integration_settings_repo, integration_webhook_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// Opens a new execution log row and returns its id - pass that id to
/// `finish` once the operation completes (success or failure).
pub fn start(conn: &Connection, workspace_id: &str, execution_type: &str, correlation_id: Option<&str>, ref_id: Option<&str>, direction: &str, actor_user_id: Option<&str>) -> String {
    let id = new_uuid();
    let _ = integration_execution_repo::start(conn, &id, workspace_id, execution_type, correlation_id, ref_id, direction, actor_user_id);
    id
}

#[derive(Debug, Clone, Default)]
pub struct FinishOutcome {
    pub status: String,
    pub http_status: Option<i64>,
    pub records_read: i64,
    pub records_written: i64,
    pub records_skipped: i64,
    pub records_failed: i64,
    pub retry_count: i64,
    pub error_category: Option<String>,
    pub error_message: Option<String>,
}

pub fn finish(conn: &Connection, execution_id: &str, outcome: &FinishOutcome) {
    let _ = integration_execution_repo::finish(
        conn,
        execution_id,
        &outcome.status,
        outcome.http_status,
        outcome.records_read,
        outcome.records_written,
        outcome.records_skipped,
        outcome.records_failed,
        outcome.retry_count,
        outcome.error_category.as_deref(),
        outcome.error_message.as_deref(),
    );
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionQuery {
    pub execution_type: Option<String>,
    pub status: Option<String>,
    pub correlation_id: Option<String>,
    pub limit: Option<i64>,
}

pub fn list_executions(conn: &Connection, workspace_id: &str, query: &ExecutionQuery) -> AppResult<Vec<IntegrationExecution>> {
    let filter = ExecutionFilter { execution_type: query.execution_type.clone(), status: query.status.clone(), correlation_id: query.correlation_id.clone() };
    Ok(integration_execution_repo::list_for_workspace(conn, workspace_id, &filter, query.limit.unwrap_or(200).clamp(1, 1000))?)
}

fn today_start_iso() -> String {
    // Same coarse "midnight UTC" cutoff `numbering_service`'s own period-key
    // convention uses elsewhere in this crate - good enough for a KPI, not
    // meant to be a precise timezone-aware "today".
    let now = crate::domain::ids::now_iso();
    format!("{}T00:00:00Z", &now[..10])
}

/// The Overview screen's KPI row (spec §3.1) - real aggregates, not
/// placeholders; see this module's own doc comment on why the jobs
/// columns are safe to query before `integration_job_service` exists.
pub fn overview(conn: &Connection, workspace_id: &str) -> AppResult<IntegrationOverview> {
    let since = today_start_iso();
    let active_connections = integration_connection_repo::count_by_status(conn, workspace_id, "connected")?;
    let failed_connections = integration_connection_repo::count_by_status(conn, workspace_id, "failed")?;
    let api_calls_today = integration_execution_repo::count_since(conn, workspace_id, Some("api_call"), None, &since)?;
    let failed_webhooks_today = integration_webhook_repo::count_failed_deliveries_since(conn, workspace_id, &since)?;
    let jobs_running: i64 = conn.query_row(
        "SELECT COUNT(*) FROM integration_job_runs r JOIN integration_jobs j ON j.id = r.job_id WHERE j.workspace_id = ?1 AND r.status = 'running'",
        [workspace_id],
        |r| r.get(0),
    )?;
    let jobs_failed_today: i64 = conn.query_row(
        "SELECT COUNT(*) FROM integration_job_runs r JOIN integration_jobs j ON j.id = r.job_id WHERE j.workspace_id = ?1 AND r.status = 'failed' AND r.started_at >= ?2",
        rusqlite::params![workspace_id, since],
        |r| r.get(0),
    )?;
    Ok(IntegrationOverview { active_connections, failed_connections, api_calls_today, failed_webhooks_today, jobs_running, jobs_failed_today })
}

pub fn get_settings(conn: &Connection, workspace_id: &str) -> AppResult<IntegrationSettings> {
    Ok(integration_settings_repo::ensure_default(conn, workspace_id)?)
}

pub fn update_settings(conn: &Connection, workspace_id: &str, input: &IntegrationSettingsUpdate, actor_user_id: Option<&str>) -> AppResult<IntegrationSettings> {
    require_admin(conn, actor_user_id)?;
    integration_settings_repo::ensure_default(conn, workspace_id)?;
    Ok(integration_settings_repo::update(
        conn,
        workspace_id,
        input.api_rate_limit_per_minute.max(1),
        input.global_rate_limit_per_minute.max(1),
        input.log_retention_days.max(1),
        input.file_retention_days.max(1),
        input.allow_insecure_connections,
        actor_user_id,
    )?)
}

/// Purges execution log rows older than the workspace's configured
/// retention (spec §22) - meant to be called periodically (the same
/// client-poll or server-scheduler cadence `integration_job_service` will
/// eventually run on), not on every request.
pub fn purge_expired(conn: &Connection, workspace_id: &str) -> AppResult<usize> {
    let settings = get_settings(conn, workspace_id)?;
    let cutoff = {
        let now = crate::domain::ids::now_iso();
        // Cheap day-subtraction via SQLite's own date math rather than
        // pulling in a date-arithmetic crate for one KPI-adjacent helper.
        conn.query_row("SELECT datetime(?1, ?2)", rusqlite::params![now, format!("-{} days", settings.log_retention_days)], |r| r.get::<_, String>(0))?
    };
    Ok(integration_execution_repo::purge_older_than(conn, workspace_id, &cutoff)?)
}
