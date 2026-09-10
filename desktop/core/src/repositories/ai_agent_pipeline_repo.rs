//! Raw CRUD for `ai_agent_pipelines`/`ai_agent_pipeline_steps` and
//! `ai_agent_triggers` (migration 0040). See
//! `services::ai_orchestration_service` for validation/gating/execution.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::ai_agent_pipeline::{AiAgentPipeline, AiAgentPipelineInput, AiAgentTrigger, AiAgentTriggerInput, PipelineStep};

fn map_pipeline_row(row: &rusqlite::Row) -> rusqlite::Result<AiAgentPipeline> {
    Ok(AiAgentPipeline {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        steps: Vec::new(), // filled in by `hydrate`
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

fn map_step_row(row: &rusqlite::Row) -> rusqlite::Result<PipelineStep> {
    Ok(PipelineStep { id: row.get("id")?, agent_id: row.get("agent_id")?, step_order: row.get("step_order")?, input_template: row.get("input_template")? })
}

pub fn list_steps(conn: &Connection, pipeline_id: &str) -> rusqlite::Result<Vec<PipelineStep>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_agent_pipeline_steps WHERE pipeline_id = ?1 ORDER BY step_order")?;
    let rows = stmt.query_map([pipeline_id], map_step_row)?.collect();
    rows
}

fn hydrate(conn: &Connection, mut pipeline: AiAgentPipeline) -> rusqlite::Result<AiAgentPipeline> {
    pipeline.steps = list_steps(conn, &pipeline.id)?;
    Ok(pipeline)
}

fn replace_steps(conn: &Connection, pipeline_id: &str, steps: &[crate::models::ai_agent_pipeline::PipelineStepInput]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_agent_pipeline_steps WHERE pipeline_id = ?1", [pipeline_id])?;
    for (i, step) in steps.iter().enumerate() {
        conn.execute(
            "INSERT INTO ai_agent_pipeline_steps (id, pipeline_id, agent_id, step_order, input_template) VALUES (?1, ?2, ?3, ?4, ?5)",
            (crate::domain::ids::new_uuid(), pipeline_id, &step.agent_id, i as i64, &step.input_template),
        )?;
    }
    Ok(())
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &AiAgentPipelineInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiAgentPipeline> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_agent_pipelines (id, workspace_id, name, description, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6, ?5, ?6)",
        (id, workspace_id, &input.name, &input.description, &now, actor_user_id),
    )?;
    replace_steps(conn, id, &input.steps)?;
    get(conn, id).map(|p| p.expect("just inserted"))
}

pub fn update(conn: &Connection, id: &str, input: &AiAgentPipelineInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiAgentPipeline> {
    let now = now_iso();
    conn.execute(
        "UPDATE ai_agent_pipelines SET name = ?1, description = ?2, updated_at = ?3, updated_by = ?4 WHERE id = ?5",
        (&input.name, &input.description, &now, actor_user_id, id),
    )?;
    replace_steps(conn, id, &input.steps)?;
    get(conn, id).map(|p| p.expect("just updated"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<AiAgentPipeline>> {
    let row = conn.query_row("SELECT * FROM ai_agent_pipelines WHERE id = ?1", [id], map_pipeline_row).optional()?;
    row.map(|p| hydrate(conn, p)).transpose()
}

pub fn list(conn: &Connection, workspace_id: &str, active_only: bool) -> rusqlite::Result<Vec<AiAgentPipeline>> {
    let sql = if active_only {
        "SELECT * FROM ai_agent_pipelines WHERE workspace_id = ?1 AND is_active = 1 ORDER BY name"
    } else {
        "SELECT * FROM ai_agent_pipelines WHERE workspace_id = ?1 ORDER BY name"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows: Vec<AiAgentPipeline> = stmt.query_map([workspace_id], map_pipeline_row)?.collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|p| hydrate(conn, p)).collect()
}

pub fn set_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> rusqlite::Result<AiAgentPipeline> {
    let now = now_iso();
    conn.execute("UPDATE ai_agent_pipelines SET is_active = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4", (is_active, &now, actor_user_id, id))?;
    get(conn, id).map(|p| p.expect("just updated"))
}

// --- Triggers ---------------------------------------------------------

fn map_trigger_row(row: &rusqlite::Row) -> rusqlite::Result<AiAgentTrigger> {
    Ok(AiAgentTrigger {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        target_type: row.get("target_type")?,
        target_id: row.get("target_id")?,
        trigger_type: row.get("trigger_type")?,
        interval_minutes: row.get("interval_minutes")?,
        is_active: row.get("is_active")?,
        last_run_at: row.get("last_run_at")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
    })
}

pub fn create_trigger(conn: &Connection, id: &str, workspace_id: &str, input: &AiAgentTriggerInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiAgentTrigger> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_agent_triggers (id, workspace_id, target_type, target_id, trigger_type, interval_minutes, is_active, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8)",
        (id, workspace_id, &input.target_type, &input.target_id, &input.trigger_type, input.interval_minutes, &now, actor_user_id),
    )?;
    get_trigger(conn, id).map(|t| t.expect("just inserted"))
}

pub fn get_trigger(conn: &Connection, id: &str) -> rusqlite::Result<Option<AiAgentTrigger>> {
    conn.query_row("SELECT * FROM ai_agent_triggers WHERE id = ?1", [id], map_trigger_row).optional()
}

pub fn list_triggers_for_target(conn: &Connection, target_type: &str, target_id: &str) -> rusqlite::Result<Vec<AiAgentTrigger>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_agent_triggers WHERE target_type = ?1 AND target_id = ?2 ORDER BY created_at")?;
    let rows = stmt.query_map((target_type, target_id), map_trigger_row)?.collect();
    rows
}

pub fn set_trigger_active(conn: &Connection, id: &str, is_active: bool) -> rusqlite::Result<()> {
    conn.execute("UPDATE ai_agent_triggers SET is_active = ?1 WHERE id = ?2", (is_active, id))?;
    Ok(())
}

pub fn delete_trigger(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_agent_triggers WHERE id = ?1", [id])?;
    Ok(())
}

/// Schedule Triggers whose interval has elapsed since `last_run_at` (or
/// have never run) - same "due" convention `integration_job_repo::
/// list_due` already established.
pub fn list_due_schedule_triggers(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<AiAgentTrigger>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM ai_agent_triggers
         WHERE workspace_id = ?1 AND trigger_type = 'schedule' AND is_active = 1
           AND (last_run_at IS NULL OR datetime(last_run_at, '+' || interval_minutes || ' minutes') <= datetime('now'))
         ORDER BY created_at",
    )?;
    let rows = stmt.query_map([workspace_id], map_trigger_row)?.collect();
    rows
}

pub fn mark_trigger_run(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE ai_agent_triggers SET last_run_at = ?1 WHERE id = ?2", (now_iso(), id))?;
    Ok(())
}
