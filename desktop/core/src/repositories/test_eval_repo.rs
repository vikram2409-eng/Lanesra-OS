//! Raw CRUD for `test_case_definitions`/`test_runs`/
//! `test_run_case_results` (migration 0075). See
//! `services::test_eval_service` for validation, the per-`test_type`
//! executors and the unified runner.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::test_eval::{TestCaseDefinition, TestCaseDefinitionInput, TestRun, TestRunCaseResult};

fn map_case_row(row: &rusqlite::Row) -> rusqlite::Result<TestCaseDefinition> {
    Ok(TestCaseDefinition {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        test_type: row.get("test_type")?,
        target_id: row.get("target_id")?,
        dataset_json: row.get("dataset_json")?,
        cost_threshold_usd: row.get("cost_threshold_usd")?,
        latency_threshold_ms: row.get("latency_threshold_ms")?,
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &TestCaseDefinitionInput, actor_user_id: Option<&str>) -> rusqlite::Result<TestCaseDefinition> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO test_case_definitions (id, workspace_id, name, description, test_type, target_id, dataset_json, cost_threshold_usd, latency_threshold_ms, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11, ?10, ?11)",
        params![
            id, workspace_id, input.name, input.description, input.test_type, input.target_id, input.dataset_json,
            input.cost_threshold_usd, input.latency_threshold_ms, now, actor_user_id,
        ],
    )?;
    get(conn, id).map(|c| c.expect("just inserted"))
}

pub fn update(conn: &Connection, id: &str, input: &TestCaseDefinitionInput, actor_user_id: Option<&str>) -> rusqlite::Result<TestCaseDefinition> {
    let now = now_iso();
    conn.execute(
        "UPDATE test_case_definitions SET name = ?1, description = ?2, test_type = ?3, target_id = ?4, dataset_json = ?5,
            cost_threshold_usd = ?6, latency_threshold_ms = ?7, updated_at = ?8, updated_by = ?9
         WHERE id = ?10",
        params![input.name, input.description, input.test_type, input.target_id, input.dataset_json, input.cost_threshold_usd, input.latency_threshold_ms, now, actor_user_id, id],
    )?;
    get(conn, id).map(|c| c.expect("just updated"))
}

pub fn set_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute("UPDATE test_case_definitions SET is_active = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4", params![is_active, now_iso(), actor_user_id, id])?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM test_case_definitions WHERE id = ?1", [id])?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<TestCaseDefinition>> {
    conn.query_row("SELECT * FROM test_case_definitions WHERE id = ?1", [id], map_case_row).optional()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<TestCaseDefinition>> {
    let mut stmt = conn.prepare("SELECT * FROM test_case_definitions WHERE workspace_id = ?1 ORDER BY name")?;
    let rows: rusqlite::Result<Vec<TestCaseDefinition>> = stmt.query_map([workspace_id], map_case_row)?.collect();
    rows
}

pub fn get_by_name(conn: &Connection, workspace_id: &str, name: &str) -> rusqlite::Result<Option<TestCaseDefinition>> {
    conn.query_row("SELECT * FROM test_case_definitions WHERE workspace_id = ?1 AND name = ?2", params![workspace_id, name], map_case_row).optional()
}

pub fn list_by_ids(conn: &Connection, ids: &[String]) -> rusqlite::Result<Vec<TestCaseDefinition>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect::<Vec<_>>().join(",");
    let sql = format!("SELECT * FROM test_case_definitions WHERE id IN ({placeholders})");
    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    let rows: rusqlite::Result<Vec<TestCaseDefinition>> = stmt.query_map(params.as_slice(), map_case_row)?.collect();
    rows
}

// --- Runs -----------------------------------------------------------------

fn map_result_row(row: &rusqlite::Row) -> rusqlite::Result<TestRunCaseResult> {
    Ok(TestRunCaseResult {
        id: row.get("id")?,
        run_id: row.get("run_id")?,
        test_case_id: row.get("test_case_id")?,
        test_case_name: row.get("test_case_name")?,
        test_type: row.get("test_type")?,
        passed: row.get("passed")?,
        evidence_json: row.get("evidence_json")?,
        trace_text: row.get("trace_text")?,
        runtime_ms: row.get("runtime_ms")?,
        retries: row.get("retries")?,
        cost_usd: row.get("cost_usd")?,
        policy_outcome: row.get("policy_outcome")?,
        component_version_ref: row.get("component_version_ref")?,
        created_at: row.get("created_at")?,
    })
}

