//! AI & Agentic Layer, Phase 7d: a real Evaluation Harness.
//!
//! An Eval Suite CRUD (Administrator-only, same shape every other admin
//! builder here uses - `ai_orchestration_service`'s own Pipeline CRUD in
//! particular), plus `run_suite`, which grades every Case with an
//! LLM-as-judge call rather than a brittle exact-match comparison an
//! open-ended agent response could never satisfy: this is the one
//! genuinely new piece of infrastructure this phase needs, and it's
//! built on the existing `ai_service::complete` primitive Phase 4's
//! natural-language reporting already uses - no new provider plumbing.
//!
//! **Gating**: identical to Pipelines - CRUD and `run_suite` require
//! Administrator up front. Unlike a Pipeline/Trigger run, an eval run has
//! no unattended path (no schedule/webhook fires it), so there's no
//! `run_internal`-style ungated entry point to reason about here.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::ai_agent_pipeline::TRIGGER_TARGET_TYPES;
use crate::models::ai_eval::{AiEvalRun, AiEvalSuite, AiEvalSuiteInput};
use crate::repositories::ai_eval_repo;
use crate::services::{ai_orchestration_service, ai_service};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn validate_suite_input(conn: &Connection, workspace_id: &str, input: &AiEvalSuiteInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Suite name is required".into()));
    }
    if !TRIGGER_TARGET_TYPES.contains(&input.target_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid target type '{}'", input.target_type)));
    }
    ai_orchestration_service::validate_target_exists(conn, workspace_id, &input.target_type, &input.target_id)?;
    if input.cases.is_empty() {
        return Err(AppError::Validation("A suite needs at least one case".into()));
    }
    for case in &input.cases {
        if case.input_text.trim().is_empty() {
            return Err(AppError::Validation("Every case needs an input".into()));
        }
        if case.success_criteria.trim().is_empty() {
            return Err(AppError::Validation("Every case needs a success criteria".into()));
        }
    }
    Ok(())
}

pub fn create_suite(conn: &Connection, workspace_id: &str, input: &AiEvalSuiteInput, actor_user_id: Option<&str>) -> AppResult<AiEvalSuite> {
    require_admin(conn, actor_user_id)?;
    validate_suite_input(conn, workspace_id, input)?;
    let id = new_uuid();
    Ok(ai_eval_repo::create_suite(conn, &id, workspace_id, input, actor_user_id)?)
}

pub fn update_suite(conn: &Connection, id: &str, workspace_id: &str, input: &AiEvalSuiteInput, actor_user_id: Option<&str>) -> AppResult<AiEvalSuite> {
    require_admin(conn, actor_user_id)?;
    ai_eval_repo::get_suite(conn, id)?.ok_or_else(|| AppError::NotFound("Eval suite".into()))?;
    validate_suite_input(conn, workspace_id, input)?;
    Ok(ai_eval_repo::update_suite(conn, id, input, actor_user_id)?)
}

pub fn get_suite(conn: &Connection, id: &str) -> AppResult<Option<AiEvalSuite>> {
    Ok(ai_eval_repo::get_suite(conn, id)?)
}

pub fn list_suites(conn: &Connection, workspace_id: &str) -> AppResult<Vec<AiEvalSuite>> {
    Ok(ai_eval_repo::list_suites(conn, workspace_id)?)
}

pub fn delete_suite(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    ai_eval_repo::get_suite(conn, id)?.ok_or_else(|| AppError::NotFound("Eval suite".into()))?;
    Ok(ai_eval_repo::delete_suite(conn, id)?)
}

pub fn list_runs(conn: &Connection, suite_id: &str, limit: i64) -> AppResult<Vec<AiEvalRun>> {
    Ok(ai_eval_repo::list_runs_for_suite(conn, suite_id, limit)?)
}

const JUDGE_SYSTEM_PROMPT: &str = "You are a strict, impartial grader for an AI system's test suite. You will be given a task the AI was asked to do, a stated success criteria, and the AI's actual response. Decide whether the response satisfies the criteria - do not reward a well-written answer that misses the criteria, and do not penalize a correct answer for an unrelated style choice. Respond with exactly one word on the first line, PASS or FAIL, followed by exactly one sentence of reasoning on the next line. Output nothing else.";

