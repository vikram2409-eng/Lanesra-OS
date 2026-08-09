//! FR-RPT: a fixed gallery of parameterized reports, not a query builder -
//! each is a named, read-only aggregate query following the exact pattern
//! `dashboard_service::summary` already uses. Available to any
//! authenticated user, same access level as the dashboard KPIs these
//! reports extend.

use chrono::Utc;
use rusqlite::Connection;

use crate::domain::AppResult;
use crate::models::report::{ArAgingBucket, LostReasonBreakdown, RevenueByMonth, SalesByOwner, WinRateByOwner};

/// `from`/`to` default to a wide-open range when not given, rather than
/// requiring the caller to know today's date.
fn resolve_range(from: &Option<String>, to: &Option<String>) -> (String, String) {
    let from = from.clone().unwrap_or_else(|| "0000-01-01".to_string());
    let to = to.clone().unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());
    (from, to)
}

pub fn revenue_by_month(
    conn: &Connection,
    workspace_id: &str,
    from: &Option<String>,
    to: &Option<String>,
) -> AppResult<Vec<RevenueByMonth>> {
    let (from, to) = resolve_range(from, to);
    let mut stmt = conn.prepare(
        "SELECT strftime('%Y-%m', issue_date) AS month, COUNT(*), COALESCE(SUM(total_cents), 0)
         FROM invoices
         WHERE workspace_id = ?1 AND issue_date IS NOT NULL AND issue_date BETWEEN ?2 AND ?3
           AND status NOT IN ('Draft', 'Void', 'Cancelled')
         GROUP BY month ORDER BY month",
    )?;
    let rows = stmt
        .query_map((workspace_id, &from, &to), |row| {
            Ok(RevenueByMonth {
                month: row.get(0)?,
                invoice_count: row.get(1)?,
                total_cents: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Grouped by owner rather than stage - see the note on
/// `models::report::WinRateByOwner` for why. Uses `updated_at` as the
/// closed-date proxy since opportunities have no dedicated "closed_at"
/// field; the transition to Won/Lost is the last thing that touches it in
/// practice.
pub fn win_rate_by_owner(
    conn: &Connection,
    workspace_id: &str,
    from: &Option<String>,
    to: &Option<String>,
) -> AppResult<Vec<WinRateByOwner>> {
    let (from, to) = resolve_range(from, to);
    let mut stmt = conn.prepare(
        "SELECT o.owner_user_id, COALESCE(u.display_name, 'Unassigned'),
                SUM(CASE WHEN o.status = 'Won' THEN 1 ELSE 0 END),
                SUM(CASE WHEN o.status = 'Lost' THEN 1 ELSE 0 END),
                SUM(CASE WHEN o.status = 'Won' THEN o.value_cents ELSE 0 END)
         FROM opportunities o
         LEFT JOIN users u ON u.id = o.owner_user_id
         WHERE o.workspace_id = ?1 AND o.status IN ('Won', 'Lost')
           AND date(o.updated_at) BETWEEN ?2 AND ?3
         GROUP BY o.owner_user_id
         ORDER BY SUM(CASE WHEN o.status = 'Won' THEN o.value_cents ELSE 0 END) DESC",
    )?;
    let rows = stmt
        .query_map((workspace_id, &from, &to), |row| {
            Ok(WinRateByOwner {
                owner_user_id: row.get(0)?,
                owner_name: row.get(1)?,
                won_count: row.get(2)?,
                lost_count: row.get(3)?,
                won_value_cents: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn lost_reason_breakdown(
    conn: &Connection,
    workspace_id: &str,
    from: &Option<String>,
    to: &Option<String>,
) -> AppResult<Vec<LostReasonBreakdown>> {
    let (from, to) = resolve_range(from, to);
    let mut stmt = conn.prepare(
        "SELECT COALESCE(NULLIF(TRIM(lost_reason), ''), 'No reason given'), COUNT(*), COALESCE(SUM(value_cents), 0)
         FROM opportunities
         WHERE workspace_id = ?1 AND status = 'Lost' AND date(updated_at) BETWEEN ?2 AND ?3
         GROUP BY 1 ORDER BY COUNT(*) DESC",
    )?;
    let rows = stmt
        .query_map((workspace_id, &from, &to), |row| {
            Ok(LostReasonBreakdown {
                reason: row.get(0)?,
                count: row.get(1)?,
                value_cents: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Buckets by days past due as of `as_of_date` (defaults to today), same
/// 0-30/31-60/61-90/90+ windows already used for contract renewal alerts.
/// Bucket order is fixed here rather than left to SQL's GROUP BY, since
/// "Not yet due" and "No due date" don't sort naturally next to the
/// numeric bands.
pub fn ar_aging(conn: &Connection, workspace_id: &str, as_of_date: &Option<String>) -> AppResult<Vec<ArAgingBucket>> {
    let as_of = as_of_date.clone().unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());
    let mut stmt = conn.prepare(
        "SELECT
            CASE
                WHEN due_date IS NULL THEN 'No due date'
                WHEN julianday(?2) - julianday(due_date) <= 0 THEN 'Not yet due'
                WHEN julianday(?2) - julianday(due_date) <= 30 THEN '1-30 days overdue'
                WHEN julianday(?2) - julianday(due_date) <= 60 THEN '31-60 days overdue'
                WHEN julianday(?2) - julianday(due_date) <= 90 THEN '61-90 days overdue'
                ELSE '90+ days overdue'
            END AS bucket,
            COUNT(*), COALESCE(SUM(balance_cents), 0)
         FROM invoices
         WHERE workspace_id = ?1 AND balance_cents > 0 AND status NOT IN ('Draft', 'Void', 'Cancelled')
         GROUP BY bucket",
    )?;
    let mut rows: Vec<ArAgingBucket> = stmt
        .query_map((workspace_id, &as_of), |row| {
            Ok(ArAgingBucket {
                bucket: row.get(0)?,
                invoice_count: row.get(1)?,
                balance_cents: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let order = [
        "Not yet due",
        "1-30 days overdue",
        "31-60 days overdue",
        "61-90 days overdue",
        "90+ days overdue",
        "No due date",
    ];
    rows.sort_by_key(|r| order.iter().position(|b| *b == r.bucket).unwrap_or(usize::MAX));
    Ok(rows)
}

/// Invoices have no owner of their own - attributed via the billed
/// Company's owner_user_id (see `models::report::SalesByOwner`).
pub fn sales_by_owner(
    conn: &Connection,
    workspace_id: &str,
    from: &Option<String>,
    to: &Option<String>,
) -> AppResult<Vec<SalesByOwner>> {
    let (from, to) = resolve_range(from, to);
    let mut stmt = conn.prepare(
        "SELECT c.owner_user_id, COALESCE(u.display_name, 'Unassigned'), COUNT(*), COALESCE(SUM(i.total_cents), 0)
         FROM invoices i
         JOIN companies c ON c.id = i.company_id
         LEFT JOIN users u ON u.id = c.owner_user_id
         WHERE i.workspace_id = ?1 AND i.issue_date IS NOT NULL AND i.issue_date BETWEEN ?2 AND ?3
           AND i.status NOT IN ('Draft', 'Void', 'Cancelled')
         GROUP BY c.owner_user_id
         ORDER BY COALESCE(SUM(i.total_cents), 0) DESC",
    )?;
    let rows = stmt
        .query_map((workspace_id, &from, &to), |row| {
            Ok(SalesByOwner {
                owner_user_id: row.get(0)?,
                owner_name: row.get(1)?,
                invoice_count: row.get(2)?,
                total_cents: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}
