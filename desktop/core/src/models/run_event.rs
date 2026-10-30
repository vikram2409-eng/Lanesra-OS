//! WP-0112: the live run event envelope (ADR-0008 §2), stored in
//! `ai_run_events` (migration 0087) and published on the in-process bus in
//! `services::run_event_service`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The envelope's schema name and version (ADR-0008 §2).
pub const RUN_EVENT_SCHEMA: &str = "lanesra.run-event";
pub const RUN_EVENT_VERSION: i64 = 1;

/// Event types, REQ-API-005. `plan.proposed` is reserved for the planner
/// (Phase 2) and not emitted yet.
pub const RUN_EVENT_TYPES: &[&str] = &[
    "run.started",
    "step.started",
    "round.completed",
    "tool.called",
    "tool.denied",
    "approval.requested",
    "plan.proposed",
    "step.completed",
    "run.completed",
    "run.failed",
];

/// The types a `summary` stream carries (ADR-0008 §3); `steps`, the
/// default, carries every type.
pub fn in_summary(event_type: &str) -> bool {
    event_type.starts_with("run.") || event_type.starts_with("step.")
}

/// The most `data` an event may carry (ADR-0008 §4).
pub const MAX_DATA_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunEvent {
    pub schema: String,
    pub v: i64,
    pub seq: i64,
    #[serde(rename = "type")]
    pub event_type: String,
    pub at: String,
    pub workspace_id: String,
    /// `pipeline` or `graph`: an optional field added within `v: 1`.
    pub run_kind: String,
    pub run_id: String,
    pub step_id: Option<String>,
    pub span_id: Option<String>,
    pub parent_span_id: Option<String>,
    pub data: Value,
    /// The stored row's id, a cursor across runs (SSE `Last-Event-ID` on a
    /// stream that follows several runs). Not part of the envelope.
    #[serde(skip)]
    pub cursor: i64,
}
