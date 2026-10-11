//! Next-Gen program, Domain A (Intelligence Foundation), FND-03: the
//! Unified Test & Evaluation Framework.
//!
//! Per the issue's own "Note on existing platform state", this is a
//! unifying runner, not a second test engine: every one of the seven
//! `test_type` executors below drives a dry-run/read-only function this
//! codebase already has for real -
//! `business_rule_service::test_rules`, `workflow_service::
//! test_workflows`, `access_service::explain_access`,
//! `screen_layout_service::resolve_effective_layout`,
//! `data_exchange_service::build_record_value` (the same per-row
//! transform a real CSV import applies), and for the two AI test types,
//! the exact run + judge-call primitives `ai_eval_service::run_suite`
//! already uses (`ai_orchestration_service::run_triggered` /
//! `graph_runtime_service::start_run`, graded with
//! `ai_eval_service::build_judge_message`/`parse_judge_reply`).
//!
//! `run_tests` is the unifying piece itself: it runs an arbitrary mixed
//! set of case ids - deterministic and AI alike - as one `TestRun` with
//! one rolled-up readiness verdict (`TestRun::is_ready`).
//! `run_for_solution` resolves a Solution's own tagged test cases and
//! calls `run_tests` for them, `triggered_by: "deployment_validation"` -
//! this is FND-03's own minimum acceptance criterion: "A Solution Release
//! can run mixed deterministic and AI tests in one validation job and
//! return a unified readiness result."
//!
//! **Isolation note** (spec: "Runner executes in an isolated test
//! context"): the five deterministic executors are genuinely read-only -
//! none of them writes to a real record, same as the admin "Test rules"/
//! "Test workflows"/Access Inspector/Dependency Explorer tools they
//! already power. The two AI executors are not sandboxed from the
//! platform's own write paths - `run_triggered`/`start_run` execute for
//! real, exactly as `ai_eval_service::run_suite` already does today, just
//! against a synthetic case input rather than historical production
//! data. A genuinely sandboxed execution context for these is FND-05's
//! own job (Simulation Runtime, named as this issue's own dependency);
//! this phase integrates with it later rather than building a second,
//! smaller sandbox now.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::domain::{AppError, AppResult};
use crate::models::access_role::Capability;
use crate::models::test_eval::{TestCaseDefinition, TestCaseDefinitionInput, TestRun, TEST_TYPES};
use crate::repositories::test_eval_repo;
use crate::services::{
    access_service, ai_agent_service, ai_eval_service, ai_orchestration_service, custom_object_service, data_exchange_service, entity_registry, execution_graph_service,
    graph_runtime_service, mapping_service, system_graph_service,
};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn require_valid_entity_type(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<()> {
    if entity_registry::CORE_ENTITY_TYPES.contains(&entity_type) {
        return Ok(());
    }
    if custom_object_service::is_valid_dynamic_entity_type(conn, workspace_id, entity_type)? {
        return Ok(());
    }
    Err(AppError::Validation(format!("'{entity_type}' is not a recognized object type")))
}

/// `target_id`'s meaning depends on `test_type` (see `models::test_eval`'s
/// own doc comment): an entity_type for the four object-scoped types, a
/// Mapping id, an AI Agent id, or an Execution Graph id.
fn require_valid_target(conn: &Connection, workspace_id: &str, test_type: &str, target_id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    match test_type {
        "business_rule" | "workflow" | "access_security" | "screen_visibility" => require_valid_entity_type(conn, workspace_id, target_id),
        "integration_mapping" => {
            mapping_service::get(conn, workspace_id, target_id)?;
            Ok(())
        }
        "agent_eval" => ai_orchestration_service::validate_target_exists(conn, workspace_id, "agent", target_id),
        "agent_team_eval" => {
            execution_graph_service::get(conn, target_id, workspace_id, actor_user_id)?;
            Ok(())
        }
        _ => Err(AppError::Validation(format!("Invalid test type '{test_type}'"))),
    }
}

fn validate(conn: &Connection, workspace_id: &str, input: &TestCaseDefinitionInput, actor_user_id: Option<&str>) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Test case name is required".into()));
    }
    if !TEST_TYPES.contains(&input.test_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid test type '{}'", input.test_type)));
    }
    serde_json::from_str::<Value>(&input.dataset_json).map_err(|e| AppError::Validation(format!("dataset_json must be valid JSON: {e}")))?;
    require_valid_target(conn, workspace_id, &input.test_type, &input.target_id, actor_user_id)?;
    Ok(())
}

