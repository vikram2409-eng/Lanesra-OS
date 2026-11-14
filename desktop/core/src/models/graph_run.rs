//! AI Agent Platform v2, Phase 3: the Shared Execution Graph Runtime's
//! durable *execution* state - `ai_runs`/`ai_run_nodes` (migration 0062).
//! See `execution_graph.rs` for the graph *definition* a run walks, and
//! `services::graph_runtime_service`'s module doc comment for the
//! queued -> ... -> completed/failed/cancelled state machine.

/// Run statuses, ADR-0007 §1. `planning` is kept for the Phase 3 plan
/// approval and behaves as a waiting status. Every change between these goes
/// through `services::run_state::transition`.
pub const RUN_STATUSES: &[&str] = &[
    "queued",
    "planning",
    "running",
    "paused",
    "waiting_approval",
    "waiting_budget",
    "waiting_model",
    "waiting_scheduled",
    // ADR-0024 §6 (WP-0704): a lane run waits here for a person to do a
    // human step. The step row sits in `waiting_person` too. No worker
    // holds a lease while a person works; it survives a restart exactly as
    // `waiting_approval` does. `waiting_child` (ADR-0024 §6, a run waiting
    // on a child lane) is added by WP-0716, which brings the machinery that
    // enters and leaves it.
    "waiting_person",
    "completed",
    "completed_partial",
    "failed",
    "cancelled",
];

/// A status a run can durably sit in across a process restart -
/// `resume_run`/`resume_due_delays`/`resolve_approval` are the only ways
/// out of one of these.
pub fn is_waiting_status(status: &str) -> bool {
    matches!(
        status,
        "planning"
            | "paused"
            | "waiting_approval"
            | "waiting_budget"
            | "waiting_model"
            | "waiting_scheduled"
            | "waiting_person"
    )
}

/// Nothing leaves a terminal status (ADR-0007 §3).
pub fn is_terminal_status(status: &str) -> bool {
    matches!(
        status,
        "completed" | "completed_partial" | "failed" | "cancelled"
    )
}

/// Step statuses, ADR-0007 §2. A step has one row per attempt.
pub const RUN_NODE_STATUSES: &[&str] = &[
    "pending",
    "running",
    "waiting_approval",
    // ADR-0024 §4 (WP-0704): a human step's row sits here while the run is
    // `waiting_person`, from when the step is reached (`started_at`) until it
    // is submitted (`finished_at`) - that span is the step's wait time.
    "waiting_person",
    "completed",
    "completed_partial",
    "failed",
    "skipped",
    "cancelled",
];

/// Why a run stopped or is waiting, ADR-0007 §4.
pub const STOP_REASONS: &[&str] = &[
    "end_reached",
    "limit_rounds",
    "limit_tokens",
    "limit_wall_clock",
    "budget_exhausted",
    "approval_pending",
    "approval_rejected",
    "guardrail_failed",
    "policy_denied",
    "model_unavailable",
    "error",
    "cancelled_by_user",
    "cancelled_by_policy",
    "interrupted",
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunNode {
    pub id: String,
    pub run_id: String,
    pub node_id: String,
    pub node_key: String,
    pub node_type: String,
    pub attempt: i64,
    pub status: String,
    /// The lane stage this node belongs to (SOLUTION-DESIGN §18), e.g.
    /// `verify` - the runner's stage bar and the stage timing reports read
    /// the grouping from here. `None` for nodes that are not lane steps.
    /// Step times are `started_at` / `finished_at`, which already exist.
    pub stage_id: Option<String>,
    pub input_json: Option<String>,
    pub output_json: Option<String>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub tokens_in: Option<i64>,
    pub tokens_out: Option<i64>,
    pub cost_usd: Option<f64>,
    /// What went into this step's context and why (WP-0111).
    pub context_receipt_json: Option<String>,
    /// Each guardrail's verdict on this step (WP-0111).
    pub guardrail_verdicts_json: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphRun {
    pub id: String,
    pub workspace_id: String,
    pub graph_id: String,
    pub status: String,
    /// The Custom Object the run is about, for a lane run (ADR-0024 §1) -
    /// e.g. `purchase_order`. `None` for runs that are not lane runs.
    pub anchor_object_key: Option<String>,
    /// The anchor record's id. Together with `anchor_object_key` this is
    /// the one record the lane run is about; one active run per lane per
    /// record (ADR-0024 §6).
    pub anchor_record_id: Option<String>,
    /// Values the lane collects that are not record fields, as JSON, with
    /// the schema declared in the lane definition (ADR-0024 §1). `None`
    /// (not `{}`) for runs that are not lane runs; WP-0704 validates
    /// entered values against the declared schema.
    pub run_data_json: Option<String>,
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
    /// Set on terminal and waiting statuses (ADR-0007 §4).
    pub stop_reason: Option<String>,
    /// The run this one replays (WP-0207).
    pub parent_run_id: Option<String>,
    /// The step of another run this one was forked from (WP-0207).
    pub fork_of_step_id: Option<String>,
    pub budget_json: Option<String>,
    pub cost_usd: Option<f64>,
    /// The worker holding a `running` run, and until when (ADR-0007 §5).
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<String>,
    pub nodes: Vec<RunNode>,
}
