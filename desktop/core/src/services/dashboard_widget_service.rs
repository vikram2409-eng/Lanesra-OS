//! Dashboard customization Phase 3: record-list widgets - a short list of
//! records for one entity type, either the most recently created ("recent",
//! any entity type including custom objects) or, for the two entities that
//! actually carry a due date, the soonest-due open ones ("due_soon" - open
//! Tasks by `due_date`, or unpaid Invoices by `due_date`). Every other
//! entity type requesting "due_soon" falls back to "recent" - not every
//! mode makes sense for every type, the same "sum doesn't apply to every
//! field either" scoping `custom_report_service`'s aggregate already
//! accepts.
//!
//! Deliberately its own small module rather than folded into
//! `dashboard_layout_service` (which only ever handles layout CRUD/publish,
//! never runs a widget's data) or `custom_report_service` (a different
//! shaped query - a flat recent-first list, not a grouped aggregate).

use rusqlite::Connection;
use serde::Serialize;

use crate::domain::AppResult;
use crate::repositories::{
    company_repo, contact_repo, contract_repo, custom_record_repo, invoice_repo, opportunity_repo, order_repo,
    product_repo, quote_repo, task_repo,
};

pub const RECORD_LIST_MODES: &[&str] = &["recent", "due_soon"];

/// Hard cap regardless of what a widget's `limit` config asks for - a
/// dashboard tile is a glance, not a list screen.
const MAX_ROWS: usize = 10;

#[derive(Debug, Clone, Serialize)]
pub struct RecordListRow {
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    /// A short secondary line - a due date for "due_soon" rows, or the
    /// creation date for "recent" rows - same "explain the sort at a
    /// glance" role `SearchResult::subtitle` already plays for search.
    pub subtitle: Option<String>,
}

fn due_soon_tasks(conn: &Connection, workspace_id: &str, limit: usize) -> AppResult<Vec<RecordListRow>> {
    let mut tasks: Vec<_> = task_repo::list(conn, workspace_id)?
        .into_iter()
        .filter(|t| t.due_date.is_some() && t.status != "Completed" && t.status != "Cancelled")
        .collect();
    tasks.sort_by(|a, b| a.due_date.cmp(&b.due_date));
    Ok(tasks
        .into_iter()
        .take(limit)
        .map(|t| RecordListRow {
            entity_type: "Task".into(),
            entity_id: t.id,
            title: t.title,
            subtitle: t.due_date.map(|d| format!("Due {d}")),
        })
        .collect())
}

fn due_soon_invoices(conn: &Connection, workspace_id: &str, limit: usize) -> AppResult<Vec<RecordListRow>> {
    let mut invoices: Vec<_> = invoice_repo::list(conn, workspace_id)?
        .into_iter()
        .filter(|inv| inv.due_date.is_some() && inv.status != "Paid" && inv.status != "Void" && inv.status != "Cancelled")
        .collect();
    invoices.sort_by(|a, b| a.due_date.cmp(&b.due_date));
    Ok(invoices
        .into_iter()
        .take(limit)
        .map(|inv| RecordListRow {
            entity_type: "Invoice".into(),
            entity_id: inv.id,
            title: inv.invoice_number,
            subtitle: inv.due_date.map(|d| format!("Due {d}")),
        })
        .collect())
}

/// (id, title, created_at) for every non-archived record of `entity_type` -
/// the shared shape "recent" mode sorts and truncates the same way for
/// every entity, built-in or custom.
fn recent_rows(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<Vec<(String, String, String)>> {
    Ok(match entity_type {
        "Company" => company_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.name, r.created_at)).collect(),
        "Contact" => contact_repo::list(conn, workspace_id)?
            .into_iter()
            .map(|r| (r.id, format!("{} {}", r.first_name, r.last_name).trim().to_string(), r.created_at))
            .collect(),
        "Opportunity" => opportunity_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.name, r.created_at)).collect(),
        "Quote" => quote_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.quote_number, r.created_at)).collect(),
        "Order" => order_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.order_number, r.created_at)).collect(),
        "Invoice" => invoice_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.invoice_number, r.created_at)).collect(),
        "Contract" => contract_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.title, r.created_at)).collect(),
        "Task" => task_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.title, r.created_at)).collect(),
        "Product" => product_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.name, r.created_at)).collect(),
        other => custom_record_repo::list(conn, workspace_id, other)?
            .into_iter()
            .map(|r| (r.id, r.primary_name, r.created_at))
            .collect(),
    })
}

/// The rows a record-list dashboard widget renders. `limit` is clamped to
/// `[1, MAX_ROWS]` regardless of what the widget's own config asks for.
pub fn run(conn: &Connection, workspace_id: &str, entity_type: &str, mode: &str, limit: i64) -> AppResult<Vec<RecordListRow>> {
    let limit = (limit.max(1) as usize).min(MAX_ROWS);

    if mode == "due_soon" {
        match entity_type {
            "Task" => return due_soon_tasks(conn, workspace_id, limit),
            "Invoice" => return due_soon_invoices(conn, workspace_id, limit),
            _ => {} // falls through to "recent" below - no due date to sort by
        }
    }

    let mut rows = recent_rows(conn, workspace_id, entity_type)?;
    rows.sort_by(|a, b| b.2.cmp(&a.2)); // newest created_at first
    Ok(rows
        .into_iter()
        .take(limit)
        .map(|(id, title, created_at)| RecordListRow {
            entity_type: entity_type.to_string(),
            entity_id: id,
            title,
            subtitle: created_at.split('T').next().map(|d| d.to_string()),
        })
        .collect())
}
