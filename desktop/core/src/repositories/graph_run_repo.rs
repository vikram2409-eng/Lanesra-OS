//! Raw CRUD for `ai_runs`/`ai_run_nodes` (migration 0062) - the direct
//! analogue of `ai_agent_run_repo` for `ai_agent_runs`/`ai_agent_run_steps`,
//! generalized from a fixed step order to an arbitrary graph position. See
//! `services::graph_runtime_service` for the state machine that writes
//! through this. `ai_runs.status` is written only by
//! `services::run_state::transition` (ADR-0007 §4); nothing here changes it.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::graph_run::{GraphRun, RunNode};

fn map_run_row(row: &rusqlite::Row) -> rusqlite::Result<GraphRun> {
    Ok(GraphRun {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        graph_id: row.get("graph_id")?,
        status: row.get("status")?,
        trigger_input: row.get("trigger_input")?,
        context_json: row.get("context_json")?,
        current_node_id: row.get("current_node_id")?,
        pending_approval_id: row.get("pending_approval_id")?,
        resume_at: row.get("resume_at")?,
        error_message: row.get("error_message")?,
        triggered_by: row.get("triggered_by")?,
        source_entity_type: row.get("source_entity_type")?,
        source_entity_id: row.get("source_entity_id")?,
        steps_executed: row.get("steps_executed")?,
        started_at: row.get("started_at")?,
        finished_at: row.get("finished_at")?,
        stop_reason: row.get("stop_reason")?,
        parent_run_id: row.get("parent_run_id")?,
        fork_of_step_id: row.get("fork_of_step_id")?,
        budget_json: row.get("budget_json")?,
        cost_usd: row.get("cost_usd")?,
        lease_owner: row.get("lease_owner")?,
        lease_expires_at: row.get("lease_expires_at")?,
        nodes: Vec::new(), // filled in by `hydrate`
    })
}

fn map_run_node_row(row: &rusqlite::Row) -> rusqlite::Result<RunNode> {
    Ok(RunNode {
        id: row.get("id")?,
        run_id: row.get("run_id")?,
        node_id: row.get("node_id")?,
        node_key: row.get("node_key")?,
        node_type: row.get("node_type")?,
        attempt: row.get("attempt")?,
        status: row.get("status")?,
        input_json: row.get("input_json")?,
        output_json: row.get("output_json")?,
        error_message: row.get("error_message")?,
        started_at: row.get("started_at")?,
        finished_at: row.get("finished_at")?,
        tokens_in: row.get("tokens_in")?,
        tokens_out: row.get("tokens_out")?,
        cost_usd: row.get("cost_usd")?,
        context_receipt_json: row.get("context_receipt_json")?,
        guardrail_verdicts_json: row.get("guardrail_verdicts_json")?,
    })
}

pub fn list_run_nodes(conn: &Connection, run_id: &str) -> rusqlite::Result<Vec<RunNode>> {
    let mut stmt =
        conn.prepare("SELECT * FROM ai_run_nodes WHERE run_id = ?1 ORDER BY started_at")?;
    let rows = stmt.query_map([run_id], map_run_node_row)?.collect();
    rows
}

fn hydrate(conn: &Connection, mut run: GraphRun) -> rusqlite::Result<GraphRun> {
    run.nodes = list_run_nodes(conn, &run.id)?;
    Ok(run)
}

#[allow(clippy::too_many_arguments)]
pub fn start_run(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    graph_id: &str,
    trigger_input: &str,
    triggered_by: Option<&str>,
    source_entity_type: Option<&str>,
    source_entity_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_runs (id, workspace_id, graph_id, status, trigger_input, triggered_by, source_entity_type, source_entity_id, started_at)
         VALUES (?1, ?2, ?3, 'queued', ?4, ?5, ?6, ?7, ?8)",
        (id, workspace_id, graph_id, trigger_input, triggered_by, source_entity_type, source_entity_id, now_iso()),
    )?;
    Ok(())
}

pub fn get_run(conn: &Connection, id: &str) -> rusqlite::Result<Option<GraphRun>> {
    let row = conn
        .query_row("SELECT * FROM ai_runs WHERE id = ?1", [id], map_run_row)
        .optional()?;
    row.map(|r| hydrate(conn, r)).transpose()
}

pub fn list_runs_for_graph(
    conn: &Connection,
    graph_id: &str,
    limit: i64,
) -> rusqlite::Result<Vec<GraphRun>> {
    let mut stmt = conn
        .prepare("SELECT * FROM ai_runs WHERE graph_id = ?1 ORDER BY started_at DESC LIMIT ?2")?;
    let rows: Vec<GraphRun> = stmt
        .query_map(rusqlite::params![graph_id, limit], map_run_row)?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|r| hydrate(conn, r)).collect()
}

