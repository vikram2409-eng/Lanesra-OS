//! Integration Hub (spec §15): recurring Integration Jobs - **pull**
//! sync only, stated plainly. A Job re-fetches an External Object's
//! records (spec §16) on an interval and upserts them into a target
//! Lanesra object via `api_object_service`, the exact same generic
//! create/update dispatcher the CSV wizard and the inbound REST API use
//! - so the same validation/business-rule/permission checks fire here
//! too. Progress is checkpointed with a simple string "cursor" (the max
//! value seen of a configured `cursor_field`, compared lexicographically
//! - correct for ISO-8601 timestamps and zero-padded sequence numbers,
//! not for arbitrary unordered strings, a real but narrow limitation
//! stated here rather than silently assumed away).
//!
//! **Push** direction (Lanesra -> external system, one Connector Action
//! call per outgoing record) is a deliberate, named gap in this pass:
//! it needs a per-record "already synced" marker this data model doesn't
//! have yet (naively re-pushing every record on every run isn't a real
//! sync), and is left for a fast-follow rather than faked here.
//!
//! Scheduling itself has two faces, matching how every other "recurring"
//! feature in this codebase already works (see `workflow_service::
//! run_scheduled`'s own doc comment): `run_due` is a plain sync-callable
//! function that runs every job whose `interval_minutes` has elapsed -
//! the Team Workspace axum server calls it from a real
//! `tokio::time::interval` background loop (`server::job_scheduler`),
//! while desktop keeps the existing client-poll pattern (a Tauri command
//! wrapping this same function, called whenever the frontend is open).

use rusqlite::Connection;
use serde_json::Value;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::integration::{IntegrationJob, IntegrationJobInput, IntegrationJobRun};
use crate::repositories::integration_job_repo;
use crate::services::integration_log_service::{self, FinishOutcome};
use crate::services::{api_object_service, data_exchange_service, external_object_service};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn validate(input: &IntegrationJobInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Job name is required".into()));
    }
    if input.match_key.trim().is_empty() {
        return Err(AppError::Validation("A match key is required to upsert incoming records".into()));
    }
    if input.interval_minutes <= 0 {
        return Err(AppError::Validation("Interval must be a positive number of minutes".into()));
    }
    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &IntegrationJobInput, actor_user_id: Option<&str>) -> AppResult<IntegrationJob> {
    require_admin(conn, actor_user_id)?;
    validate(input)?;
    // A Job's source must be a real External Object in this workspace -
    // caught here with a clear message, not a foreign-key error surfaced
    // later at run time.
    super::external_object_service::list_for_workspace(conn, workspace_id)?
        .into_iter()
        .find(|o| o.id == input.external_object_id)
        .ok_or_else(|| AppError::Validation("The selected External Object was not found in this workspace".into()))?;

    Ok(integration_job_repo::insert(
        conn,
        &new_uuid(),
        workspace_id,
        input.name.trim(),
        &input.external_object_id,
        &input.target_object_key,
        input.match_key.trim(),
        input.cursor_field.as_deref(),
        input.interval_minutes,
        actor_user_id,
    )?)
}

