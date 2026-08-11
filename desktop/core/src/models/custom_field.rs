use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const CUSTOM_FIELD_TYPES: &[&str] = &["text", "number", "date", "boolean", "select"];
/// Admin flexibility: every major entity, not just Company/Contact from
/// Phase 1 - matches WORKFLOW_ENTITY_TYPES plus Product (which has no
/// status/stage field to trigger a workflow on, but can still carry custom
/// fields).
pub const CUSTOM_FIELD_ENTITY_TYPES: &[&str] = &[
    "Company", "Contact", "Opportunity", "Quote", "Order", "Invoice", "Contract", "Task", "Product",
];

#[derive(Debug, Clone, Serialize)]
pub struct CustomFieldDefinition {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub key: String,
    pub label: String,
    pub field_type: String,
    pub options: Vec<String>,
    pub required: bool,
    pub show_in_list: bool,
    pub sort_order: i64,
    pub is_active: bool,
    /// ADM-CF-04 (Should): optional validation - min/max apply to `number`
    /// fields, max_length/regex_pattern apply to `text`. All optional;
    /// unset means unrestricted, same as today.
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub max_length: Option<i64>,
    pub regex_pattern: Option<String>,
    /// ADM-CF-05: capability flags. is_reportable gates whether this field
    /// appears as a report builder option; is_searchable/is_filterable are
    /// forward-looking metadata for global search/list-view filtering,
    /// which this build doesn't implement yet (see the migration's
    /// header comment).
    pub is_searchable: bool,
    pub is_filterable: bool,
    pub is_reportable: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomFieldDefinitionInput {
    pub entity_type: String,
    pub label: String,
    pub field_type: String,
    pub options: Vec<String>,
    pub required: bool,
    pub show_in_list: bool,
    pub sort_order: i64,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub max_length: Option<i64>,
    pub regex_pattern: Option<String>,
    pub is_searchable: bool,
    pub is_filterable: bool,
    pub is_reportable: bool,
}

/// Editing a definition: label/options/required/show_in_list/sort_order/
/// active/validation/flags can all change (FR-CFG-02); entity_type, key
/// and field_type cannot, to protect already-stored values from becoming
/// meaningless.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomFieldDefinitionUpdate {
    pub label: String,
    pub options: Vec<String>,
    pub required: bool,
    pub show_in_list: bool,
    pub sort_order: i64,
    pub is_active: bool,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub max_length: Option<i64>,
    pub regex_pattern: Option<String>,
    pub is_searchable: bool,
    pub is_filterable: bool,
    pub is_reportable: bool,
}

/// Values keyed by definition key (not id) - the frontend form only ever
/// needs to know field keys/labels, never the underlying definition row.
pub type CustomFieldValues = HashMap<String, String>;
