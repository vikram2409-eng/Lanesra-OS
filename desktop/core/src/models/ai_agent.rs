//! AI & Agentic Layer, Phase 6: the AI Agent Foundry's own entities - a
//! named, admin-defined `AiAgentDefinition` (persona + Memory + Actions +
//! delegation) and a reusable `AiSkill` library. See
//! `services::ai_agent_service` and `services::chat_service::
//! run_agent_once`/`send_agent_message` for how these actually run.

use serde::{Deserialize, Serialize};

use crate::models::ai::AiAgentModelRouting;

#[derive(Debug, Clone, Serialize)]
pub struct AiAgentDefinition {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub system_prompt: String,
    /// A living document this agent reads every run and can revise itself
    /// via the always-available `update_memory` tool - see
    /// `chat_service`'s own doc comment. Never `null`, only ever `""`
    /// before anything's been written.
    pub memory_md: String,
    /// Phase 7c: a free-text operational-boundary statement, injected into
    /// this agent's system prompt alongside its persona/memory (see
    /// `chat_service::agent_system_prompt`) - advisory/prompted, not
    /// independently code-enforced (the one guard this phase does enforce
    /// in code, consecutive-identical-tool-call loop detection, needs no
    /// column here - see `chat_service::run_agent_once`). Never `null`,
    /// only ever `""` before anything's been written - same convention as
    /// `memory_md`, and edited the same separate-admin-action way (see
    /// `ai_agent_service::set_guardrails`), not part of the main
    /// create/update form.
    pub guardrails_md: String,
    /// Individual tool names drawn from `chat_service::record_tools()`/
    /// `admin_tools()` - not a coarse scope. Whether this agent needs
    /// Administrator is computed from this set, not stored -
    /// see `ai_agent_service::agent_requires_admin`.
    pub action_names: Vec<String>,
    /// Other active agents this one may call via the `delegate_to_agent`
    /// tool - hierarchy. Populated by `ai_agent_repo::list_delegate_ids`.
    pub delegate_agent_ids: Vec<String>,
    /// Skills attached to this agent - only their name+description are
    /// injected into its system prompt; `instructions_md` is loaded only
    /// when the model calls `use_skill`.
    pub skill_ids: Vec<String>,
    /// Phase 7a: the Gateway's optional per-agent routing policy - `None`
    /// means this agent still dispatches straight through the workspace's
    /// single `ai_settings` default, exactly as every agent did before
    /// this phase. Populated by `ai_agent_repo::get_routing`, edited
    /// separately via `ai_agent_service::set_model_routing` - the same
    /// "its own admin action, not part of the main create/update form
    /// payload" shape `memory_md`/`set_memory` already established.
    pub model_routing: Option<AiAgentModelRouting>,
    pub is_active: bool,
    /// AI Agent Platform v2, Phase 1: this agent's current Published
    /// version (`ai_agents.current_version_id`) - `None` only ever
    /// transiently, between a fresh `ai_agent_repo::create` insert and the
    /// `create_initial_version` call right after it; every agent a caller
    /// can actually observe has one. `chat_service::run_agent_once` reads
    /// it to find this run's Structured Output contract
    /// (`AiAgentVersion.output_schema`), if any.
    pub current_version_id: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiAgentInput {
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub system_prompt: String,
    pub action_names: Vec<String>,
    /// Replace-all-on-update, same convention business rule
    /// conditions/actions already use - not a diff against the prior set.
    pub delegate_agent_ids: Vec<String>,
    pub skill_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiAgentMemoryUpdate {
    pub memory_md: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiAgentGuardrailsUpdate {
    pub guardrails_md: String,
}

/// Phase 7b: one snapshot of `memory_md` taken just before it was
/// overwritten - by the agent's own `update_memory` tool (`changed_by:
/// "agent"`) or an admin's direct edit via `ai_agent_service::set_memory`
/// (`changed_by`: that admin's user id). See
/// `ai_agent_repo::update_memory`/`list_memory_history`.
#[derive(Debug, Clone, Serialize)]
pub struct AiAgentMemorySnapshot {
    pub id: String,
    pub agent_id: String,
    pub memory_md: String,
    pub changed_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiSkill {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    /// What the model sees up front, on every agent this skill is
    /// attached to, before it decides whether to call `use_skill`.
    pub description: String,
    /// Only ever returned via the `use_skill` tool result - never
    /// injected into a system prompt directly.
    pub instructions_md: String,
    pub is_active: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiSkillInput {
    pub name: String,
    pub description: String,
    pub instructions_md: String,
}

/// AI Agent Platform v2, Phase 1: one immutable-once-`published` snapshot
/// of an agent's persona/instructions/tools/model routing/Structured
/// Output contract - see migration `0059_ai_agent_versioning.sql`'s own
/// doc comment for why `ai_agents` keeps its own columns unchanged
/// alongside this (an additive parallel history, not a breaking
/// cutover) and `services::agent_version_service` (not yet written) for
/// the Draft -> Test -> Published -> Deprecated -> Disabled lifecycle
/// this row's `status` moves through.
#[derive(Debug, Clone, Serialize)]
pub struct AiAgentVersion {
    pub id: String,
    pub agent_id: String,
    pub version_number: i64,
    pub status: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub system_prompt: String,
    pub memory_md: String,
    pub guardrails_md: String,
    pub action_names: Vec<String>,
    pub delegate_agent_ids: Vec<String>,
    pub skill_ids: Vec<String>,
    pub model_routing: Option<AiAgentModelRouting>,
    /// A JSON Schema this version's Structured Output must conform to -
    /// `None` means free-form text, exactly today's behavior.
    pub output_schema: Option<serde_json::Value>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiAgentVersionInput {
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub system_prompt: String,
    pub action_names: Vec<String>,
    pub delegate_agent_ids: Vec<String>,
    pub skill_ids: Vec<String>,
    pub model_routing: Option<AiAgentModelRouting>,
    pub output_schema: Option<serde_json::Value>,
}

/// A deliberately small JSON Schema subset (`type`, `required`,
/// `properties`, `items`, `enum`) - enough to gate an agent version's
/// Structured Output contract (`chat_service::run_agent_once`'s one
/// repair-retry loop) without pulling in a full schema-validation crate for
/// one call site. Returns every violation found (not just the first), each
/// as a human-readable `path: problem` string, so a repair prompt can name
/// exactly what to fix.
pub fn validate_output_schema(schema: &serde_json::Value, instance: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();
    validate_node(schema, instance, "$", &mut errors);
    errors
}

fn validate_node(schema: &serde_json::Value, instance: &serde_json::Value, path: &str, errors: &mut Vec<String>) {
    let Some(schema_obj) = schema.as_object() else { return };

    if let Some(expected_type) = schema_obj.get("type").and_then(|t| t.as_str()) {
        let matches = match expected_type {
            "object" => instance.is_object(),
            "array" => instance.is_array(),
            "string" => instance.is_string(),
            "number" => instance.is_number(),
            "integer" => instance.is_i64() || instance.is_u64(),
            "boolean" => instance.is_boolean(),
            "null" => instance.is_null(),
            _ => true,
        };
        if !matches {
            errors.push(format!("{path}: expected type '{expected_type}', got {}", type_name(instance)));
            return;
        }
    }

    if let Some(allowed) = schema_obj.get("enum").and_then(|e| e.as_array()) {
        if !allowed.contains(instance) {
            errors.push(format!("{path}: value is not one of the allowed enum values"));
        }
    }

    if let Some(required) = schema_obj.get("required").and_then(|r| r.as_array()) {
        if let Some(obj) = instance.as_object() {
            for key in required {
                if let Some(key) = key.as_str() {
                    if !obj.contains_key(key) {
                        errors.push(format!("{path}: missing required property '{key}'"));
                    }
                }
            }
        }
    }

    if let Some(properties) = schema_obj.get("properties").and_then(|p| p.as_object()) {
        if let Some(obj) = instance.as_object() {
            for (key, sub_schema) in properties {
                if let Some(value) = obj.get(key) {
                    validate_node(sub_schema, value, &format!("{path}.{key}"), errors);
                }
            }
        }
    }

    if let Some(items_schema) = schema_obj.get("items") {
        if let Some(arr) = instance.as_array() {
            for (i, item) in arr.iter().enumerate() {
                validate_node(items_schema, item, &format!("{path}[{i}]"), errors);
            }
        }
    }
}

fn type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}
