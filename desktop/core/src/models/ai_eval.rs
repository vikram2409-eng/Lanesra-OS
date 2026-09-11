//! AI & Agentic Layer, Phase 7d: a real Evaluation Harness -
//! `AiEvalSuite` (a named set of golden test Cases against one Agent or
//! Pipeline) and `AiEvalRun`/`AiEvalCaseResult` (one run's graded
//! results). See `services::ai_eval_service`.

use serde::{Deserialize, Serialize};

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
    /// Replace-all-on-update, same convention `AiAgentPipelineInput::steps`
    /// already uses.
    pub cases: Vec<AiEvalCaseInput>,
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
