use serde::{Deserialize, Serialize};

pub const TRIGGER_TYPES: &[&str] = &[
    "record_created", "record_updated", "status_changed", "field_changed", "date_reached", "due_overdue", "scheduled",
];
pub const ACTION_TYPES: &[&str] = &[
    "create_task", "update_field", "assign_owner", "create_related_record", "add_notification", "create_reminder",
];
pub const NOTIFICATION_AUDIENCES: &[&str] = &["owner", "all_admins"];

/// Entities whose events can drive record_created/record_updated/
/// status_changed workflow triggers - every entity with a status-like
/// field except Product, which has no meaningful "transition" (unchanged
/// from the original engine's WORKFLOW_ENTITY_TYPES), plus every active
/// custom object (ADM-WF-06), checked dynamically rather than listed here.
pub const CORE_WORKFLOW_ENTITY_TYPES: &[&str] = &[
    "Company", "Contact", "Opportunity", "Quote", "Order", "Invoice", "Contract", "Task",
];

/// Which field's value change fires a status_changed workflow for this
/// entity type - `stage` for Opportunity, `status` for everything else
/// (including custom objects, which only ever have `status`).
pub fn transition_field_for(entity_type: &str) -> &'static str {
    match entity_type {
        "Opportunity" => "stage",
        _ => "status",
    }
}

/// Built-in date fields date_reached/due_overdue can watch, per entity
/// type - deliberately a small, curated list (not every entity has a
/// meaningful date field to automate on) rather than every date column.
pub fn date_fields_for(entity_type: &str) -> &'static [&'static str] {
    match entity_type {
        "Task" => &["due_date"],
        "Quote" => &["expiry_date"],
        "Contract" => &["end_date", "renewal_date"],
        "Invoice" => &["due_date"],
        _ => &[],
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowCondition {
    pub id: String,
    pub field_source: String,
    pub field_key: String,
    pub operator: String,
    pub value: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowConditionInput {
    pub field_source: String,
    pub field_key: String,
    pub operator: String,
    pub value: String,
}

/// `params_json` is a JSON-encoded object whose shape depends on
/// `action_type` - see workflow_service's `*Params` structs for what each
/// action_type expects. Kept as an opaque string at the model/repo layer
/// (like custom_field_definitions.options_json) so this module doesn't
/// need one Rust type per action shape; the service layer parses it at
/// validation and execution time.
#[derive(Debug, Clone, Serialize)]
pub struct WorkflowAction {
    pub id: String,
    pub action_type: String,
    pub params_json: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowActionInput {
    pub action_type: String,
    pub params_json: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_status: Option<String>,
    pub trigger_field_key: Option<String>,
    pub trigger_offset_days: i64,
    pub match_type: String,
    pub priority: i64,
    pub is_active: bool,
    pub is_protected: bool,
    pub last_scheduled_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub conditions: Vec<WorkflowCondition>,
    pub actions: Vec<WorkflowAction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowDefinitionInput {
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_status: Option<String>,
    pub trigger_field_key: Option<String>,
    pub trigger_offset_days: i64,
    pub match_type: String,
    pub priority: i64,
    pub conditions: Vec<WorkflowConditionInput>,
    pub actions: Vec<WorkflowActionInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowDefinitionUpdate {
    pub name: String,
    pub description: Option<String>,
    pub trigger_status: Option<String>,
    pub trigger_field_key: Option<String>,
    pub trigger_offset_days: i64,
    pub match_type: String,
    pub priority: i64,
    pub is_active: bool,
    pub conditions: Vec<WorkflowConditionInput>,
    pub actions: Vec<WorkflowActionInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowRun {
    pub id: String,
    pub workspace_id: String,
    pub workflow_id: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub trigger_type: String,
    pub triggered_at: String,
    pub outcome: String,
    pub actions_summary: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Notification {
    pub id: String,
    pub workspace_id: String,
    pub recipient_user_id: Option<String>,
    pub message: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub created_at: String,
    pub read_at: Option<String>,
}
