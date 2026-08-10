use serde::{Deserialize, Serialize};

pub const RULE_OPERATORS: &[&str] = &["equals", "not_equals"];
pub const RULE_EFFECTS: &[&str] = &["require", "hide"];
pub const TRIGGER_SOURCES: &[&str] = &["builtin", "custom"];

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
pub struct FieldRule {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub trigger_field_source: String,
    pub trigger_field_key: String,
    pub operator: String,
    pub trigger_value: String,
    pub target_field_key: String,
    pub effect: String,
    pub is_active: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldRuleInput {
    pub entity_type: String,
    pub trigger_field_source: String,
    pub trigger_field_key: String,
    pub operator: String,
    pub trigger_value: String,
    pub target_field_key: String,
    pub effect: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldRuleUpdate {
    pub trigger_field_source: String,
    pub trigger_field_key: String,
    pub operator: String,
    pub trigger_value: String,
    pub target_field_key: String,
    pub effect: String,
    pub sort_order: i64,
    pub is_active: bool,
}
