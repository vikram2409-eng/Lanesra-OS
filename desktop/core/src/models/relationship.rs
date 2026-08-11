use serde::{Deserialize, Serialize};

pub const RELATIONSHIP_TYPES: &[&str] = &["many_to_one", "one_to_one", "many_to_many"];
pub const DELETE_BEHAVIORS: &[&str] = &["restrict", "archive"];

#[derive(Debug, Clone, Serialize)]
pub struct RelationshipDefinition {
    pub id: String,
    pub workspace_id: String,
    pub key: String,
    pub source_entity_type: String,
    pub target_entity_type: String,
    pub relationship_type: String,
    pub forward_label: String,
    pub reverse_label: String,
    pub is_required: bool,
    pub show_related_list: bool,
    pub delete_behavior: String,
    pub is_protected: bool,
    pub is_active: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RelationshipDefinitionInput {
    pub source_entity_type: String,
    pub target_entity_type: String,
    pub relationship_type: String,
    pub forward_label: String,
    pub reverse_label: String,
    pub is_required: bool,
    pub show_related_list: bool,
    pub delete_behavior: String,
    pub sort_order: i64,
}

/// Editing a definition: everything but the two entity types and
/// relationship_type can change - swapping what a relationship connects or
/// its cardinality after links may already exist is not a safe in-place
/// edit, so that requires deactivating and creating a new definition
/// instead (the same "narrow, deliberate" edit surface custom objects use
/// for their `key`).
#[derive(Debug, Clone, Deserialize)]
pub struct RelationshipDefinitionUpdate {
    pub forward_label: String,
    pub reverse_label: String,
    pub is_required: bool,
    pub show_related_list: bool,
    pub delete_behavior: String,
    pub sort_order: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelationshipInstance {
    pub id: String,
    pub workspace_id: String,
    pub relationship_definition_id: String,
    pub source_entity_type: String,
    pub source_id: String,
    pub target_entity_type: String,
    pub target_id: String,
    pub created_at: String,
}

/// One row the UI renders in a related list - the linked record's own
/// identity plus a display label, without the caller having to make a
/// second round trip through entity_registry itself.
#[derive(Debug, Clone, Serialize)]
pub struct RelatedRecord {
    pub instance_id: String,
    pub relationship_definition_id: String,
    pub relationship_key: String,
    pub label: String,
    pub entity_type: String,
    pub entity_id: String,
    pub display_name: String,
    pub status: String,
    pub archived: bool,
}
