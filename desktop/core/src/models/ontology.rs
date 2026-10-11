//! Next-Gen program, Ontology Layer epic (issues #340-#345): the
//! unified Object Type registry (issue #341) and its Link Types
//! (issue #342). A read model over data that already exists -
//! `entity_registry::CORE_ENTITY_TYPES` + `custom_object_service` for
//! object identity, `relationship_service` for links, the Business
//! Glossary (FND-02) for a whole-object semantic description where one
//! is mapped - not a new object/relationship store. See
//! `services::ontology_service` for how each field is resolved.

/// `color_index` is 1..=6, meant to be rendered as `var(--chart-N)` by
/// the caller - never a hardcoded hex. Lets a Theme Studio preset
/// switch re-color every Object Type consistently instead of a second,
/// untouchable palette (the same gap issue #339 named for the
/// Dependency Explorer's own node colors).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ObjectTypeSummary {
    pub key: String,
    pub is_custom: bool,
    pub label_singular: String,
    pub label_plural: String,
    pub icon: String,
    pub color_index: i64,
    /// From a whole-object Business Glossary mapping (FND-02) where one
    /// exists - `None` otherwise, never fabricated.
    pub description: Option<String>,
}

/// One Link Type as seen from `from_object_type`'s own point of view -
/// `name`/`inverse_name` are already oriented so `name` reads naturally
/// from this object type (the relationship's `forward_label` when this
/// object type is the source, `reverse_label` when it's the target).
#[derive(Debug, Clone, serde::Serialize)]
pub struct LinkTypeSummary {
    pub relationship_id: String,
    pub name: String,
    pub inverse_name: String,
    pub from_object_type: String,
    /// `None` for a polymorphic target relationship viewed from its
    /// source side - each link instance carries its own real target
    /// type rather than the definition fixing one (see
    /// `RelationshipDefinition::target_is_polymorphic`'s own doc
    /// comment) - there's no single "to" type to name.
    pub to_object_type: Option<String>,
    pub relationship_type: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ObjectTypeDetail {
    pub object_type: ObjectTypeSummary,
    pub link_types: Vec<LinkTypeSummary>,
}
