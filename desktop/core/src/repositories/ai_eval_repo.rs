//! Raw CRUD for `ai_eval_suites`/`ai_eval_cases`/`ai_eval_runs`/
//! `ai_eval_case_results` (migration 0045). See
//! `services::ai_eval_service` for validation/gating/execution.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::ai_eval::{AiEvalCase, AiEvalCaseInput, AiEvalCaseResult, AiEvalRun, AiEvalSuite, AiEvalSuiteInput};

fn map_suite_row(row: &rusqlite::Row) -> rusqlite::Result<AiEvalSuite> {
    Ok(AiEvalSuite {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        target_type: row.get("target_type")?,
        target_id: row.get("target_id")?,
        cases: Vec::new(), // filled in by `hydrate`
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

fn map_case_row(row: &rusqlite::Row) -> rusqlite::Result<AiEvalCase> {
    Ok(AiEvalCase { id: row.get("id")?, case_order: row.get("case_order")?, input_text: row.get("input_text")?, success_criteria: row.get("success_criteria")? })
}

pub fn list_cases(conn: &Connection, suite_id: &str) -> rusqlite::Result<Vec<AiEvalCase>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_eval_cases WHERE suite_id = ?1 ORDER BY case_order")?;
    let rows = stmt.query_map([suite_id], map_case_row)?.collect();
    rows
}

fn hydrate(conn: &Connection, mut suite: AiEvalSuite) -> rusqlite::Result<AiEvalSuite> {
    suite.cases = list_cases(conn, &suite.id)?;
    Ok(suite)
}

fn replace_cases(conn: &Connection, suite_id: &str, cases: &[AiEvalCaseInput]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_eval_cases WHERE suite_id = ?1", [suite_id])?;
    for (i, case) in cases.iter().enumerate() {
        conn.execute(
            "INSERT INTO ai_eval_cases (id, suite_id, case_order, input_text, success_criteria) VALUES (?1, ?2, ?3, ?4, ?5)",
            (new_uuid(), suite_id, i as i64, &case.input_text, &case.success_criteria),
        )?;
    }
    Ok(())
}

pub fn create_suite(conn: &Connection, id: &str, workspace_id: &str, input: &AiEvalSuiteInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiEvalSuite> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_eval_suites (id, workspace_id, name, description, target_type, target_id, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?7, ?8)",
        (id, workspace_id, &input.name, &input.description, &input.target_type, &input.target_id, &now, actor_user_id),
    )?;
    replace_cases(conn, id, &input.cases)?;
    get_suite(conn, id).map(|s| s.expect("just inserted"))
}

pub fn update_suite(conn: &Connection, id: &str, input: &AiEvalSuiteInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiEvalSuite> {
    let now = now_iso();
    conn.execute(
        "UPDATE ai_eval_suites SET name = ?1, description = ?2, target_type = ?3, target_id = ?4, updated_at = ?5, updated_by = ?6 WHERE id = ?7",
        (&input.name, &input.description, &input.target_type, &input.target_id, &now, actor_user_id, id),
    )?;
    replace_cases(conn, id, &input.cases)?;
    get_suite(conn, id).map(|s| s.expect("just updated"))
}

pub fn get_suite(conn: &Connection, id: &str) -> rusqlite::Result<Option<AiEvalSuite>> {
    let row = conn.query_row("SELECT * FROM ai_eval_suites WHERE id = ?1", [id], map_suite_row).optional()?;
    row.map(|s| hydrate(conn, s)).transpose()
}

pub fn list_suites(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<AiEvalSuite>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_eval_suites WHERE workspace_id = ?1 ORDER BY name")?;
    let rows: Vec<AiEvalSuite> = stmt.query_map([workspace_id], map_suite_row)?.collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|s| hydrate(conn, s)).collect()
}

pub fn delete_suite(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_eval_suites WHERE id = ?1", [id])?;
    Ok(())
}

// --- Runs ---------------------------------------------------------------

fn map_result_row(row: &rusqlite::Row) -> rusqlite::Result<AiEvalCaseResult> {
    Ok(AiEvalCaseResult {
        id: row.get("id")?,
        case_id: row.get("case_id")?,
        input_text: row.get("input_text")?,
        success_criteria: row.get("success_criteria")?,
        actual_output: row.get("actual_output")?,
        passed: row.get("passed")?,
        judge_reasoning: row.get("judge_reasoning")?,
        error: row.get("error")?,
    })
}

fn map_run_row(row: &rusqlite::Row) -> rusqlite::Result<AiEvalRun> {
    Ok(AiEvalRun {
        id: row.get("id")?,
        suite_id: row.get("suite_id")?,
        workspace_id: row.get("workspace_id")?,
        status: row.get("status")?,
        passed_count: row.get("passed_count")?,
        failed_count: row.get("failed_count")?,
        started_at: row.get("started_at")?,
        finished_at: row.get("finished_at")?,
        results: Vec::new(), // filled in by `hydrate_run`
    })
}

pub fn list_case_results(conn: &Connection, eval_run_id: &str) -> rusqlite::Result<Vec<AiEvalCaseResult>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_eval_case_results WHERE eval_run_id = ?1 ORDER BY rowid")?;
    let rows = stmt.query_map([eval_run_id], map_result_row)?.collect();
    rows
}

fn hydrate_run(conn: &Connection, mut run: AiEvalRun) -> rusqlite::Result<AiEvalRun> {
    run.results = list_case_results(conn, &run.id)?;
    Ok(run)
}

/// Starts a run row up front (mirrors `ai_agent_run_repo::start_run`'s
/// own "insert now, finish later" shape), so a run interrupted mid-way
/// still leaves a record rather than silently vanishing.
pub fn start_run(conn: &Connection, id: &str, suite_id: &str, workspace_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_eval_runs (id, suite_id, workspace_id, status, started_at) VALUES (?1, ?2, ?3, 'running', ?4)",
        (id, suite_id, workspace_id, now_iso()),
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn append_case_result(
    conn: &Connection,
    eval_run_id: &str,
    case_id: &str,
    input_text: &str,
    success_criteria: &str,
    actual_output: Option<&str>,
    passed: bool,
    judge_reasoning: Option<&str>,
    error: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_eval_case_results (id, eval_run_id, case_id, input_text, success_criteria, actual_output, passed, judge_reasoning, error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        (new_uuid(), eval_run_id, case_id, input_text, success_criteria, actual_output, passed, judge_reasoning, error),
    )?;
    Ok(())
}

pub fn finish_run(conn: &Connection, id: &str, passed_count: i64, failed_count: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ai_eval_runs SET status = 'completed', passed_count = ?1, failed_count = ?2, finished_at = ?3 WHERE id = ?4",
        (passed_count, failed_count, now_iso(), id),
    )?;
    Ok(())
}

pub fn get_run(conn: &Connection, id: &str) -> rusqlite::Result<Option<AiEvalRun>> {
    let row = conn.query_row("SELECT * FROM ai_eval_runs WHERE id = ?1", [id], map_run_row).optional()?;
    row.map(|r| hydrate_run(conn, r)).transpose()
}

pub fn list_runs_for_suite(conn: &Connection, suite_id: &str, limit: i64) -> rusqlite::Result<Vec<AiEvalRun>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_eval_runs WHERE suite_id = ?1 ORDER BY started_at DESC LIMIT ?2")?;
    let rows: Vec<AiEvalRun> = stmt.query_map(rusqlite::params![suite_id, limit], map_run_row)?.collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|r| hydrate_run(conn, r)).collect()
}
