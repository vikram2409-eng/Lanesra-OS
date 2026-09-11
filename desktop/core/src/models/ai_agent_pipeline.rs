//! AI & Agentic Layer, Phase 6b: orchestration on top of Phase 6a's
//! Agents - `AiAgentPipeline` (a deterministic, ordered chain of
//! Agents), `AiAgentTrigger` (schedule/webhook - see
//! `services::ai_orchestration_service`'s own doc comment for why a
//! Workflow Automation action is a third trigger kind that needs no
//! model here), and `AiAgentRun`/`AiAgentRunStep` (the unified run log a
//! Pipeline and a lone triggered Agent both write through).

use serde::{Deserialize, Serialize};

pub const TRIGGER_TARGET_TYPES: &[&str] = &["agent", "pipeline"];
pub const TRIGGER_TYPES: &[&str] = &["schedule", "webhook"];

/// AI & Agentic Layer, Phase 7d: three orchestration topologies a
/// Pipeline's `steps` can mean.
/// - `sequential` (the original, still the default): each step chains
///   into the next via `{{previous_output}}`, exactly as before this
///   phase.
/// - `consensus`: every step but the last is a "candidate" - each runs
///   independently against the pipeline's own input (never chained to
///   each other), and the last step is the "synthesizer", which can
///   reference every candidate's answer via a new `{{candidate_outputs}}`
///   placeholder.
/// - `peer_review`: exactly two steps - a drafter and a reviewer -
///   looping (drafter drafts, reviewer critiques, drafter revises from
///   that critique, ...) until the reviewer's answer starts with
///   "APPROVED" or `MAX_PEER_REVIEW_ROUNDS` is reached.
///
/// See `services::ai_orchestration_service::run_internal` for the actual
/// execution split.
pub const PIPELINE_TOPOLOGIES: &[&str] = &["sequential", "consensus", "peer_review"];

#[derive(Debug, Clone, Serialize)]
pub struct PipelineStep {
    pub id: String,
    pub agent_id: String,
    pub step_order: i64,
    /// May reference `{{previous_output}}` (the prior step's final
    /// answer) or `{{trigger_input}}` (what a schedule/webhook/workflow
    /// trigger resolved) - see `ai_orchestration_service::run`'s own
    /// doc comment for the exact substitution rules.
    pub input_template: String,
    /// AI & Agentic Layer, Phase 7e (Human-in-the-loop): when true, the
    /// run pauses right after this step - status `awaiting_approval` -
    /// instead of continuing automatically, until an Administrator
    /// approves (optionally editing the step's output) or rejects it.
    /// Sequential topology only - `validate_pipeline_input` rejects this
    /// on a consensus/peer_review step, since neither maps cleanly onto
    /// "pause after step N, resume from N+1".
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PipelineStepInput {
    pub agent_id: String,
    pub input_template: String,
    #[serde(default)]
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiAgentPipeline {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub topology: String,
    pub steps: Vec<PipelineStep>,
    pub is_active: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiAgentPipelineInput {
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_topology")]
    pub topology: String,
    /// Replace-all-on-update, same convention every other ordered list
    /// in this codebase (business rule conditions, workflow actions)
    /// already uses.
    pub steps: Vec<PipelineStepInput>,
}

fn default_topology() -> String {
    "sequential".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct AiAgentTrigger {
    pub id: String,
    pub workspace_id: String,
    pub target_type: String,
    pub target_id: String,
    pub trigger_type: String,
    pub interval_minutes: Option<i64>,
    pub is_active: bool,
    pub last_run_at: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiAgentTriggerInput {
    pub target_type: String,
    pub target_id: String,
    pub trigger_type: String,
    pub interval_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiAgentRunStep {
    pub id: String,
    pub run_id: String,
    pub agent_id: String,
    pub step_order: i64,
    pub input_text: String,
    pub output_text: Option<String>,
    pub error: Option<String>,
    pub tool_calls_count: i64,
    /// Phase 7e: real wall-clock timing for this step's agent call - the
    /// tracing primitive `run_to_otlp_json`'s spans are built from.
    /// `None` only for a step recorded before this phase's migration.
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiAgentRun {
    pub id: String,
    pub workspace_id: String,
    pub target_type: String,
    pub target_id: String,
    /// 'succeeded' | 'failed' | 'awaiting_approval' | 'rejected'.
    pub status: String,
    pub error: Option<String>,
    pub triggered_by: Option<String>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
    /// Phase 7e: persisted so a paused run can be resumed correctly from
    /// an entirely separate request, whenever the real approval comes in
    /// - a later step's template may reference `{{trigger_input}}`
    /// directly, not only the immediately-prior step's output.
    pub trigger_input: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    /// Phase 7e: set only while `status == "awaiting_approval"` - the
    /// step index `approve_pending_step` resumes from, and the effective
    /// `{{previous_output}}` for that step (the paused-on step's own real
    /// output, unless the approver edits it).
    pub paused_at_step_order: Option<i64>,
    pub resume_previous_output: Option<String>,
    pub steps: Vec<AiAgentRunStep>,
}
