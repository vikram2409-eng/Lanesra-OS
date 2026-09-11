//! Integration Hub (spec §15): the one real OS-level recurring
//! background loop anywhere in this codebase - the Team Workspace axum
//! server is the one long-running process that exists here, so this is
//! where the "server scheduler" half of Integration Jobs
//! (`integration_job_service::run_due`) belongs; desktop keeps the same
//! client-poll pattern already used for scheduled Workflow Automation
//! instead (see `workflow_service::run_scheduled`'s own doc comment for
//! why Personal Workspace never gets an OS-level scheduler).
//!
//! Also the one place two other queues get drained, both following the
//! same "enqueue now, run for real later" shape Integration Jobs itself
//! doesn't need but two *workflow actions* do (a record save must never
//! block on a real outbound call):
//! `connector_execution_service::drain_pending_actions` (Workflow
//! Automation's `call_connector_action` - present since migration 0033,
//! but never actually wired into any poll loop until now, so it's been
//! silently non-functional; an incidental fix, not new scope) and
//! `ai_orchestration_service::drain_pending_runs` (AI & Agentic Layer,
//! Phase 6b's `run_ai_agent` workflow action, plus schedule/webhook
//! Triggers - `enqueue_due_schedules` is what actually enqueues a due
//! schedule Trigger, right before the drain that runs it) and Phase 7g's
//! `vector_search_service::drain_pending_embeddings` (a Custom Object
//! record's own migration-0048 triggers enqueue it, this is where the
//! real embedding-provider call happens).
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

use lanesra_core::services::{ai_orchestration_service, connector_execution_service, integration_job_service, secret_service, vector_search_service};

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

    // Each step runs independently - one queue's failure must never
    // block the others, same "log and move on" resilience
    // `integration_job_service::run_due` itself already gives each
    // individual due job.
    if let Err(e) = integration_job_service::run_due(conn, &workspace.id, &master_key).await {
        tracing::error!(error = %e, "Integration Jobs run_due failed");
    }
    if let Err(e) = connector_execution_service::drain_pending_actions(conn, &workspace.id, &master_key, 50).await {
        tracing::error!(error = %e, "drain_pending_actions (call_connector_action) failed");
    }
    if let Err(e) = ai_orchestration_service::enqueue_due_schedules(conn, &workspace.id) {
        tracing::error!(error = %e, "enqueue_due_schedules (AI Agent Foundry schedule triggers) failed");
    }
    if let Err(e) = ai_orchestration_service::drain_pending_runs(conn, &workspace.id, &master_key, 50).await {
        tracing::error!(error = %e, "drain_pending_runs (AI Agent Foundry) failed");
    }
    // AI & Agentic Layer, Phase 7g: same "enqueue now, drain later" shape
    // as drain_pending_runs above - a workspace with no provider key
    // configured yet logs here every tick rather than reindexing, the
    // same "log and move on" resilience every other step in this
    // function already has for its own not-yet-configured case.
    if let Err(e) = vector_search_service::drain_pending_embeddings(conn, &workspace.id, &master_key, 50).await {
        tracing::error!(error = %e, "drain_pending_embeddings (vector search) failed");
    }
    Ok(())
}