/// A `depends_on` edge to the target's own System Graph node, when one
/// exists - see `models::system_graph`'s own doc comment for why an
/// integration_mapping case (no `Mapping` node type yet) and a case
/// targeting a *built-in* entity_type (only a real Custom Object gets a
/// `custom_object` node - see `custom_object_service::create`) get no
/// edge rather than a fabricated one.
fn sync_graph_node(conn: &Connection, workspace_id: &str, case: &TestCaseDefinition) -> AppResult<()> {
    let edge = match case.test_type.as_str() {
        "business_rule" | "workflow" | "access_security" | "screen_visibility" => {
            if system_graph_service::get_node(conn, workspace_id, "custom_object", &case.target_id)?.is_some() {
                Some(("depends_on", "custom_object", case.target_id.clone()))
            } else {
                None
            }
        }
        "agent_eval" => Some(("depends_on", "ai_agent", case.target_id.clone())),
        "agent_team_eval" => Some(("depends_on", "execution_graph", case.target_id.clone())),
        _ => None,
    };
    let edges: Vec<crate::models::system_graph::SystemEdgeTarget> = edge
        .into_iter()
        .map(|(edge_type, to_node_type, to_component_id)| crate::models::system_graph::SystemEdgeTarget { edge_type: edge_type.into(), to_node_type: to_node_type.into(), to_component_id })
        .collect();
    system_graph_service::sync_node(conn, workspace_id, "test_case_definition", &case.id, &case.name, "{}", &edges)
}

