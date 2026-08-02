use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub id: String,
    pub workspace_id: String,
    pub task_number: String,
    pub title: String,
    pub description: Option<String>,
    pub owner_user_id: Option<String>,
    pub priority: String,
    pub status: String,
    pub due_date: Option<String>,
    pub reminder_at: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub archived_at: Option<String>,
    /// None when this is a General task (FR-TSK-02).
    pub related_type: Option<String>,
    pub related_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskInput {
    pub title: String,
    pub description: Option<String>,
    pub owner_user_id: Option<String>,
    pub priority: String,
    pub status: String,
    pub due_date: Option<String>,
    pub reminder_at: Option<String>,
    /// Both None means a General task. Otherwise related_type must be one
    /// of TASK_RELATED_TYPES and related_id must reference an existing
    /// record of that type (FR-TSK-02 / FR-TSK-03).
    pub related_type: Option<String>,
    pub related_id: Option<String>,
}

pub const TASK_PRIORITIES: &[&str] = &["Low", "Normal", "High", "Urgent"];
pub const TASK_STATUSES: &[&str] = &["Not Started", "In Progress", "Waiting", "Completed", "Cancelled"];
pub const TASK_RELATED_TYPES: &[&str] = &[
    "Company",
    "Contact",
    "Opportunity",
    "Quote",
    "Order",
    "Invoice",
    "Contract",
];
