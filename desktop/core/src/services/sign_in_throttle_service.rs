//! WP-0106: sign-in throttling and lockout. Failed sign-ins are counted per
//! account and per source address (migration 0089's `sign_in_lockouts`);
//! past a soft threshold each further failure imposes a growing delay, and
//! at the configured maximum the identifier is locked out for the
//! configured window. A lockout is an ADR-0009 `policy` error carrying a
//! `retry_after_ms`. Times are epoch-millisecond integers and every entry
//! point takes an explicit `now_ms`, so the whole thing is unit-testable
//! with an injected clock (as the tool-call rate limiter is).

use crate::domain::errors::ErrorClass;
use crate::domain::{AppError, AppResult};
use crate::models::security::SecuritySettings;
use crate::repositories::sign_in_lockouts_repo as repo;
use rusqlite::Connection;

/// Delays start after this many failures; below it, a wrong password just
/// fails with no delay.
const SOFT_THRESHOLD: i64 = 3;
/// The growing per-failure delay is capped here (ms) before the hard lockout.
const SOFT_DELAY_CAP_MS: i64 = 30_000;

pub fn user_identifier(username: &str) -> String {
    format!("user:{}", username.trim().to_lowercase())
}

pub fn addr_identifier(addr: &str) -> String {
    format!("addr:{addr}")
}

fn locked_error(retry_after_ms: i64) -> AppError {
    let secs = (retry_after_ms + 999) / 1000;
    AppError::Classified {
        class: ErrorClass::Policy,
        code: "policy.account_locked",
        message: format!("Too many sign-in attempts. Try again in {secs} second(s)."),
        retry_after_ms: Some(retry_after_ms.max(0) as u64),
        http_status: None,
    }
}

/// Refuses while `identifier` is locked. Call before checking a password.
pub fn check_at(
    conn: &Connection,
    workspace_id: &str,
    identifier: &str,
    now_ms: i64,
) -> AppResult<()> {
    if let Some(row) = repo::get(conn, workspace_id, identifier)? {
        if let Some(until) = row.locked_until_ms {
            if until > now_ms {
                return Err(locked_error(until - now_ms));
            }
        }
    }
    Ok(())
}

/// Records one failed attempt and returns the lockout end (ms) if this
/// failure locked the identifier, else None.
pub fn record_failure_at(
    conn: &Connection,
    workspace_id: &str,
    identifier: &str,
    settings: &SecuritySettings,
    now_ms: i64,
) -> AppResult<Option<i64>> {
    let existing = repo::get(conn, workspace_id, identifier)?;
    let failed_count = existing.as_ref().map(|r| r.failed_count).unwrap_or(0) + 1;
    let first_failed_at_ms = existing
        .as_ref()
        .map(|r| r.first_failed_at_ms)
        .unwrap_or(now_ms);

    let max = settings.lockout_max_failures.max(1);
    let locked_until_ms = if failed_count >= max {
        Some(now_ms + settings.lockout_minutes.max(0) * 60_000)
    } else if failed_count >= SOFT_THRESHOLD {
        let step = failed_count - SOFT_THRESHOLD; // 0, 1, 2, ...
        let delay = 1000i64
            .checked_shl(step.min(16) as u32)
            .unwrap_or(SOFT_DELAY_CAP_MS)
            .min(SOFT_DELAY_CAP_MS);
        Some(now_ms + delay)
    } else {
        None
    };

    repo::upsert(
        conn,
        workspace_id,
        identifier,
        failed_count,
        first_failed_at_ms,
        locked_until_ms,
    )?;

    // Only the hard lockout (reaching the max) is reported to the caller as
    // a "locked" event worth auditing; the soft delays are silent cooldowns.
    Ok(if failed_count >= max {
        locked_until_ms
    } else {
        None
    })
}

/// Clears all failure state for an identifier (on a successful sign-in).
/// Returns true if there was state to clear.
pub fn clear(conn: &Connection, workspace_id: &str, identifier: &str) -> AppResult<bool> {
    let had = repo::get(conn, workspace_id, identifier)?.is_some();
    repo::clear(conn, workspace_id, identifier)?;
    Ok(had)
}