fn map_run_row(row: &rusqlite::Row) -> rusqlite::Result<TestRun> {
    Ok(TestRun {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        solution_id: row.get("solution_id")?,
        status: row.get("status")?,
        passed_count: row.get("passed_count")?,
        failed_count: row.get("failed_count")?,
        triggered_by: row.get("triggered_by")?,
        started_at: row.get("started_at")?,
        finished_at: row.get("finished_at")?,
        results: Vec::new(), // filled in by `hydrate_run`
    })
}

pub fn list_case_results(conn: &Connection, run_id: &str) -> rusqlite::Result<Vec<TestRunCaseResult>> {
    let mut stmt = conn.prepare("SELECT * FROM test_run_case_results WHERE run_id = ?1 ORDER BY rowid")?;
    let rows = stmt.query_map([run_id], map_result_row)?.collect();
    rows
}

fn hydrate_run(conn: &Connection, mut run: TestRun) -> rusqlite::Result<TestRun> {
    run.results = list_case_results(conn, &run.id)?;
    Ok(run)
}

/// Starts a run row up front (mirrors `ai_eval_repo::start_run`'s own
/// "insert now, finish later" shape) so a run interrupted mid-way still
/// leaves a record rather than silently vanishing.
pub fn start_run(conn: &Connection, id: &str, workspace_id: &str, solution_id: Option<&str>, triggered_by: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO test_runs (id, workspace_id, solution_id, status, triggered_by, started_at) VALUES (?1, ?2, ?3, 'running', ?4, ?5)",
        params![id, workspace_id, solution_id, triggered_by, now_iso()],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn append_case_result(
    conn: &Connection,
    run_id: &str,
    test_case_id: Option<&str>,
    test_case_name: &str,
    test_type: &str,
    passed: bool,
    evidence_json: &str,
    trace_text: Option<&str>,
    runtime_ms: i64,
    policy_outcome: &str,
    component_version_ref: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO test_run_case_results (id, run_id, test_case_id, test_case_name, test_type, passed, evidence_json, trace_text, runtime_ms, retries, cost_usd, policy_outcome, component_version_ref, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, NULL, ?10, ?11, ?12)",
        params![new_uuid(), run_id, test_case_id, test_case_name, test_type, passed, evidence_json, trace_text, runtime_ms, policy_outcome, component_version_ref, now_iso()],
    )?;
    Ok(())
}

pub fn finish_run(conn: &Connection, id: &str, passed_count: i64, failed_count: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE test_runs SET status = 'completed', passed_count = ?1, failed_count = ?2, finished_at = ?3 WHERE id = ?4",
        params![passed_count, failed_count, now_iso(), id],
    )?;
    Ok(())
}

pub fn get_run(conn: &Connection, id: &str) -> rusqlite::Result<Option<TestRun>> {
    let row = conn.query_row("SELECT * FROM test_runs WHERE id = ?1", [id], map_run_row).optional()?;
    row.map(|r| hydrate_run(conn, r)).transpose()
}

pub fn list_runs_for_workspace(conn: &Connection, workspace_id: &str, limit: i64) -> rusqlite::Result<Vec<TestRun>> {
    let mut stmt = conn.prepare("SELECT * FROM test_runs WHERE workspace_id = ?1 ORDER BY started_at DESC LIMIT ?2")?;
    let rows: Vec<TestRun> = stmt.query_map(params![workspace_id, limit], map_run_row)?.collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|r| hydrate_run(conn, r)).collect()
}

pub fn list_runs_for_solution(conn: &Connection, solution_id: &str, limit: i64) -> rusqlite::Result<Vec<TestRun>> {
    let mut stmt = conn.prepare("SELECT * FROM test_runs WHERE solution_id = ?1 ORDER BY started_at DESC LIMIT ?2")?;
    let rows: Vec<TestRun> = stmt.query_map(params![solution_id, limit], map_run_row)?.collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|r| hydrate_run(conn, r)).collect()
}
