//! WP-0111: the span tree recorded for every run (ADR-0008 §1) -
//! `ai_run_spans`/`ai_run_span_bodies` (migration 0085). See
//! `services::trace_service` for how spans are opened and closed while a
//! run executes.

use serde::Serialize;
use serde_json::Value;

/// Span kinds, ADR-0008 §1. `context` and `approval` are reserved for the
/// work packages that record them (WP-0131, WP-0150).
pub const SPAN_KINDS: &[&str] = &[
    "run",
    "step",
    "context",
    "model_call",
    "tool_call",
    "policy",
    "guardrail",
    "approval",
];

/// Which table a traced run lives in.
pub const RUN_KINDS: &[&str] = &["pipeline", "graph"];

/// The admin setting for prompt and output bodies (ADR-0008 §4, owner
/// decision ADR-0013 #2). `off` is the default.
pub const TRACE_BODY_MODES: &[&str] = &["off", "redacted", "full"];

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RunSpan {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub run_kind: String,
    pub run_id: String,
    pub seq: i64,
    pub kind: String,
    pub name: String,
    /// `ok`, `error` or `cancelled`; `None` while the span is still open
    /// (for example a run waiting on an approval).
    pub status: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub attributes: Value,
}

/// A stored prompt or output body, kept only when `trace_bodies` is not
/// `off`. `part` is `input` or `output`.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SpanBody {
    pub span_id: String,
    pub part: String,
    pub body: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TraceBodiesInput {
    pub trace_bodies: String,
}