pub fn create(conn: &Connection, workspace_id: &str, input: &TestCaseDefinitionInput, actor_user_id: Option<&str>) -> AppResult<TestCaseDefinition> {
    require_admin(conn, actor_user_id)?;
    validate(conn, workspace_id, input, actor_user_id)?;
    if test_eval_repo::get_by_name(conn, workspace_id, input.name.trim())?.is_some() {
        return Err(AppError::Conflict(format!("A test case named '{}' already exists", input.name.trim())));
    }
    let id = crate::domain::ids::new_uuid();
    let created = test_eval_repo::create(conn, &id, workspace_id, input, actor_user_id)?;
    super::solution_component_service::tag_local(conn, workspace_id, "test_case_definition", &created.id, actor_user_id)?;
    sync_graph_node(conn, workspace_id, &created)?;
    Ok(created)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<TestCaseDefinition>> {
    Ok(test_eval_repo::get(conn, id)?)
}

pub fn list(conn: &Connection, workspace_id: &str, active_only: bool) -> AppResult<Vec<TestCaseDefinition>> {
    let all = test_eval_repo::list(conn, workspace_id)?;
    Ok(if active_only { all.into_iter().filter(|c| c.is_active).collect() } else { all })
}

pub fn update(conn: &Connection, id: &str, workspace_id: &str, input: &TestCaseDefinitionInput, actor_user_id: Option<&str>) -> AppResult<TestCaseDefinition> {
    require_admin(conn, actor_user_id)?;
    validate(conn, workspace_id, input, actor_user_id)?;
    test_eval_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Test case".into()))?;
    if let Some(existing) = test_eval_repo::get_by_name(conn, workspace_id, input.name.trim())? {
        if existing.id != id {
            return Err(AppError::Conflict(format!("A test case named '{}' already exists", input.name.trim())));
        }
    }
    let updated = test_eval_repo::update(conn, id, input, actor_user_id)?;
    sync_graph_node(conn, workspace_id, &updated)?;
    Ok(updated)
}

pub fn deactivate(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<TestCaseDefinition> {
    require_admin(conn, actor_user_id)?;
    test_eval_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Test case".into()))?;
    let _ = workspace_id;
    test_eval_repo::set_active(conn, id, false, actor_user_id)?;
    test_eval_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Test case".into()))
}

/// Hard delete is allowed - nothing else in this slice references a test
/// case by id, the same "nothing dangles" reasoning `metric_service::delete`
/// already applies to metrics.
pub fn delete(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    test_eval_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Test case".into()))?;
    test_eval_repo::delete(conn, id)?;
    system_graph_service::remove_node(conn, workspace_id, "test_case_definition", id)?;
    Ok(())
}

pub fn list_runs(conn: &Connection, workspace_id: &str, limit: i64) -> AppResult<Vec<TestRun>> {
    Ok(test_eval_repo::list_runs_for_workspace(conn, workspace_id, limit)?)
}

pub fn list_runs_for_solution(conn: &Connection, solution_id: &str, limit: i64) -> AppResult<Vec<TestRun>> {
    Ok(test_eval_repo::list_runs_for_solution(conn, solution_id, limit)?)
}

pub fn get_run(conn: &Connection, id: &str) -> AppResult<Option<TestRun>> {
    Ok(test_eval_repo::get_run(conn, id)?)
}

// --- Dataset shapes (see `models::test_eval`'s own doc comment) -----------

#[derive(Debug, Default, Deserialize)]
struct BusinessRuleDataset {
    #[serde(default)]
    ctx: HashMap<String, String>,
    #[serde(default)]
    expect_field_effects: Option<HashMap<String, String>>,
    #[serde(default)]
    expect_blocked: Option<bool>,
    #[serde(default)]
    expect_errors: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
struct WorkflowDataset {
    #[serde(default)]
    ctx: HashMap<String, String>,
    #[serde(default)]
    expect_matched_workflow_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct AccessSecurityDataset {
    actor_user_id: String,
    capability: String,
    #[serde(default)]
    record_id: Option<String>,
    expect_allowed: bool,
}

#[derive(Debug, Default, Deserialize)]
struct ScreenVisibilityDataset {
    #[serde(default)]
    actor_user_id: Option<String>,
    #[serde(default)]
    expect_visible_fields: Vec<String>,
    #[serde(default)]
    expect_hidden_fields: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct IntegrationMappingDataset {
    #[serde(default)]
    source_row: HashMap<String, String>,
    #[serde(default)]
    expect_target_fields: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct EvalDataset {
    input_text: String,
    success_criteria: String,
}

fn parse_dataset<T: for<'de> Deserialize<'de>>(dataset_json: &str) -> AppResult<T> {
    serde_json::from_str(dataset_json).map_err(|e| AppError::Validation(format!("Invalid dataset: {e}")))
}

/// One case's outcome - what each `test_type` branch of `execute_case`
/// produces, before it's written as a `test_run_case_results` row.
struct CaseOutcome {
    passed: bool,
    evidence: Value,
    trace: Option<String>,
    policy_outcome: String,
    component_version_ref: Option<String>,
}

fn ok_outcome(passed: bool, evidence: Value, trace: Option<String>) -> CaseOutcome {
    CaseOutcome { passed, evidence, trace, policy_outcome: "n/a".into(), component_version_ref: Some("live".into()) }
}

/// Runs exactly one case against its target and grades it - the
/// per-`test_type` dispatch this whole framework exists to unify. An
/// error here (a bad dataset, a target that's since been deleted) is
/// caught by the caller and recorded as a failed result, not propagated
/// to abort the rest of the run - the same "grading nothing is not the
/// same as passing" stance `ai_eval_service::run_suite` already takes for
/// a target run that fails outright.
#[allow(clippy::too_many_arguments)]
async fn execute_case(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], case: &TestCaseDefinition, actor_user_id: Option<&str>) -> AppResult<CaseOutcome> {
    match case.test_type.as_str() {
        "business_rule" => {
            let dataset: BusinessRuleDataset = parse_dataset(&case.dataset_json)?;
            let result = super::business_rule_service::test_rules(conn, workspace_id, &case.target_id, &dataset.ctx, actor_user_id)?;
            // A dataset's `expect_field_effects` names a field key opaquely -
            // whether `RuleEvaluation` filed that effect under the
            // custom-field or built-in-field map (see its own doc comment)
            // is an implementation detail the dataset author shouldn't have
            // to know, so check both.
            let mut all_field_effects = result.field_effects.clone();
            all_field_effects.extend(result.builtin_field_effects.clone());
            let mut failures = Vec::new();
            if let Some(expected) = &dataset.expect_field_effects {
                for (k, v) in expected {
                    match all_field_effects.get(k) {
                        Some(actual) if actual == v => {}
                        Some(actual) => failures.push(format!("field '{k}': expected effect '{v}', got '{actual}'")),
                        None => failures.push(format!("field '{k}': expected effect '{v}', got none")),
                    }
                }
            }
            if let Some(expect_blocked) = dataset.expect_blocked {
                let actually_blocked = result.blocked.is_some();
                if actually_blocked != expect_blocked {
                    failures.push(format!("expected blocked={expect_blocked}, got {actually_blocked}"));
                }
            }
            if let Some(expected_errors) = &dataset.expect_errors {
                let mut expected_sorted = expected_errors.clone();
                expected_sorted.sort();
                let mut actual_sorted = result.errors.clone();
                actual_sorted.sort();
                if actual_sorted != expected_sorted {
                    failures.push(format!("expected errors {expected_sorted:?}, got {actual_sorted:?}"));
                }
            }
            let evidence = json!({"field_effects": all_field_effects, "blocked": result.blocked, "errors": result.errors});
            Ok(ok_outcome(failures.is_empty(), evidence, if failures.is_empty() { None } else { Some(failures.join("; ")) }))
        }
        "workflow" => {
            let dataset: WorkflowDataset = parse_dataset(&case.dataset_json)?;
            let result = super::workflow_service::test_workflows(conn, workspace_id, &case.target_id, &dataset.ctx, actor_user_id)?;
            let matched_names: Vec<String> = result.matches.iter().map(|m| m.workflow_name.clone()).collect();
            let mut failures = Vec::new();
            if let Some(expected) = &dataset.expect_matched_workflow_names {
                let mut expected_sorted = expected.clone();
                expected_sorted.sort();
                let mut actual_sorted = matched_names.clone();
                actual_sorted.sort();
                if expected_sorted != actual_sorted {
                    failures.push(format!("expected matched workflows {expected_sorted:?}, got {actual_sorted:?}"));
                }
            }
            let evidence = json!({"matched_workflow_names": matched_names});
            Ok(ok_outcome(failures.is_empty(), evidence, if failures.is_empty() { None } else { Some(failures.join("; ")) }))
        }
        "access_security" => {
            let dataset: AccessSecurityDataset = parse_dataset(&case.dataset_json)?;
            let capability = Capability::from_str(&dataset.capability).ok_or_else(|| AppError::Validation(format!("'{}' is not a recognized capability", dataset.capability)))?;
            let result = access_service::explain_access(conn, &dataset.actor_user_id, &case.target_id, capability, dataset.record_id.as_deref())?;
            let passed = result.decision.allowed == dataset.expect_allowed;
            let evidence = json!({"allowed": result.decision.allowed, "matched_role": result.decision.matched_role, "reason": result.decision.reason});
            Ok(ok_outcome(passed, evidence, Some(result.decision.reason)))
        }
        "screen_visibility" => {
            let dataset: ScreenVisibilityDataset = parse_dataset(&case.dataset_json)?;
            let tabs = super::screen_layout_service::resolve_effective_layout(conn, workspace_id, &case.target_id, dataset.actor_user_id.as_deref())?;
            let visible: HashSet<String> =
                tabs.map(|t| t.tabs.iter().flat_map(|tab| tab.sections.iter().flat_map(|s| s.fields.iter().map(|f| f.key.clone()))).collect()).unwrap_or_default();
            let mut failures = Vec::new();
            for f in &dataset.expect_visible_fields {
                if !visible.contains(f) {
                    failures.push(format!("expected '{f}' visible, but it is not on the resolved layout"));
                }
            }
            for f in &dataset.expect_hidden_fields {
                if visible.contains(f) {
                    failures.push(format!("expected '{f}' hidden, but it is on the resolved layout"));
                }
            }
            let evidence = json!({"visible_fields": visible.into_iter().collect::<Vec<_>>()});
            Ok(ok_outcome(failures.is_empty(), evidence, if failures.is_empty() { None } else { Some(failures.join("; ")) }))
        }
        "integration_mapping" => {
            let dataset: IntegrationMappingDataset = parse_dataset(&case.dataset_json)?;
            let mapping = mapping_service::get(conn, workspace_id, &case.target_id)?;
            let produced = data_exchange_service::build_record_value(&dataset.source_row, &mapping.field_map);
            let mut failures = Vec::new();
            for (k, expected_v) in &dataset.expect_target_fields {
                let actual_v = produced.get(k).and_then(|v| v.as_str()).map(str::to_string);
                if actual_v.as_deref() != Some(expected_v.as_str()) {
                    failures.push(format!("target field '{k}': expected '{expected_v}', got {actual_v:?}"));
                }
            }
            let evidence = json!({"produced": produced});
            Ok(CaseOutcome {
                passed: failures.is_empty(),
                evidence,
                trace: if failures.is_empty() { None } else { Some(failures.join("; ")) },
                policy_outcome: "n/a".into(),
                component_version_ref: Some(format!("updated_at:{}", mapping.updated_at)),
            })
        }
        "agent_eval" => {
            let dataset: EvalDataset = parse_dataset(&case.dataset_json)?;
            let agent = ai_agent_service::get(conn, &case.target_id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
            let run = ai_orchestration_service::run_triggered(conn, workspace_id, master_key, "agent", &case.target_id, actor_user_id, &dataset.input_text, Some("test_eval"), None, None).await;
            match run {
                Ok(r) if r.status == "succeeded" => {
                    let policy_violations: i64 = r.steps.iter().map(|s| s.policy_violations_count).sum();
                    let actual_output = r.steps.last().and_then(|s| s.output_text.clone()).unwrap_or_default();
                    let judge_message = ai_eval_service::build_judge_message(&dataset.input_text, &dataset.success_criteria, &actual_output);
                    let reply = super::ai_service::complete(conn, workspace_id, master_key, ai_eval_service::JUDGE_SYSTEM_PROMPT, &judge_message).await?;
                    let (passed, reasoning) = ai_eval_service::parse_judge_reply(&reply);
                    let policy_outcome = if policy_violations == 0 { "clean" } else { "violation" };
                    let evidence = json!({"actual_output": actual_output, "judge_reasoning": reasoning, "policy_violations": policy_violations});
                    Ok(CaseOutcome { passed, evidence, trace: Some(reasoning), policy_outcome: policy_outcome.into(), component_version_ref: agent.current_version_id.clone() })
                }
                Ok(r) => {
                    let msg = r.error.unwrap_or_else(|| "The target agent run failed".to_string());
                    Ok(CaseOutcome { passed: false, evidence: json!({"error": msg}), trace: Some(msg), policy_outcome: "n/a".into(), component_version_ref: agent.current_version_id.clone() })
                }
                Err(e) => Ok(CaseOutcome { passed: false, evidence: json!({"error": e.to_string()}), trace: Some(e.to_string()), policy_outcome: "n/a".into(), component_version_ref: agent.current_version_id.clone() }),
            }
        }
        "agent_team_eval" => {
            let dataset: EvalDataset = parse_dataset(&case.dataset_json)?;
            let graph = execution_graph_service::get(conn, &case.target_id, workspace_id, actor_user_id)?;
            let component_version_ref = Some(format!("v{}", graph.version));
            let run = graph_runtime_service::start_run(conn, workspace_id, master_key, &case.target_id, &dataset.input_text, actor_user_id, Some("test_eval"), None, None).await;
            match run {
                Ok(r) if r.status == "completed" => {
                    let judge_message = ai_eval_service::build_judge_message(&dataset.input_text, &dataset.success_criteria, &r.context_json);
                    let reply = super::ai_service::complete(conn, workspace_id, master_key, ai_eval_service::JUDGE_SYSTEM_PROMPT, &judge_message).await?;
                    let (passed, reasoning) = ai_eval_service::parse_judge_reply(&reply);
                    let evidence = json!({"context": r.context_json, "judge_reasoning": reasoning});
                    Ok(CaseOutcome { passed, evidence, trace: Some(reasoning), policy_outcome: "n/a".into(), component_version_ref })
                }
                Ok(r) => {
                    let msg = r.error_message.unwrap_or_else(|| "The target agent team run did not complete".to_string());
                    Ok(CaseOutcome { passed: false, evidence: json!({"error": msg}), trace: Some(msg), policy_outcome: "n/a".into(), component_version_ref })
                }
                Err(e) => Ok(CaseOutcome { passed: false, evidence: json!({"error": e.to_string()}), trace: Some(e.to_string()), policy_outcome: "n/a".into(), component_version_ref }),
            }
        }
        other => Err(AppError::Validation(format!("Invalid test type '{other}'"))),
    }
}

/// The unified runner: executes `case_ids` (any mix of the seven test
/// types) as one `TestRun`, recording every case's result and rolling up
/// to one readiness verdict (`TestRun::is_ready`) - FND-03's own minimum
/// acceptance criterion.
pub async fn run_tests(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], case_ids: &[String], solution_id: Option<&str>, triggered_by: &str, actor_user_id: Option<&str>) -> AppResult<TestRun> {
    require_admin(conn, actor_user_id)?;
    if case_ids.is_empty() {
        return Err(AppError::Validation("No test cases to run".into()));
    }
    let cases = test_eval_repo::list_by_ids(conn, case_ids)?;

    let run_id = crate::domain::ids::new_uuid();
    test_eval_repo::start_run(conn, &run_id, workspace_id, solution_id, triggered_by)?;

    let mut passed_count = 0i64;
    let mut failed_count = 0i64;
    for case in &cases {
        let started = Instant::now();
        let outcome = execute_case(conn, workspace_id, master_key, case, actor_user_id).await;
        let runtime_ms = started.elapsed().as_millis() as i64;
        let (passed, evidence_json, trace, policy_outcome, component_version_ref) = match outcome {
            Ok(o) => (o.passed, o.evidence.to_string(), o.trace, o.policy_outcome, o.component_version_ref),
            Err(e) => (false, json!({"error": e.to_string()}).to_string(), Some(e.to_string()), "n/a".to_string(), None),
        };
        if passed {
            passed_count += 1;
        } else {
            failed_count += 1;
        }
        test_eval_repo::append_case_result(
            conn, &run_id, Some(&case.id), &case.name, &case.test_type, passed, &evidence_json, trace.as_deref(), runtime_ms, &policy_outcome, component_version_ref.as_deref(),
        )?;
    }

    test_eval_repo::finish_run(conn, &run_id, passed_count, failed_count)?;
    Ok(test_eval_repo::get_run(conn, &run_id)?.expect("just finished"))
}

/// A Solution Release's own deployment validation job: every
/// `test_case_definition` curated as a member of `solution_id`, run
/// together as one `run_tests` call. This is FND-03's minimum acceptance
/// criterion made concrete - see this module's own doc comment.
pub async fn run_for_solution(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], solution_id: &str, actor_user_id: Option<&str>) -> AppResult<TestRun> {
    super::solution_service::get(conn, workspace_id, solution_id)?;
    let case_ids: Vec<String> = super::solution_service::list_member_refs(conn, solution_id)?
        .into_iter()
        .filter(|m| m.artifact_type == "test_case_definition")
        .map(|m| m.metadata_id)
        .collect();
    if case_ids.is_empty() {
        return Err(AppError::Validation("This Solution has no test cases to validate".into()));
    }
    run_tests(conn, workspace_id, master_key, &case_ids, Some(solution_id), "deployment_validation", actor_user_id).await
}
