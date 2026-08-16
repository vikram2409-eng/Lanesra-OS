use serde::{Deserialize, Serialize};

/// The admin-facing (PascalCase) entity types whose auto-generated number
/// format can be customized - mirrors domain::numbering's lowercase,
/// lifecycle-internal entity_type strings via numbering_service::config_for.
pub const NUMBERING_ENTITY_TYPES: &[&str] = &[
    "Company", "Contact", "Opportunity", "Product", "Quote", "Order", "Invoice", "Contract", "Task",
];

#[derive(Debug, Clone, Serialize)]
pub struct NumberingOverride {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub prefix: String,
    pub digits: i64,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NumberingOverrideInput {
    pub entity_type: String,
    pub prefix: String,
    pub digits: i64,
}

/// What allocate_number will actually produce for this entity type right
/// now - either an admin override or the built-in default, so the admin
/// screen can show one unified list instead of "overrides" plus a
/// separately-documented set of hardcoded defaults.
#[derive(Debug, Clone, Serialize)]
pub struct EffectiveNumbering {
    pub entity_type: String,
    pub prefix: String,
    pub digits: i64,
    pub example: String,
    pub is_custom: bool,
    /// Only populated when `is_custom` is true - a fallback to the
    /// built-in default has no real record behind it to attribute.
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}
