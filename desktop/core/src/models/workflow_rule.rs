use serde::{Deserialize, Serialize};

/// Entities whose stage/status transitions can trigger a workflow rule -
/// see the migration's header comment for why Phase 1 is scoped to these
/// two.
pub const WORKFLOW_ENTITY_TYPES: &[&str] = &["Opportunity", "Invoice"];

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
