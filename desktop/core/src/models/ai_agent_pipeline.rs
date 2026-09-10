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
}

#[derive(Debug, Clone, Deserialize)]
pub struct PipelineStepInput {
    pub agent_id: String,
    pub input_template: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiAgentPipeline {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
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
    /// Replace-all-on-update, same convention every other ordered list
    /// in this codebase (business rule conditions, workflow actions)
    /// already uses.
    pub steps: Vec<PipelineStepInput>,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct AiAgentRun {
    pub id: String,
    pub workspace_id: String,
    pub target_type: String,
    pub target_id: String,
    pub status: String,
    pub error: Option<String>,
    pub triggered_by: Option<String>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub steps: Vec<AiAgentRunStep>,
}
