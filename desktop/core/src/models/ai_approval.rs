//! AI Agent Platform v2, Phase 1: a durable, generalized pending-action
//! row - see migration `0059_ai_agent_versioning.sql`'s own doc comment
//! for why `subject_type`/`subject_id` name what's pending rather than a
//! column-per-subject-kind, and `services::approval_service` (not yet
//! written) for create/resolve. Not yet wired to
//! `ai_agent_runs.awaiting_approval`/`paused_at_step_order` - that
//! rewiring is later phase work, not this migration.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct AiApproval {
    pub id: String,
    pub workspace_id: String,
    pub subject_type: String,
    pub subject_id: String,
    /// A snapshot of exactly what's being proposed, so resolving this
    /// approval later never depends on `subject_id`'s row still looking
    /// the same as it did when the approval was created.
    pub proposal: serde_json::Value,
    /// `"pending"` | `"approved"` | `"rejected"`.
    pub status: String,
    pub requested_by: Option<String>,
    pub resolved_by: Option<String>,
    pub resolution_notes: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiApprovalInput {
    pub subject_type: String,
    pub subject_id: String,
    pub proposal: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiApprovalResolution {
    /// `true` to approve, `false` to reject.
    pub approve: bool,
    pub resolution_notes: Option<String>,
}
