//! Next-Gen program, Ontology Layer epic, issue #343: Action Types - the
//! epic's single biggest net-new primitive. A named, parameterized write
//! operation scoped to one Object Type ("Mark Opportunity Won",
//! "Reassign Company Owner"), invokable identically by a human (a
//! record-page Quick Action button) and by a governed AI agent tool call
//! (issue #344) - always through the object's own real write path
//! (`api_object_service`/`custom_field_service`), never a second write
//! engine. See `services::action_type_service` for execution.
//!
//! `target_object_type` is restricted to whatever `api_object_service`
//! can actually write through today - Company/Contact/Product/Task and
//! any active Custom Object. Opportunity/Quote/Order/Invoice/Contract
//! are compound documents with their own conversion workflow and are
//! read-only there (see that module's own doc comment) - the same
//! boundary applies here, named rather than silently worked around.

/// The closed, dispatchable vocabulary `action_type_service::invoke`
/// switches on - extensible later the same one-line-constant-change way
/// every other enum-shaped column in this codebase already is.
///
/// - `set_status`: writes the record's one universal `status` field
///   (every writable object type's Input/Update shape has one) to a
///   literal value fixed on the Action Type definition itself
///   (`effect_config_json = {"target_status": "Won"}`), the same literal
///   convention the page builder's pre-existing (inert) `quick_action`
///   component already used for this one field.
/// - `set_fields`: writes one or more admin-defined Custom Field values,
///   sourced from this invocation's own input parameters
///   (`effect_config_json = {"field_sources": {"<custom_field_key>":
///   "<parameter_key>"}}`), through `custom_field_service::set_entity_values`
///   - the one place Business Rule evaluation actually fires in this
///   codebase today, so this is the effect that gets real rule
///   enforcement "for free."
/// - `reassign_owner`: writes the record's universal `owner_user_id`
///   field from a named input parameter
///   (`effect_config_json = {"owner_param": "<parameter_key>"}`) - an
///   admin typically pairs this with `required_capability = "assign"`,
///   though that pairing is data (what the admin picks), not hardcoded.
pub const ACTION_EFFECTS: &[&str] = &["set_status", "set_fields", "reassign_owner"];

/// The exact `models::access_role::Capability` string values
/// (`Capability::as_str()`) - re-declared locally as the validated
/// vocabulary this column's plain string is checked against, the same
/// "duplicate a short literal rather than import across modules" call
/// `models::semantic::DATA_CLASSIFICATIONS` already made.
pub const ACTION_CAPABILITIES: &[&str] = &["create", "read", "update", "delete", "assign"];

/// One entry in an Action Type's ordered parameter list - reuses
/// `models::custom_field::CUSTOM_FIELD_TYPES` for `field_type` rather
/// than inventing a second type system, per the issue's own instruction.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionParameterSpec {
    pub key: String,
    pub label: String,
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionTypeDefinition {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: String,
    pub target_object_type: String,
    pub required_capability: String,
    /// Deserialized from `parameters_json` by the repo layer, the same
    /// opaque-JSON-column convention `BusinessGlossaryTerm::synonyms`
    /// already uses.
    pub parameters: Vec<ActionParameterSpec>,
    pub effect: String,
    pub effect_config_json: String,
    pub is_active: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ActionTypeDefinitionInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub target_object_type: String,
    pub required_capability: String,
    #[serde(default)]
    pub parameters: Vec<ActionParameterSpec>,
    pub effect: String,
    #[serde(default = "default_effect_config_json")]
    pub effect_config_json: String,
}

fn default_effect_config_json() -> String {
    "{}".to_string()
}