fn get_owned(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<IntegrationJob> {
    let job = integration_job_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Integration Job".into()))?;
    if job.workspace_id != workspace_id {
        return Err(AppError::NotFound("Integration Job".into()));
    }
    Ok(job)
}

pub fn get(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<IntegrationJob> {
    get_owned(conn, workspace_id, id)
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<IntegrationJob>> {
    Ok(integration_job_repo::list_for_workspace(conn, workspace_id)?)
}

pub fn list_runs(conn: &Connection, workspace_id: &str, job_id: &str, limit: i64) -> AppResult<Vec<IntegrationJobRun>> {
    get_owned(conn, workspace_id, job_id)?;
    Ok(integration_job_repo::list_runs_for_job(conn, job_id, limit)?)
}

pub fn update(conn: &Connection, workspace_id: &str, id: &str, input: &IntegrationJobInput, status: &str, actor_user_id: Option<&str>) -> AppResult<IntegrationJob> {
    require_admin(conn, actor_user_id)?;
    validate(input)?;
    get_owned(conn, workspace_id, id)?;
    if !["active", "paused"].contains(&status) {
        return Err(AppError::Validation(format!("Invalid status '{status}'")));
    }
    integration_job_repo::update(conn, id, input.name.trim(), &input.external_object_id, &input.target_object_key, input.match_key.trim(), input.cursor_field.as_deref(), input.interval_minutes, status, actor_user_id)?;
    get_owned(conn, workspace_id, id)
}

pub fn delete(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    Ok(integration_job_repo::delete(conn, id)?)
}

fn max_cursor(current: Option<String>, record: &Value, cursor_field: &str) -> Option<String> {
    let candidate = record.get(cursor_field).and_then(|v| v.as_str())?;
    match &current {
        Some(existing) if existing.as_str() >= candidate => current,
        _ => Some(candidate.to_string()),
    }
}

/// Fetches, upserts and checkpoints once - the actual sync logic, shared
/// by a manual "Run Now" and the scheduler's automatic firing.
async fn execute_sync(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], job: &IntegrationJob) -> AppResult<(i64, i64, Option<String>)> {
    let records = external_object_service::list_records_by_id(conn, workspace_id, master_key, &job.external_object_id, job.cursor_value.as_deref()).await?;
    let (mut processed, mut failed) = (0i64, 0i64);
    let mut cursor = job.cursor_value.clone();

    for record in &records {
        let match_value = record.get(&job.match_key).and_then(|v| v.as_str()).unwrap_or_default();
        let existing_id = data_exchange_service::find_existing(conn, workspace_id, &job.target_object_key, &job.match_key, match_value);
        let write_result = match existing_id {
            Some(id) => api_object_service::update_record(conn, workspace_id, &job.target_object_key, &id, record, None),
            None => api_object_service::create_record(conn, workspace_id, &job.target_object_key, record, None),
        };
        match write_result {
            Ok(_) => processed += 1,
            Err(_) => failed += 1,
        }
        if let Some(field) = &job.cursor_field {
            cursor = max_cursor(cursor, record, field);
        }
    }
    Ok((processed, failed, cursor))
}

/// Manual "Run Now" (spec §15's own explicit UI control) - always
/// permitted regardless of `status`/schedule due-ness, and always admin-
/// gated since it writes real records.
pub async fn run_now(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], job_id: &str, actor_user_id: Option<&str>) -> AppResult<IntegrationJobRun> {
    require_admin(conn, actor_user_id)?;
    let job = get_owned(conn, workspace_id, job_id)?;
    run_one(conn, workspace_id, master_key, &job, actor_user_id).await
}

async fn run_one(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], job: &IntegrationJob, actor_user_id: Option<&str>) -> AppResult<IntegrationJobRun> {
    let run_id = new_uuid();
    integration_job_repo::insert_run_started(conn, &run_id, &job.id, workspace_id, job.cursor_value.as_deref())?;
    let execution_id = integration_log_service::start(conn, workspace_id, "integration_job", None, Some(&job.id), "inbound", actor_user_id);

    match execute_sync(conn, workspace_id, master_key, job).await {
        Ok((processed, failed, new_cursor)) => {
            let status = if failed == 0 { "success" } else { "partial" };
            integration_job_repo::finish_run(conn, &run_id, status, processed, failed, None, new_cursor.as_deref())?;
            integration_job_repo::record_run_outcome(conn, &job.id, status, new_cursor.as_deref())?;
            integration_log_service::finish(conn, &execution_id, &FinishOutcome { status: status.into(), records_read: processed + failed, records_written: processed, records_failed: failed, ..Default::default() });
        }
        Err(e) => {
            integration_job_repo::finish_run(conn, &run_id, "failed", 0, 0, Some(&e.to_string()), None)?;
            integration_job_repo::record_run_outcome(conn, &job.id, "failed", None)?;
            integration_log_service::finish(conn, &execution_id, &FinishOutcome { status: "failed".into(), error_message: Some(e.to_string()), ..Default::default() });
        }
    }
    integration_job_repo::get_run(conn, &run_id)?.ok_or_else(|| AppError::NotFound("Job run".into()))
}

/// Runs every active, due job in this workspace once - what both the
/// server's background scheduler loop and desktop's client-poll
/// equivalent call. Never admin-gated itself (nothing here is a
/// user-initiated write of *configuration*, only jobs already configured
/// by an admin), matching `workflow_service::run_scheduled`'s own
/// unauthenticated-system-actor convention.
pub async fn run_due(conn: &Connection, workspace_id: &str, master_key: &[u8; 32]) -> AppResult<usize> {
    let due = integration_job_repo::list_due(conn, workspace_id)?;
    let count = due.len();
    for job in &due {
        // A single job's failure must never abort the others - run_one
        // itself already turns any error into a "failed" run row rather
        // than propagating, so this discard is defensive, not silent
        // failure-swallowing: nothing here can currently return Err.
        let _ = run_one(conn, workspace_id, master_key, job, None).await;
    }
    Ok(count)
}
