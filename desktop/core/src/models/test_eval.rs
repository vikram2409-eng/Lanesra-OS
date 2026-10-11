//! Next-Gen program, Domain A (Intelligence Foundation), FND-03: the
//! Unified Test & Evaluation Framework (migration 0075). See
//! `services::test_eval_service` for the seven per-`test_type` executors
//! and the unified runner that drives a Solution Release's validation
//! job.
//!
//! `dataset_json`'s shape depends on `test_type` - it's never
//! deserialized into one fixed struct, only read as a `serde_json::Value`
//! by the matching executor:
//!
//! - `business_rule` (target_id = entity_type): `{"ctx": {"field_key":
//!   "value", ...}, "expect_field_effects": {"field_key": "hide"|...}?,
//!   "expect_blocked": bool?, "expect_errors": ["..."]? }`. An omitted
//!   `expect_*` key means that assertion is skipped - a case can assert
//!   only what it cares about, same as `AiEvalCase`'s "no success
//!   criteria needed" evaluator types.
//! - `workflow` (target_id = entity_type): `{"ctx": {...},
//!   "expect_matched_workflow_names": ["..."]}` (compared as a set).
//! - `access_security` (target_id = object_key): `{"actor_user_id": "...",
//!   "capability": "create"|"read"|"update"|"delete"|"assign",
//!   "record_id": "..."?, "expect_allowed": bool}`.
//! - `screen_visibility` (target_id = entity_type): `{"actor_user_id":
//!   "..."?, "expect_visible_fields": ["..."]?,
//!   "expect_hidden_fields": ["..."]?}` - checked against the actor's
//!   `resolve_effective_layout` result.
//! - `integration_mapping` (target_id = a Mapping's id): `{"source_row":
//!   {"column": "raw value", ...}, "expect_target_fields": {"field":
//!   "value", ...}}`.
//! - `agent_eval` (target_id = an AI Agent's id) / `agent_team_eval`
//!   (target_id = an Execution Graph's id): `{"input_text": "...",
//!   "success_criteria": "..."}`, graded with the same judge call
//!   `ai_eval_service::run_suite`'s `task_completion` evaluator already
//!   uses - not a second grading mechanism.
pub const TEST_TYPES: &[&str] =
    &["business_rule", "workflow", "access_security", "screen_visibility", "integration_mapping", "agent_eval", "agent_team_eval"];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestCaseDefinition {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub test_type: String,
    pub target_id: String,
    pub dataset_json: String,
    pub cost_threshold_usd: Option<f64>,
    pub latency_threshold_ms: Option<i64>,
    pub is_active: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TestCaseDefinitionInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub test_type: String,
    pub target_id: String,
    #[serde(default = "default_dataset_json")]
    pub dataset_json: String,
    #[serde(default)]
    pub cost_threshold_usd: Option<f64>,
    #[serde(default)]
    pub latency_threshold_ms: Option<i64>,
}

fn default_dataset_json() -> String {
    "{}".to_string()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TestRunCaseResult {
    pub id: String,
    pub run_id: String,
    pub test_case_id: Option<String>,
    pub test_case_name: String,
    pub test_type: String,
    pub passed: bool,
    pub evidence_json: String,
    pub trace_text: Option<String>,
    pub runtime_ms: i64,
    pub retries: i64,
    pub cost_usd: Option<f64>,
    pub policy_outcome: String,
    pub component_version_ref: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TestRun {
    pub id: String,
    pub workspace_id: String,
    pub solution_id: Option<String>,
    pub status: String,
    pub passed_count: i64,
    pub failed_count: i64,
    pub triggered_by: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub results: Vec<TestRunCaseResult>,
}

impl TestRun {
    /// The unified readiness result FND-03's own minimum acceptance asks
    /// for: a Solution Release is ready only when every mixed
    /// deterministic/AI case in its validation run passed.
    pub fn is_ready(&self) -> bool {
        self.status == "completed" && self.failed_count == 0
    }
}
