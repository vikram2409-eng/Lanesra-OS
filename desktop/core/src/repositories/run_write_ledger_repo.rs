//! WP-0141: the idempotent write ledger for graph runs (migration 0096).
//! A side effect is recorded under its idempotency key (run, node, attempt,
//! write_key); `find` returns the stored result when the key is already
//! present, and `insert` records a new one. The caller
//! (`idempotency_service`) writes the ledger row in the same transaction as
//! the side effect, so the two commit together.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};

/// The stored result for an idempotency key, or `None` when the key has not
/// been seen. `Some(None)` means the key is recorded but its write returned
/// no result; `None` means the write has not happened.
pub fn find(
    conn: &Connection,
    run_id: &str,
    node_id: &str,
    attempt: i64,
    write_key: &str,
) -> rusqlite::Result<Option<Option<String>>> {
    conn.query_row(
        "SELECT result_json FROM ai_run_write_ledger
          WHERE run_id = ?1 AND node_id = ?2 AND attempt = ?3 AND write_key = ?4",
        rusqlite::params![run_id, node_id, attempt, write_key],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()
}

/// Records that the side effect for this key has been applied, with its
/// result. Fails on the UNIQUE constraint if the key already exists - the
/// caller looks up `find` first, inside the same transaction.
pub fn insert(
    conn: &Connection,
    workspace_id: &str,
    run_id: &str,
    node_id: &str,
    attempt: i64,
    write_key: &str,
    result_json: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_run_write_ledger
            (id, workspace_id, run_id, node_id, attempt, write_key, result_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            new_uuid(),
            workspace_id,
            run_id,
            node_id,
            attempt,
            write_key,
            result_json,
            now_iso(),
        ],
    )?;
    Ok(())
}
