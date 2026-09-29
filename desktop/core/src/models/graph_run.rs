//! AI Agent Platform v2, Phase 3: the Shared Execution Graph Runtime's
//! durable *execution* state - `ai_runs`/`ai_run_nodes` (migration 0062).
//! See `execution_graph.rs` for the graph *definition* a run walks, and
//! `services::graph_runtime_service`'s module doc comment for the
//! queued -> ... -> completed/failed/cancelled state machine.

pub const RUN_STATUSES: &[&str] = &[
    "queued", "planning", "running", "waiting_tool", "waiting_agent", "waiting_approval", "waiting_scheduled", "paused", "completed", "failed", "cancelled",
];

/// A status a run can durably sit in across a process restart -
/// `resume_run`/`resume_due_delays`/`resolve_approval` are the only ways
/// out of one of these.
pub fn is_waiting_status(status: &str) -> bool {
    matches!(status, "waiting_tool" | "waiting_agent" | "waiting_approval" | "waiting_scheduled" | "paused")
}

pub fn is_terminal_status(status: &str) -> bool {
    matches!(status, "completed" | "failed" | "cancelled")
}

pub const RUN_NODE_STATUSES: &[&str] = &["running", "waiting_approval", "completed", "failed", "skipped"];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunNode {
    pub id: String,
    pub run_id: String,
    pub node_id: String,
    pub node_key: String,
    pub node_type: String,
    pub attempt: i64,
    pub status: String,
    pub input_json: Option<String>,
    pub output_json: Option<String>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphRun {
    pub id: String,
    pub workspace_id: String,
    pub graph_id: String,
    pub status: String,
    pub trigger_input: String,
    /// Every completed node's output, keyed by `node_key`, as a JSON
    /// object - `graph_runtime_service::resolve_template` substitutes
    /// `{{node_key.field}}` against this, generalizing
    /// `ai_orchestration_service`'s fixed `{{previous_output}}`/
    /// `{{trigger_input}}` placeholders to an arbitrary graph shape.
    pub context_json: String,
    pub current_node_id: Option<String>,
    pub pending_approval_id: Option<String>,
    pub resume_at: Option<String>,
    pub error_message: Option<String>,
    pub triggered_by: Option<String>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
    pub steps_executed: i64,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub nodes: Vec<RunNode>,
}
