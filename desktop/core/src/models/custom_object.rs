use serde::{Deserialize, Serialize};

/// Fixed status set for every custom object's records (spec §20.2 gives no
/// separate "custom status values" requirement; this mirrors the simplest
/// existing built-in pattern - Product's Active/Inactive - plus Archived
/// for soft-archive parity with every other entity in the product).
pub const CUSTOM_RECORD_STATUSES: &[&str] = &["Active", "Inactive", "Archived"];

#[derive(Debug, Clone, Serialize)]
pub struct CustomObjectDefinition {
    pub id: String,
    pub workspace_id: String,
    pub key: String,
    pub singular_label: String,
    pub plural_label: String,
    pub icon: String,
    pub prefix: String,
    pub digits: i64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomObjectDefinitionInput {
    pub singular_label: String,
    pub plural_label: String,
    pub icon: String,
    pub prefix: String,
    pub digits: i64,
}

/// Editing a definition: label/icon/prefix/digits/active can all change.
/// `key` cannot - every custom_field_definition/field_rule/workflow_rule/
/// custom_record row for this object is keyed by it.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomObjectDefinitionUpdate {
    pub singular_label: String,
    pub plural_label: String,
    pub icon: String,
    pub prefix: String,
    pub digits: i64,
    pub is_active: bool,
}
