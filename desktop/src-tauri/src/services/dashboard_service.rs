use rusqlite::Connection;

use crate::domain::AppResult;
use crate::models::dashboard::{DashboardSummary, RecentActivity, StageCount};

pub fn summary(conn: &Connection, workspace_id: &str) -> AppResult<DashboardSummary> {
    let (open_pipeline_value_cents, open_pipeline_count): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(value_cents), 0), COUNT(*) FROM opportunities
         WHERE workspace_id = ?1 AND status = 'Open'",
        [workspace_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let won_revenue_cents: i64 = conn.query_row(
        "SELECT COALESCE(SUM(value_cents), 0) FROM opportunities WHERE workspace_id = ?1 AND status = 'Won'",
        [workspace_id],
        |row| row.get(0),
    )?;

    let outstanding_invoices_cents: i64 = conn.query_row(
        "SELECT COALESCE(SUM(balance_cents), 0) FROM invoices
         WHERE workspace_id = ?1 AND status NOT IN ('Paid', 'Void', 'Cancelled', 'Draft')",
        [workspace_id],
        |row| row.get(0),
    )?;

    let (overdue_invoices_cents, overdue_invoices_count): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(balance_cents), 0), COUNT(*) FROM invoices
         WHERE workspace_id = ?1 AND status = 'Overdue'",
        [workspace_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let quotes_awaiting_response: i64 = conn.query_row(
        "SELECT COUNT(*) FROM quotes WHERE workspace_id = ?1 AND status IN ('Sent', 'Viewed')",
        [workspace_id],
        |row| row.get(0),
    )?;

    let mut stage_stmt = conn.prepare(
        "SELECT stage, COUNT(*), COALESCE(SUM(value_cents), 0) FROM opportunities
         WHERE workspace_id = ?1 AND status = 'Open' GROUP BY stage",
    )?;
    let pipeline_by_stage = stage_stmt
        .query_map([workspace_id], |row| {
            Ok(StageCount {
                stage: row.get(0)?,
                count: row.get(1)?,
                value_cents: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut activity_stmt = conn.prepare(
        "SELECT occurred_at, event_type, summary FROM audit_events
         WHERE workspace_id = ?1 ORDER BY occurred_at DESC LIMIT 10",
    )?;
    let recent_activity = activity_stmt
        .query_map([workspace_id], |row| {
            Ok(RecentActivity {
                occurred_at: row.get(0)?,
                event_type: row.get(1)?,
                summary: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(DashboardSummary {
        open_pipeline_value_cents,
        open_pipeline_count,
        won_revenue_cents,
        outstanding_invoices_cents,
        overdue_invoices_cents,
        overdue_invoices_count,
        quotes_awaiting_response,
        pipeline_by_stage,
        recent_activity,
    })
}
