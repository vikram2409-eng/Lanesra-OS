//! WP-0141: idempotent side effects for graph runs (REQ-TOOL-005,
//! REQ-GOV-023; ADR-0007 §5).
//!
//! Every write a run makes is keyed by where it happened - the run, the node
//! and the attempt - plus a `write_key` that separates several writes of one
//! step (WP-0705; a single-write step uses `""`). `run_once` applies the
//! write only if that key has not been recorded, and records it in the same
//! transaction as the write, so:
//!   - a **retry** after a transient failure finds the key and does not write
//!     again (AC-2);
//!   - a **resume** after a crash either finds the key (the write committed)
//!     and skips it, or does not (it never committed) and applies it once -
//!     never a duplicate (AC-1);
//!   - a **double submit** of the same attempt writes once and the second
//!     caller gets the first result (AC-3).
//!
//! The strong guarantee is for writes made on the run's own SQLite
//! connection (native record writes), where the ledger row and the write
//! commit together. The derived key is also the idempotency token a
//! connector or MCP write passes to an external API that supports one.

use rusqlite::Connection;
use serde_json::Value;

use crate::domain::{AppError, AppResult};
use crate::repositories::run_write_ledger_repo;

/// The idempotency key for one step attempt: stable across a retry or resume
/// of the same attempt, different for a new attempt. Threaded to a connector
/// or MCP tool as the token an idempotent external API expects.
pub fn step_idempotency_key(run_id: &str, node_id: &str, attempt: i64) -> String {
    format!("{run_id}:{node_id}:{attempt}")
}

/// Applies `f` exactly once for the key (run, node, attempt, write_key). If
/// the key is already recorded, `f` is not called and the stored result is
/// returned. Otherwise `f` runs and its result is recorded in the same
/// transaction, so the write and its ledger row commit together.
///
/// `f` must do its writes on `conn` (the transaction is open on it). A `f`
/// that returns `Err` leaves nothing behind (the transaction is rolled back).
pub fn run_once<F>(
    conn: &Connection,
    workspace_id: &str,
    run_id: &str,
    node_id: &str,
    attempt: i64,
    write_key: &str,
    f: F,
) -> AppResult<Value>
where
    F: FnOnce() -> AppResult<Value>,
{
    if let Some(stored) = run_write_ledger_repo::find(conn, run_id, node_id, attempt, write_key)? {
        return Ok(stored
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(Value::Null));
    }
    let tx = conn.unchecked_transaction()?;
    let result = f()?;
    let result_json = serde_json::to_string(&result).ok();
    match run_write_ledger_repo::insert(
        conn,
        workspace_id,
        run_id,
        node_id,
        attempt,
        write_key,
        result_json.as_deref(),
    ) {
        Ok(()) => {
            tx.commit()?;
            Ok(result)
        }
        Err(rusqlite::Error::SqliteFailure(e, _))
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            // Another connection applied this key between our `find` and our
            // `insert` (a double submit from two workers). Roll back our own
            // write and return the result the winner recorded.
            drop(tx);
            let stored = run_write_ledger_repo::find(conn, run_id, node_id, attempt, write_key)?
                .ok_or_else(|| {
                    AppError::Conflict("idempotent write raced and left no ledger row".into())
                })?;
            Ok(stored
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(Value::Null))
        }
        Err(e) => Err(e.into()),
    }
}

/// Whether a write has already been recorded for this key - a read-only
/// check used where the caller wants to branch without applying anything.
pub fn already_done(
    conn: &Connection,
    run_id: &str,
    node_id: &str,
    attempt: i64,
    write_key: &str,
) -> AppResult<bool> {
    Ok(run_write_ledger_repo::find(conn, run_id, node_id, attempt, write_key)?.is_some())
}