fn build_judge_message(input_text: &str, success_criteria: &str, actual_output: &str) -> String {
    format!("Task input:\n{input_text}\n\nSuccess criteria:\n{success_criteria}\n\nActual response:\n{actual_output}\n\nDoes the actual response satisfy the success criteria?")
}

/// Parses the judge call's own response - the first non-empty line
/// decides pass/fail; anything that isn't a clean "PASS" is treated as a
/// FAIL (a malformed judge reply is graded as failing, not silently
/// ignored - the same fail-closed default `ai_agent_run_repo::start_run`'s
/// own doc comment already reasons about for an interrupted run).
fn parse_judge_reply(reply: &str) -> (bool, String) {
    let mut lines = reply.lines().map(str::trim).filter(|l| !l.is_empty());
    let verdict_line = lines.next().unwrap_or("");
    let passed = verdict_line.eq_ignore_ascii_case("PASS");
    let reasoning = lines.next().unwrap_or("").to_string();
    (passed, reasoning)
}

/// Runs every Case in `suite_id` against its target (agent or pipeline)
/// via the exact same `ai_orchestration_service::run_triggered` a
/// schedule/webhook fire already goes through (`triggered_by: "eval"`,
/// so a run this produces is distinguishable from a real automation
/// fire in the target's own run history), then grades each result with
/// one judge call. A case whose target run itself fails (a provider
/// error, a deleted agent) is recorded as failed without a wasted judge
/// call - grading nothing is not the same as passing.
pub async fn run_suite(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], suite_id: &str, actor_user_id: Option<&str>) -> AppResult<AiEvalRun> {
    require_admin(conn, actor_user_id)?;
    let suite = ai_eval_repo::get_suite(conn, suite_id)?.ok_or_else(|| AppError::NotFound("Eval suite".into()))?;
    if suite.cases.is_empty() {
        return Err(AppError::Validation("This suite has no cases".into()));
    }

    let run_id = new_uuid();
    ai_eval_repo::start_run(conn, &run_id, suite_id, workspace_id)?;

    let mut passed_count = 0i64;
    let mut failed_count = 0i64;
    for case in &suite.cases {
        let target_run = ai_orchestration_service::run_triggered(conn, workspace_id, master_key, &suite.target_type, &suite.target_id, actor_user_id, &case.input_text, Some("eval"), None, None).await;

        let (actual_output, target_error) = match target_run {
            Ok(run) if run.status == "succeeded" => (run.steps.last().and_then(|s| s.output_text.clone()), None),
            Ok(run) => (None, Some(run.error.unwrap_or_else(|| "The target run failed".to_string()))),
            Err(e) => (None, Some(e.to_string())),
        };

        if let Some(error) = target_error {
            failed_count += 1;
            ai_eval_repo::append_case_result(conn, &run_id, &case.id, &case.input_text, &case.success_criteria, None, false, None, Some(&error))?;
            continue;
        }

        let actual_output = actual_output.unwrap_or_default();
        let judge_message = build_judge_message(&case.input_text, &case.success_criteria, &actual_output);
        match ai_service::complete(conn, workspace_id, master_key, JUDGE_SYSTEM_PROMPT, &judge_message).await {
            Ok(reply) => {
                let (passed, reasoning) = parse_judge_reply(&reply);
                if passed {
                    passed_count += 1;
                } else {
                    failed_count += 1;
                }
                ai_eval_repo::append_case_result(conn, &run_id, &case.id, &case.input_text, &case.success_criteria, Some(&actual_output), passed, Some(&reasoning), None)?;
            }
            Err(e) => {
                failed_count += 1;
                let msg = format!("The judge call itself failed: {e}");
                ai_eval_repo::append_case_result(conn, &run_id, &case.id, &case.input_text, &case.success_criteria, Some(&actual_output), false, None, Some(&msg))?;
            }
        }
    }

    ai_eval_repo::finish_run(conn, &run_id, passed_count, failed_count)?;
    Ok(ai_eval_repo::get_run(conn, &run_id)?.expect("just finished"))
}
