//! Integration Hub (spec §15): the one real OS-level recurring
//! background loop anywhere in this codebase - the Team Workspace axum
//! server is the one long-running process that exists here, so this is
//! where the "server scheduler" half of Integration Jobs
//! (`integration_job_service::run_due`) belongs; desktop keeps the same
//! client-poll pattern already used for scheduled Workflow Automation
//! instead (see `workflow_service::run_scheduled`'s own doc comment for
//! why Personal Workspace never gets an OS-level scheduler).
//!
//! Runs on its **own dedicated OS thread with its own single-threaded
//! Tokio runtime**, not `tokio::spawn`ed onto axum's shared
//! multi-threaded one - a real bug caught only by actually compiling
//! this (not by inspection): a Job's sync (`execute_sync`) makes a real
//! outbound HTTP call while needing its `&rusqlite::Connection` to stay
//! valid across that `.await`, but `Connection` isn't `Sync`, so `&Connection`
//! isn't `Send` - and `tokio::spawn` requires the whole future to be
//! `Send + 'static`. `Runtime::block_on` on a dedicated thread has no
//! such requirement (nothing here is ever moved to another thread), so
//! this sidesteps the problem entirely rather than fighting it.
//!
//! Opens its **own** SQLite connection to the same file rather than
//! sharing `ServerState.conn` (a `std::sync::Mutex`, whose guard has the
//! exact same non-`Send` problem, on top of being the wrong tool here
//! regardless: nothing else ever touches this connection, so it needs no
//! locking at all). SQLite's WAL mode plus the `busy_timeout`
//! `open_workspace_db` sets on both connections is what makes two
//! connections to the same file safe under the occasional concurrent
//! write.

use std::path::PathBuf;
use std::time::Duration;

use lanesra_core::services::{integration_job_service, secret_service};

/// Spawns the scheduler loop on its own OS thread and returns
/// immediately - call once from `main`, after the primary workspace
/// database file already exists. `tick_interval` is how often it checks
/// for due jobs (production: 60s; a test can pass something far shorter
/// to prove firing without a real wait).
pub fn spawn(db_path: PathBuf, key_file_path: PathBuf, tick_interval: Duration) {
    std::thread::Builder::new()
        .name("integration-jobs-scheduler".into())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::error!(error = %e, "Integration Jobs scheduler could not start its own Tokio runtime - disabled for this run");
                    return;
                }
            };
            rt.block_on(run_loop(db_path, key_file_path, tick_interval));
        })
        .expect("failed to spawn the Integration Jobs scheduler thread");
}

async fn run_loop(db_path: PathBuf, key_file_path: PathBuf, tick_interval: Duration) {
    let conn = match lanesra_core::db::open_workspace_db(&db_path) {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!(error = %e, path = %db_path.display(), "Integration Jobs scheduler could not open its own database connection - disabled for this run");
            return;
        }
    };
    let mut ticker = tokio::time::interval(tick_interval);
    loop {
        ticker.tick().await;
        if let Err(e) = tick(&conn, &key_file_path).await {
            tracing::error!(error = %e, "Integration Jobs scheduler tick failed");
        }
    }
}

async fn tick(conn: &rusqlite::Connection, key_file_path: &std::path::Path) -> Result<(), String> {
    let Some(workspace) = lanesra_core::repositories::workspace_repo::get_current(conn).map_err(|e| e.to_string())? else {
        // No workspace set up yet (first_run_setup not completed) -
        // nothing to schedule, not an error.
        return Ok(());
    };
    let master_key = secret_service::resolve_master_key(key_file_path).map_err(|e| e.to_string())?;
    integration_job_service::run_due(conn, &workspace.id, &master_key).await.map_err(|e| e.to_string())?;
    Ok(())
}
