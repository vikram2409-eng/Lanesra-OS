//! Raw CRUD for `sign_in_lockouts` (migration 0089): brute-force state per
//! (workspace, identifier), where the identifier is "user:<name>" or
//! "addr:<ip>". The decision logic lives in
//! `services::sign_in_throttle_service`; this is storage only. Times are
//! epoch-millisecond integers so the throttle is testable with an injected
//! clock (as the tool-call rate limiter does).

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;

#[derive(Debug, Clone)]
pub struct LockoutRow {
    pub failed_count: i64,
    pub first_failed_at_ms: i64,
    pub locked_until_ms: Option<i64>,
}

pub fn get(
    conn: &Connection,
    workspace_id: &str,
    identifier: &str,
) -> rusqlite::Result<Option<LockoutRow>> {
    conn.query_row(
        "SELECT failed_count, first_failed_at_ms, locked_until_ms
           FROM sign_in_lockouts WHERE workspace_id = ?1 AND identifier = ?2",
        rusqlite::params![workspace_id, identifier],
        |r| {
            Ok(LockoutRow {
                failed_count: r.get(0)?,
                first_failed_at_ms: r.get(1)?,
                locked_until_ms: r.get(2)?,
            })
        },
    )
    .optional()
}

pub fn upsert(
    conn: &Connection,
    workspace_id: &str,
    identifier: &str,
    failed_count: i64,
    first_failed_at_ms: i64,
    locked_until_ms: Option<i64>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO sign_in_lockouts
            (workspace_id, identifier, failed_count, first_failed_at_ms, locked_until_ms, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (workspace_id, identifier) DO UPDATE SET
            failed_count = excluded.failed_count,
            first_failed_at_ms = excluded.first_failed_at_ms,
            locked_until_ms = excluded.locked_until_ms,
            updated_at = excluded.updated_at",
        rusqlite::params![
            workspace_id,
            identifier,
            failed_count,
            first_failed_at_ms,
            locked_until_ms,
            now_iso(),
        ],
    )?;
    Ok(())
}

pub fn clear(conn: &Connection, workspace_id: &str, identifier: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM sign_in_lockouts WHERE workspace_id = ?1 AND identifier = ?2",
        rusqlite::params![workspace_id, identifier],
    )?;
    Ok(())
}
