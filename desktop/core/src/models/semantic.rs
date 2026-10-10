//! Next-Gen program, Domain A (Intelligence Foundation), FND-02: the
//! Semantic Metadata Layer (migration 0074) - explicit business
//! semantics above raw schema. See `services::glossary_service`,
//! `services::semantic_mapping_service` and `services::metric_service`
//! for the behavior each of these backs, and `models::system_graph`'s own
//! doc comment for how a glossary term/metric's System Graph node is
//! derived from these rows rather than tracked separately.
//!
//! Wave 0 scope: `MetricDefinition` is a declarative *description* of a
//! metric (its aggregation/source/grain/filters/time logic), not a
//! working formula evaluator - no computation engine exists yet. Domain
//! D's later Derived Metrics work consumes this metadata; it doesn't
//! duplicate it.

/// Same 3-value vocabulary `models::ai_memory::CLASSIFICATIONS` already
/// established for memory items - re-declared locally rather than
/// imported across modules for a 3-item constant.
pub const DATA_CLASSIFICATIONS: &[&str] = &["standard", "sensitive", "restricted"];

pub const AGGREGATIONS: &[&str] = &["sum", "avg", "count", "count_distinct", "min", "max"];

/// A curated starting vocabulary, not an exhaustive one - the spec gives
/// these as examples ("such as Customer, Policyholder, Amount..."), and
/// a plain validated string column (like every other enum in this
/// codebase) means adding a value later is a one-line constant change,
/// never a migration.
pub const SEMANTIC_ROLES: &[&str] = &[
    "customer", "policyholder", "amount", "currency", "quantity", "percentage", "effective_date", "expiration_date", "region",
    "owner", "status", "identifier", "email", "phone",
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BusinessGlossaryTerm {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub definition: String,
    pub owner_user_id: Option<String>,
    /// Deserialized from `synonyms_json` by the repo layer, the same
    /// opaque-JSON-column convention `custom_field_definitions.options`
    /// already uses.
    pub synonyms: Vec<String>,
    pub data_classification: String,
    pub is_active: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BusinessGlossaryTermInput {
    pub name: String,
    pub definition: String,
    #[serde(default)]
    pub owner_user_id: Option<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default = "default_classification")]
    pub data_classification: String,
}

fn default_classification() -> String {
    "standard".to_string()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SemanticMapping {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub field_key: Option<String>,
    pub glossary_term_id: Option<String>,
    pub semantic_role: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SemanticMappingInput {
    pub entity_type: String,
    #[serde(default)]
    pub field_key: Option<String>,
    #[serde(default)]
    pub glossary_term_id: Option<String>,
    #[serde(default)]
    pub semantic_role: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricDefinition {
    pub id: String,
    pub workspace_id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub source_entity_type: String,
    pub source_field_key: Option<String>,
    pub aggregation: String,
    pub grain: Option<String>,
    pub filters_json: String,
    pub time_logic: Option<String>,
    pub owner_user_id: Option<String>,
    pub glossary_term_id: Option<String>,
    pub version: i64,
    pub effective_start_date: Option<String>,
    pub effective_end_date: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MetricDefinitionInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub source_entity_type: String,
    #[serde(default)]
    pub source_field_key: Option<String>,
    pub aggregation: String,
    #[serde(default)]
    pub grain: Option<String>,
    #[serde(default = "default_filters_json")]
    pub filters_json: String,
    #[serde(default)]
    pub time_logic: Option<String>,
    #[serde(default)]
    pub owner_user_id: Option<String>,
    #[serde(default)]
    pub glossary_term_id: Option<String>,
    #[serde(default)]
    pub effective_start_date: Option<String>,
    #[serde(default)]
    pub effective_end_date: Option<String>,
}

fn default_filters_json() -> String {
    "{}".to_string()
}

/// A saved revision of a `MetricDefinition` - `snapshot` is the full
/// definition as it stood at `saved_at`, the same
/// deserialize-in-the-service-layer convention `BusinessRuleVersion`
/// already uses for `business_rule_versions`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricVersion {
    pub id: String,
    pub metric_definition_id: String,
    pub snapshot: MetricDefinition,
    pub saved_at: String,
}
