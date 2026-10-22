//! WP-0110: the one guard every run status change goes through (ADR-0007 §3,
//! §4). Nothing else writes `ai_runs.status`.
//!
//! - `is_legal` is the transition table. Nothing leaves a terminal status;
//!   replay and fork make a new run instead (WP-0207).
//! - `transition` moves a run only if it still holds the `from` status, so two
//!   workers cannot both move it, and records why it stopped.
//! - A run that becomes `running` takes a worker lease, renewed at every
//!   checkpoint and dropped when it leaves `running` (ADR-0007 §5). The sweep
//!   that finds expired leases is WP-0141.
//!
//! Run events for each change (ADR-0008) come with WP-0112.
//!
//! Covers: REQ-CORE-008 (WP-0110).

use chrono::{Duration, Utc};
use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::domain::{AppError, AppResult};
use crate::models::graph_run::{is_terminal_status, is_waiting_status, RUN_STATUSES, STOP_REASONS};

/// How long a worker's lease on a `running` run lasts without renewal.
pub const LEASE_SECONDS: i64 = 120;

/// Whether ADR-0007 §3 allows a run to move from `from` to `to`.
pub fn is_legal(from: &str, to: &str) -> bool {
    if !RUN_STATUSES.contains(&from) || !RUN_STATUSES.contains(&to) || is_terminal_status(from) {
        return false;
    }
    match (from, to) {
        ("queued", "running" | "cancelled") => true,
        (
            "running",
            "waiting_approval" | "waiting_budget" | "waiting_model" | "waiting_scheduled"
            | "paused" | "completed" | "completed_partial" | "failed" | "cancelled",
        ) => true,
        ("waiting_approval", "failed") => true,
        (from, "queued" | "cancelled") => is_waiting_status(from),
        _ => false,
    }
}

/// This process, as the holder of a run's lease.
pub fn lease_owner() -> String {
    format!("pid:{}", std::process::id())
}

fn lease_expiry() -> String {
    (Utc::now() + Duration::seconds(LEASE_SECONDS)).to_rfc3339()
}

/// Moves run `run_id` from `from` to `to`, recording `stop_reason`.
///
/// Fails without changing anything when the move is not in the table, the
/// stop reason is unknown, or the run no longer holds `from` (another worker
/// moved it first, or it does not exist).
pub fn transition(
    conn: &Connection,
    run_id: &str,
    from: &str,
    to: &str,
    stop_reason: Option<&str>,
) -> AppResult<()> {
    if !is_legal(from, to) {
        return Err(AppError::Validation(format!(
            "A run cannot go from '{from}' to '{to}'"
        )));
    }
    if let Some(reason) = stop_reason {
        if !STOP_REASONS.contains(&reason) {
            return Err(AppError::Validation(format!(
                "Unknown stop reason '{reason}'"
            )));
        }
    }
    let running = to == "running";
    let changed = conn.execute(
        "UPDATE ai_runs
            SET status = ?1,
                stop_reason = ?2,
                finished_at = CASE WHEN ?3 THEN ?4 ELSE finished_at END,
                lease_owner = ?5,
                lease_expires_at = ?6,
                pending_approval_id = CASE WHEN ?1 = 'waiting_approval' THEN pending_approval_id ELSE NULL END,
                resume_at = CASE WHEN ?1 = 'waiting_scheduled' THEN resume_at ELSE NULL END
          WHERE id = ?7 AND status = ?8",
        rusqlite::params![
            to,
            stop_reason,
            is_terminal_status(to),
            now_iso(),
            running.then(lease_owner),
            running.then(lease_expiry),
            run_id,
            from,
        ],
    )?;
    if changed == 0 {
        return Err(AppError::Conflict(format!(
            "This run is no longer '{from}'"
        )));
    }
    Ok(())
}

/// Extends this worker's lease on a `running` run. Called at every checkpoint.
pub fn renew_lease(conn: &Connection, run_id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE ai_runs SET lease_owner = ?1, lease_expires_at = ?2 WHERE id = ?3 AND status = 'running'",
        rusqlite::params![lease_owner(), lease_expiry(), run_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_leaves_a_terminal_status() {
        for from in ["completed", "completed_partial", "failed", "cancelled"] {
            for to in RUN_STATUSES {
                assert!(!is_legal(from, to), "{from} -> {to}");
            }
        }
    }

    #[test]
    fn unknown_statuses_are_never_legal() {
        assert!(!is_legal("waiting_tool", "queued"));
        assert!(!is_legal("running", "waiting_agent"));
    }
}