/// Runs (in any workspace) sitting in `waiting_scheduled` whose `resume_at`
/// has passed - what `graph_runtime_service::resume_due_delays`'s sweep
/// (mirroring `workflow_service::run_scheduled`'s own pattern for the
/// Workflow engine's `scheduled` trigger type) queries every tick.
pub fn list_due_delayed_runs(
    conn: &Connection,
    workspace_id: &str,
) -> rusqlite::Result<Vec<GraphRun>> {
    let mut stmt =
        conn.prepare("SELECT * FROM ai_runs WHERE workspace_id = ?1 AND status = 'waiting_scheduled' AND resume_at IS NOT NULL AND resume_at <= ?2")?;
    let rows: Vec<GraphRun> = stmt
        .query_map(rusqlite::params![workspace_id, now_iso()], map_run_row)?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|r| hydrate(conn, r)).collect()
}

/// Every run this workspace has left `running` - what the interrupted-run
/// sweep (ADR-0007 §5, WP-0141) checks for an expired lease.
pub fn list_resumable_runs(
    conn: &Connection,
    workspace_id: &str,
) -> rusqlite::Result<Vec<GraphRun>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM ai_runs WHERE workspace_id = ?1 AND status = 'running' ORDER BY started_at",
    )?;
    let rows: Vec<GraphRun> = stmt
        .query_map([workspace_id], map_run_row)?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|r| hydrate(conn, r)).collect()
}

/// Checkpoint after every node transition (see this migration's own module
/// doc comment) - the one function every node executor in
/// `graph_runtime_service` calls before moving on, so a process restart at
/// any point resumes from `current_node_id`/`context_json` exactly as they
/// stood after the last node to actually finish, never a half-applied one.
#[allow(clippy::too_many_arguments)]
pub fn checkpoint(
    conn: &Connection,
    id: &str,
    context_json: &str,
    current_node_id: Option<&str>,
    steps_executed: i64,
) -> rusqlite::Result<()> {
    // WP-0127: no do-not-persist value reaches stored state.
    let context_json = crate::do_not_persist::scrub(context_json);
    conn.execute(
        "UPDATE ai_runs SET context_json = ?1, current_node_id = ?2, steps_executed = ?3 WHERE id = ?4",
        (&context_json, current_node_id, steps_executed, id),
    )?;
    Ok(())
}

/// What a `waiting_approval` run is blocked on. Set right after the run
/// moves to that status.
pub fn set_pending_approval(
    conn: &Connection,
    id: &str,
    context_json: &str,
    current_node_id: &str,
    approval_id: &str,
) -> rusqlite::Result<()> {
    let context_json = crate::do_not_persist::scrub(context_json);
    conn.execute(
        "UPDATE ai_runs SET context_json = ?1, current_node_id = ?2, pending_approval_id = ?3 WHERE id = ?4",
        (&context_json, current_node_id, approval_id, id),
    )?;
    Ok(())
}

/// When a `waiting_scheduled` run is due. Set right after the run moves to
/// that status.
pub fn set_resume_at(
    conn: &Connection,
    id: &str,
    context_json: &str,
    current_node_id: &str,
    resume_at: &str,
) -> rusqlite::Result<()> {
    let context_json = crate::do_not_persist::scrub(context_json);
    conn.execute(
        "UPDATE ai_runs SET context_json = ?1, current_node_id = ?2, resume_at = ?3 WHERE id = ?4",
        (&context_json, current_node_id, resume_at, id),
    )?;
    Ok(())
}

pub fn set_error_message(
    conn: &Connection,
    id: &str,
    error_message: Option<&str>,
) -> rusqlite::Result<()> {
    let error_message = crate::do_not_persist::scrub_opt(error_message);
    conn.execute(
        "UPDATE ai_runs SET error_message = ?1 WHERE id = ?2",
        (error_message, id),
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn append_run_node(
    conn: &Connection,
    run_id: &str,
    node_id: &str,
    node_key: &str,
    node_type: &str,
    attempt: i64,
    status: &str,
    input_json: Option<&str>,
    output_json: Option<&str>,
    error_message: Option<&str>,
    started_at: &str,
    finished_at: Option<&str>,
) -> rusqlite::Result<String> {
    let id = crate::domain::ids::new_uuid();
    let input_json = crate::do_not_persist::scrub_opt(input_json);
    let output_json = crate::do_not_persist::scrub_opt(output_json);
    let error_message = crate::do_not_persist::scrub_opt(error_message);
    conn.execute(
        "INSERT INTO ai_run_nodes (id, run_id, node_id, node_key, node_type, attempt, status, input_json, output_json, error_message, started_at, finished_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        (&id, run_id, node_id, node_key, node_type, attempt, status, input_json, output_json, error_message, started_at, finished_at),
    )?;
    Ok(id)
}
