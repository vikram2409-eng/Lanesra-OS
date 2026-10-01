//! AI & Agentic Layer, Phase 7d: a real Evaluation Harness -
//! `AiEvalSuite` (a named set of golden test Cases against one Agent or
//! Pipeline) and `AiEvalRun`/`AiEvalCaseResult` (one run's graded
//! results). See `services::ai_eval_service`.

use serde::{Deserialize, Serialize};

/// AI Agent Platform v2, Phase 6a (GitHub issue #171, evaluator-expansion
/// slice): what `ai_eval_service::run_suite` actually checks for every
/// Case in a Suite. `task_completion` is the original, only evaluator
/// this harness ever had - an LLM-as-judge call against a plain-English
/// `success_criteria`. The other two are deterministic - no judge call,
/// no `success_criteria` needed - reusing data this codebase already
/// tracks rather than inventing new grading infrastructure:
/// `structured_output` validates the target agent's current version's
/// declared output schema (AI Agent Platform v2 Phase 1) against the
/// actual response; `policy_compliance` checks whether the target run
/// had any Tool-Call Firewall denial/required-approval (Phase 2).
pub const EVALUATOR_TYPES: [&str; 3] = ["task_completion", "structured_output", "policy_compliance"];

#[derive(Debug, Clone, Serialize)]
pub struct AiEvalCase {
    pub id: String,
    pub case_order: i64,
    pub input_text: String,
    pub success_criteria: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiEvalCaseInput {
    pub input_text: String,
    pub success_criteria: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiEvalSuite {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub target_type: String,
    pub target_id: String,
    pub evaluator_type: String,
    pub cases: Vec<AiEvalCase>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiEvalSuiteInput {
    pub name: String,
    pub description: Option<String>,
    pub target_type: String,
    pub target_id: String,
    #[serde(default = "default_evaluator_type")]
    pub evaluator_type: String,
    /// Replace-all-on-update, same convention `AiAgentPipelineInput::steps`
    /// already uses.
    pub cases: Vec<AiEvalCaseInput>,
}

fn default_evaluator_type() -> String {
    "task_completion".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct AiEvalCaseResult {
    pub id: String,
    pub case_id: String,
    pub input_text: String,
    pub success_criteria: String,
    pub actual_output: Option<String>,
    pub passed: bool,
    pub judge_reasoning: Option<String>,
    /// Set instead of a judged result when the target agent/pipeline run
    /// itself failed - graded as failed without a wasted judge call.
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiEvalRun {
    pub id: String,
    pub suite_id: String,
    pub workspace_id: String,
    pub status: String,
    pub passed_count: i64,
    pub failed_count: i64,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub results: Vec<AiEvalCaseResult>,
}
