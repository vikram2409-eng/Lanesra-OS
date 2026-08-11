use serde::{Deserialize, Serialize};

/// Entities whose status/stage field changes through a single generic,
/// caller-supplied entry point - `Company`/`Contact`/`Opportunity`
/// (stage)/`Contract`/`Task`'s `update()`, `Quote`/`Order`'s `set_status()`
/// - which is what makes enforcing an administrator-configured allow-list
/// straightforward. Deliberately excludes:
/// - `Invoice`, whose status changes through several dedicated methods
///   with their own hardcoded semantics (`issue`, `void`,
///   `record_payment`'s automatic Paid/Partially Paid transition,
///   `refresh_overdue`) rather than a caller-supplied "set to X" - gating
///   those behind a generic allow-list risks silently breaking the
///   payment-triggered status flow.
/// - `Product`, whose "status" is a boolean (`is_active`) with no
///   meaningful multi-state transition concept (same reason it's already
///   excluded from `workflow::CORE_WORKFLOW_ENTITY_TYPES`).
/// - Custom objects, deferred to a later phase.
pub const TRANSITION_ENTITY_TYPES: &[&str] =
    &["Company", "Contact", "Opportunity", "Quote", "Order", "Contract", "Task"];

#[derive(Debug, Clone, Serialize)]
pub struct StatusTransition {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    /// `None` = "from any status" (a wildcard rule).
    pub from_status: Option<String>,
    pub to_status: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusTransitionInput {
    pub entity_type: String,
    pub from_status: Option<String>,
    pub to_status: String,
}
