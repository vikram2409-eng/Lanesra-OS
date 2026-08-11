use serde::{Deserialize, Serialize};

pub const MATCH_TYPES: &[&str] = &["all", "any"];
pub const CONDITION_OPERATORS: &[&str] = &[
    "equals", "not_equals", "contains", "not_contains", "is_empty", "is_not_empty",
    "greater_than", "less_than", "on_or_after", "on_or_before",
];
pub const TRIGGER_SOURCES: &[&str] = &["builtin", "custom"];
/// `require`/`hide`/`lock`/`set_default`/`set_value` act on `target_field_key`
/// (which must be an active custom field, same restriction the original
/// field_rules engine had - see the migration's header comment for why).
/// `block_save`/`show_message` act on the whole record and use `message`
/// instead.
pub const ACTION_TYPES: &[&str] = &["require", "hide", "lock", "set_default", "set_value", "block_save", "show_message"];
pub const FIELD_TARGETED_ACTIONS: &[&str] = &["require", "hide", "lock", "set_default", "set_value"];
pub const MESSAGE_ACTIONS: &[&str] = &["block_save", "show_message"];

/// The one built-in, enum-like field each entity type exposes as a rule
/// trigger. Every entity has a `status` column except Product, which only
/// has `is_active` (stored/compared as the strings "true"/"false", the
/// same convention boolean custom field values already use).
pub fn builtin_trigger_field_for(entity_type: &str) -> &'static str {
    match entity_type {
        "Product" => "is_active",
        _ => "status",
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BusinessRuleCondition {
    pub id: String,
    pub field_source: String,
    pub field_key: String,
    pub operator: String,
    pub value: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessRuleConditionInput {
    pub field_source: String,
    pub field_key: String,
    pub operator: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BusinessRuleAction {
    pub id: String,
    pub action_type: String,
    pub target_field_key: Option<String>,
    pub action_value: Option<String>,
    pub message: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessRuleActionInput {
    pub action_type: String,
    pub target_field_key: Option<String>,
    pub action_value: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BusinessRule {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub match_type: String,
    pub priority: i64,
    pub is_active: bool,
    pub effective_start_date: Option<String>,
    pub effective_end_date: Option<String>,
    pub is_protected: bool,
    pub created_at: String,
    pub updated_at: String,
    pub conditions: Vec<BusinessRuleCondition>,
    pub actions: Vec<BusinessRuleAction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessRuleInput {
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub match_type: String,
    pub priority: i64,
    pub effective_start_date: Option<String>,
    pub effective_end_date: Option<String>,
    pub conditions: Vec<BusinessRuleConditionInput>,
    pub actions: Vec<BusinessRuleActionInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessRuleUpdate {
    pub name: String,
    pub description: Option<String>,
    pub match_type: String,
    pub priority: i64,
    pub is_active: bool,
    pub effective_start_date: Option<String>,
    pub effective_end_date: Option<String>,
    pub conditions: Vec<BusinessRuleConditionInput>,
    pub actions: Vec<BusinessRuleActionInput>,
}
