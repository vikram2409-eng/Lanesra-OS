//! Raw CRUD for `ai_agent_runs`/`ai_agent_run_steps` (migration 0040) -
//! the unified run log a manual Run, a drained schedule/webhook/workflow
//! trigger, and an inline webhook-trigger call all write through. See
//! `services::ai_orchestration_service::run`/`drain_pending_runs`.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::ai_agent_pipeline::{AiAgentRun, AiAgentRunStep};

fn map_run_row(row: &rusqlite::Row) -> rusqlite::Result<AiAgentRun> {
    Ok(AiAgentRun {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        target_type: row.get("target_type")?,
        target_id: row.get("target_id")?,
        status: row.get("status")?,
        error: row.get("error")?,
        triggered_by: row.get("triggered_by")?,
        source_entity_type: row.get("source_entity_type")?,
        source_entity_id: row.get("source_entity_id")?,
        started_at: row.get("started_at")?,
        finished_at: row.get("finished_at")?,
        steps: Vec::new(), // filled in by `hydrate`
    })
}

fn map_step_row(row: &rusqlite::Row) -> rusqlite::Result<AiAgentRunStep> {
    Ok(AiAgentRunStep {
        id: row.get("id")?,
        run_id: row.get("run_id")?,
        agent_id: row.get("agent_id")?,
        step_order: row.get("step_order")?,
        input_text: row.get("input_text")?,
        output_text: row.get("output_text")?,
        error: row.get("error")?,
        tool_calls_count: row.get("tool_calls_count")?,
    })
}

pub fn list_run_steps(conn: &Connection, run_id: &str) -> rusqlite::Result<Vec<AiAgentRunStep>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_agent_run_steps WHERE run_id = ?1 ORDER BY step_order")?;
    let rows = stmt.query_map([run_id], map_step_row)?.collect();
    rows
}

fn hydrate(conn: &Connection, mut run: AiAgentRun) -> rusqlite::Result<AiAgentRun> {
    run.steps = list_run_steps(conn, &run.id)?;
    Ok(run)
}

#[allow(clippy::too_many_arguments)]
pub fn start_run(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    target_type: &str,
    target_id: &str,
    triggered_by: Option<&str>,
    source_entity_type: Option<&str>,
    source_entity_id: Option<&str>,
) -> rusqlite::Result<()> {
    // 'failed' is the placeholder status until `finish_run` overwrites it -
    // fail-closed, so a row that's interrupted before finishing (a panic,
    // a process restart mid-run) reads as failed rather than falsely
    // succeeded.
    conn.execute(
        "INSERT INTO ai_agent_runs (id, workspace_id, target_type, target_id, status, triggered_by, source_entity_type, source_entity_id, started_at)
         VALUES (?1, ?2, ?3, ?4, 'failed', ?5, ?6, ?7, ?8)",
        (id, workspace_id, target_type, target_id, triggered_by, source_entity_type, source_entity_id, now_iso()),
    )?;
    Ok(())
}

pub fn append_run_step(conn: &Connection, run_id: &str, agent_id: &str, step_order: i64, input_text: &str, output_text: Option<&str>, error: Option<&str>, tool_calls_count: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_agent_run_steps (id, run_id, agent_id, step_order, input_text, output_text, error, tool_calls_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (crate::domain::ids::new_uuid(), run_id, agent_id, step_order, input_text, output_text, error, tool_calls_count),
    )?;
    Ok(())
}

pub fn finish_run(conn: &Connection, id: &str, status: &str, error: Option<&str>) -> rusqlite::Result<()> {
    conn.execute("UPDATE ai_agent_runs SET status = ?1, error = ?2, finished_at = ?3 WHERE id = ?4", (status, error, now_iso(), id))?;
    Ok(())
}

pub fn get_run(conn: &Connection, id: &str) -> rusqlite::Result<Option<AiAgentRun>> {
    let row = conn.query_row("SELECT * FROM ai_agent_runs WHERE id = ?1", [id], map_run_row).optional()?;
    row.map(|r| hydrate(conn, r)).transpose()
}

pub fn list_runs_for_target(conn: &Connection, target_type: &str, target_id: &str, limit: i64) -> rusqlite::Result<Vec<AiAgentRun>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_agent_runs WHERE target_type = ?1 AND target_id = ?2 ORDER BY started_at DESC LIMIT ?3")?;
    let rows: Vec<AiAgentRun> = stmt.query_map(rusqlite::params![target_type, target_id, limit], map_run_row)?.collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|r| hydrate(conn, r)).collect()
}
