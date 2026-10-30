//! WP-0112: live run events (ADR-0008 §2, §3, §5).
//!
//! A run's events are written by `trace_service` as its spans open and
//! close (`run.started`, `step.*`, `round.completed`, `tool.*`,
//! `approval.requested`, `run.completed`, `run.failed`), each with the next
//! gapless `seq` of its run, stored in `ai_run_events` and then published
//! on this in-process bus. Runs write outside any transaction, so a stored
//! event is already committed when it is published. Nothing polls a table
//! for live events.
//!
//! A reader subscribes to the bus first and then replays stored events,
//! dropping anything it has already seen by `(run_id, seq)`, so no event
//! falls between the two. Replay reads the table, never memory, so it
//! survives a closed tab and a server restart.
//!
//! **Who sees a run's events.** Runs do not record who started them yet
//! (WP-0100 adds record scope), and both run kinds are read by
//! Administrators only today (`graph_runtime_service::get_run`), so a
//! signed-in user must be an Administrator. An API client needs the
//! `events.read` scope, checked by the server before it calls in here.

use std::sync::{Arc, OnceLock};

use rusqlite::Connection;
use tokio::sync::broadcast;

use crate::domain::{AppError, AppResult};
use crate::models::run_event::RunEvent;
use crate::repositories::run_event_repo;

/// Events a slow subscriber may fall behind by before it misses some; it
/// then replays from the table.
const BUS_CAPACITY: usize = 4096;

fn bus() -> &'static broadcast::Sender<Arc<RunEvent>> {
    static BUS: OnceLock<broadcast::Sender<Arc<RunEvent>>> = OnceLock::new();
    BUS.get_or_init(|| broadcast::channel(BUS_CAPACITY).0)
}

/// Every event published from now on, in publish order. A receiver that
/// lags gets `RecvError::Lagged` and should replay from the table.
pub fn subscribe() -> broadcast::Receiver<Arc<RunEvent>> {
    bus().subscribe()
}

/// Stores `event` and publishes it. Returns false when it could not be
/// stored, in which case nothing is published and its `seq` stays free.
pub(crate) fn publish(conn: &Connection, mut event: RunEvent) -> bool {
    match run_event_repo::insert(conn, &event) {
        Ok(cursor) => {
            event.cursor = cursor;
            // No subscriber is not an error.
            let _ = bus().send(Arc::new(event));
            true
        }
        Err(_) => false,
    }
}

/// The signed-in user may read run events: an Administrator.
pub fn require_viewer(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
        .map_err(|_| AppError::PolicyBlocked("Only an Administrator can see a run's events".into()))
}

/// The run's kind, or `NotFound` when this workspace has no such run.
pub fn run_kind(conn: &Connection, workspace_id: &str, run_id: &str) -> AppResult<&'static str> {
    run_event_repo::run_kind_of(conn, workspace_id, run_id)?
        .ok_or_else(|| AppError::NotFound("Run".into()))
}

/// A run's stored events after `after_seq`, for a caller already allowed
/// to see them (`require_viewer`, or an API client with `events.read`).
pub fn events_for_run(
    conn: &Connection,
    workspace_id: &str,
    run_id: &str,
    after_seq: i64,
) -> AppResult<Vec<RunEvent>> {
    run_kind(conn, workspace_id, run_id)?;
    Ok(run_event_repo::list_for_run(
        conn,
        workspace_id,
        run_id,
        after_seq,
    )?)
}

/// Stored events of several runs after the cross-run cursor `after_id`.
pub fn events_after_cursor(
    conn: &Connection,
    workspace_id: &str,
    run_ids: &[String],
    after_id: i64,
) -> AppResult<Vec<RunEvent>> {
    Ok(run_event_repo::list_after_cursor(
        conn,
        workspace_id,
        run_ids,
        after_id,
    )?)
}

/// `list_run_events` for the desktop and the Team Workspace UI: the
/// signed-in user's view of one run's events after `after_seq`.
pub fn list_run_events(
    conn: &Connection,
    workspace_id: &str,
    run_id: &str,
    after_seq: i64,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<RunEvent>> {
    require_viewer(conn, actor_user_id)?;
    events_for_run(conn, workspace_id, run_id, after_seq)
}
