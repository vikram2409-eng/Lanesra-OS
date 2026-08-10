use serde::{Deserialize, Serialize};

/// Entities whose stage/status transitions can trigger a workflow rule -
/// every entity with a status-like field except Product, which only has a
/// boolean `is_active` and no meaningful "transition" to automate on.
pub const WORKFLOW_ENTITY_TYPES: &[&str] = &[
    "Company", "Contact", "Opportunity", "Quote", "Order", "Invoice", "Contract", "Task",
];

/// Which field's value change fires a workflow rule for this entity type -
/// `stage` for Opportunity (the field that actually flows through the
/// sales pipeline; `status` there only tracks Open/Won/Lost/Archived,
/// which overlaps stage's own Won/Lost), `status` for everything else.
pub fn transition_field_for(entity_type: &str) -> &'static str {
    match entity_type {
        "Opportunity" => "stage",
        _ => "status",
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowRule {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    /// The stage (Opportunity) or status (Invoice) value that triggers
    /// this rule when the entity transitions *into* it.
    pub trigger_status: String,
    pub task_title: String,
    pub task_description: Option<String>,
    pub due_in_days: i64,
    /// None means "assign to the triggering record's owner".
    pub assignee_user_id: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRuleInput {
    pub entity_type: String,
    pub trigger_status: String,
    pub task_title: String,
    pub task_description: Option<String>,
    pub due_in_days: i64,
    pub assignee_user_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRuleUpdate {
    pub trigger_status: String,
    pub task_title: String,
    pub task_description: Option<String>,
    pub due_in_days: i64,
    pub assignee_user_id: Option<String>,
    pub is_active: bool,
}
