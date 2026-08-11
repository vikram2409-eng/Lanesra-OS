use serde::{Deserialize, Serialize};

pub const MATCH_TYPES: &[&str] = &["all", "any"];
/// Re-exported from `domain::conditions` rather than kept as a second,
/// hand-copied list - this and `domain::conditions::CONDITION_OPERATORS`
/// used to be two separately maintained lists of the same 10 operators;
/// consolidated to one source of truth while adding the new operators
/// below, so there's nothing left to keep in sync by hand.
pub use crate::domain::conditions::CONDITION_OPERATORS;
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
    /// Addendum §2.2 "field-to-field comparison": when both are `Some`,
    /// the condition's effective comparison value is this field's current
    /// value instead of the literal `value` above - resolved at evaluation
    /// time in each engine's service layer (`domain::conditions` itself
    /// never sees the difference). `None` for an ordinary literal-value
    /// condition, which is still the common case.
    pub compare_field_source: Option<String>,
    pub compare_field_key: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessRuleConditionInput {
    pub field_source: String,
    pub field_key: String,
    pub operator: String,
    pub value: String,
    #[serde(default)]
    pub compare_field_source: Option<String>,
    #[serde(default)]
    pub compare_field_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BusinessRuleAction {
    pub id: String,
    pub action_type: String,
    pub target_field_key: Option<String>,
    /// 'builtin' | 'custom' - which registry `target_field_key` was
    /// validated against (see `domain::builtin_fields`). Only meaningful
    /// for `FIELD_TARGETED_ACTIONS`; `None`-target actions (block_save/
    /// show_message) leave this at the column default ('custom') unused.
    pub target_field_source: String,
    pub action_value: Option<String>,
    pub message: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessRuleActionInput {
    pub action_type: String,
    pub target_field_key: Option<String>,
    #[serde(default = "default_field_source")]
    pub target_field_source: String,
    pub action_value: Option<String>,
    pub message: Option<String>,
}

fn default_field_source() -> String {
    "custom".to_string()
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
