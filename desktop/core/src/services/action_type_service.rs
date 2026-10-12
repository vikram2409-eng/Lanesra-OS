//! Next-Gen program, Ontology Layer epic, issue #343: Action Types.
//! `invoke` is the epic's single biggest net-new primitive - a named,
//! parameterized write that always goes through the target object's own
//! real write path (`api_object_service::get_record`/`update_record`,
//! `custom_field_service::set_entity_values`), never a second write
//! engine. See `models::action_type`'s own doc comment for the 3
//! supported `effect`s and why `target_object_type` is restricted to
//! what `api_object_service` can actually write through.

use std::collections::HashMap;

use rusqlite::Connection;
use serde_json::Value;

use crate::domain::{AppError, AppResult};
use crate::models::access_role::Capability;
use crate::models::action_type::{ActionTypeDefinition, ActionTypeDefinitionInput, ACTION_CAPABILITIES, ACTION_EFFECTS};
use crate::models::custom_field::CUSTOM_FIELD_TYPES;
use crate::repositories::{action_type_repo, audit_repo, custom_field_repo};
use crate::services::{access_service, api_object_service, custom_field_service, custom_object_service, entity_registry};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    access_service::require_admin_or_explicit_update(conn, actor_user_id, "ActionTypeDefinition", "Only an Administrator can manage Action Types")
}

fn capability_from_str(s: &str) -> AppResult<Capability> {
    match s {
        "create" => Ok(Capability::Create),
        "read" => Ok(Capability::Read),
        "update" => Ok(Capability::Update),
        "delete" => Ok(Capability::Delete),
        "assign" => Ok(Capability::Assign),
        other => Err(AppError::Validation(format!("Invalid required_capability '{other}'"))),
    }
}

/// The exact boundary `api_object_service::update_record` enforces
/// (Company/Contact/Product/Task, or any active Custom Object) - checked
/// here too so an Action Type is never defined against an object it can
/// never actually invoke on. Deliberately not just
/// `custom_object_service::is_valid_dynamic_entity_type` - that also
/// returns true for every built-in in `CUSTOM_FIELD_ENTITY_TYPES`
/// (Opportunity/Quote/Order/Invoice/Contract included, since they can
/// still carry Custom Fields even though they're read-only for generic
/// writes), which would wrongly let an Action Type target them.
fn require_writable_object_type(conn: &Connection, workspace_id: &str, object_type: &str) -> AppResult<()> {
    if api_object_service::WRITABLE_BUILTINS.contains(&object_type) {
        return Ok(());
    }
    if !entity_registry::CORE_ENTITY_TYPES.contains(&object_type) && custom_object_service::is_valid_dynamic_entity_type(conn, workspace_id, object_type)? {
        return Ok(());
    }
    Err(AppError::Validation(format!(
        "'{object_type}' is not writable through the generic record API, so no Action Type can target it yet - see api_object_service's own doc comment"
    )))
}

fn validate(conn: &Connection, workspace_id: &str, input: &ActionTypeDefinitionInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Action Type name is required".into()));
    }
    require_writable_object_type(conn, workspace_id, &input.target_object_type)?;
    if !ACTION_CAPABILITIES.contains(&input.required_capability.as_str()) {
        return Err(AppError::Validation(format!("Invalid required_capability '{}'", input.required_capability)));
    }
    if !ACTION_EFFECTS.contains(&input.effect.as_str()) {
        return Err(AppError::Validation(format!("Invalid effect '{}'", input.effect)));
    }
    serde_json::from_str::<Value>(&input.effect_config_json).map_err(|e| AppError::Validation(format!("effect_config_json must be valid JSON: {e}")))?;
    let mut seen_keys = std::collections::HashSet::new();
    for param in &input.parameters {
        if param.key.trim().is_empty() {
            return Err(AppError::Validation("Every parameter needs a key".into()));
        }
        if !seen_keys.insert(param.key.clone()) {
            return Err(AppError::Validation(format!("Duplicate parameter key '{}'", param.key)));
        }
        if !CUSTOM_FIELD_TYPES.contains(&param.field_type.as_str()) {
            return Err(AppError::Validation(format!("Invalid parameter field_type '{}'", param.field_type)));
        }
    }
    match input.effect.as_str() {
        "set_status" => {
            let config: Value = serde_json::from_str(&input.effect_config_json).unwrap_or_default();
            if config.get("target_status").and_then(Value::as_str).map(str::trim).unwrap_or("").is_empty() {
                return Err(AppError::Validation("set_status's effect_config_json needs a non-empty 'target_status'".into()));
            }
        }
        "reassign_owner" => {
            let config: Value = serde_json::from_str(&input.effect_config_json).unwrap_or_default();
            let owner_param = config.get("owner_param").and_then(Value::as_str).unwrap_or("");
            if owner_param.is_empty() || !input.parameters.iter().any(|p| p.key == owner_param) {
                return Err(AppError::Validation("reassign_owner's effect_config_json needs an 'owner_param' naming one of this Action Type's own parameters".into()));
            }
        }
        "set_fields" => {
            let config: Value = serde_json::from_str(&input.effect_config_json).unwrap_or_default();
            let field_sources = config.get("field_sources").and_then(Value::as_object);
            if field_sources.map(|m| m.is_empty()).unwrap_or(true) {
                return Err(AppError::Validation("set_fields's effect_config_json needs a non-empty 'field_sources' map".into()));
            }
            let custom_fields = custom_field_repo::list_definitions(conn, workspace_id, &input.target_object_type)?;
            for (field_key, param_key) in field_sources.unwrap() {
                if !custom_fields.iter().any(|d| &d.key == field_key) {
                    return Err(AppError::Validation(format!("'{field_key}' is not a recognized Custom Field on '{}'", input.target_object_type)));
                }
                let param_key = param_key.as_str().unwrap_or("");
                if !input.parameters.iter().any(|p| p.key == param_key) {
                    return Err(AppError::Validation(format!("field_sources['{field_key}'] must name one of this Action Type's own parameters")));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Ontology Layer epic (#340): an Action Type `depends_on` its target
/// Object Type, the same `entity_type` -> `custom_object` edge shape
/// `business_rule_service`'s own sync helper already uses - this is the
/// one piece of System Graph wiring issue #343 never added.
fn sync_graph_node(conn: &Connection, workspace_id: &str, action_type: &ActionTypeDefinition) -> AppResult<()> {
    super::system_graph_service::sync_node(
        conn,
        workspace_id,
        "action_type_definition",
        &action_type.id,
        &action_type.name,
        "{}",
        &[crate::models::system_graph::SystemEdgeTarget {
            edge_type: "depends_on".into(),
            to_node_type: "custom_object".into(),
            to_component_id: action_type.target_object_type.clone(),
        }],
    )
}

pub fn create(conn: &Connection, workspace_id: &str, input: &ActionTypeDefinitionInput, actor_user_id: Option<&str>) -> AppResult<ActionTypeDefinition> {
    require_admin(conn, actor_user_id)?;
    validate(conn, workspace_id, input)?;
    let id = crate::domain::ids::new_uuid();
    let created = action_type_repo::create(conn, &id, workspace_id, input, actor_user_id)?;
    super::solution_component_service::tag_local(conn, workspace_id, "action_type_definition", &created.id, actor_user_id)?;
    sync_graph_node(conn, workspace_id, &created)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "create", Some("action_type_definition"), Some(&created.id), &format!("Created Action Type '{}'", created.name), None)?;
    Ok(created)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<ActionTypeDefinition>> {
    Ok(action_type_repo::get(conn, id)?)
}

pub fn list(conn: &Connection, workspace_id: &str, target_object_type: Option<&str>, active_only: bool) -> AppResult<Vec<ActionTypeDefinition>> {
    let all = action_type_repo::list(conn, workspace_id, target_object_type)?;
    Ok(if active_only { all.into_iter().filter(|a| a.is_active).collect() } else { all })
}

pub fn update(conn: &Connection, id: &str, workspace_id: &str, input: &ActionTypeDefinitionInput, actor_user_id: Option<&str>) -> AppResult<ActionTypeDefinition> {
    require_admin(conn, actor_user_id)?;
    validate(conn, workspace_id, input)?;
    action_type_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Action Type".into()))?;
    let updated = action_type_repo::update(conn, id, input, actor_user_id)?;
    sync_graph_node(conn, workspace_id, &updated)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "update", Some("action_type_definition"), Some(id), &format!("Updated Action Type '{}'", updated.name), None)?;
    Ok(updated)
}

/// Soft only - an Action Type referenced by a page layout's `quick_action`
/// component or an AI agent's Ontology binding (issue #344) must not
/// silently vanish, the same "deactivate, never hard-delete" rule every
/// other referenceable definition in this codebase already follows.
pub fn deactivate(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<ActionTypeDefinition> {
    require_admin(conn, actor_user_id)?;
    let existing = action_type_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Action Type".into()))?;
    action_type_repo::set_active(conn, id, false, actor_user_id)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "deactivate", Some("action_type_definition"), Some(id), &format!("Deactivated Action Type '{}'", existing.name), None)?;
    action_type_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Action Type".into()))
}

fn validate_input_params(definition: &ActionTypeDefinition, input_params: &HashMap<String, String>) -> AppResult<()> {
    for param in &definition.parameters {
        let value = input_params.get(&param.key).map(|s| s.trim()).unwrap_or("");
        if param.required && value.is_empty() {
            return Err(AppError::Validation(format!("'{}' is required", param.label)));
        }
        if value.is_empty() {
            continue;
        }
        match param.field_type.as_str() {
            "number" => {
                value.parse::<f64>().map_err(|_| AppError::Validation(format!("'{}' must be a number", param.label)))?;
            }
            "boolean" => {
                if value != "true" && value != "false" {
                    return Err(AppError::Validation(format!("'{}' must be true or false", param.label)));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Validates `input_params`, checks `definition.required_capability`,
/// dispatches on `definition.effect` through the target object's real
/// write path, writes one audit row naming the Action Type and its
/// resolved parameters, and returns the record's fresh state.
pub fn invoke(
    conn: &Connection,
    workspace_id: &str,
    action_type_id: &str,
    record_id: &str,
    input_params: &HashMap<String, String>,
    actor_user_id: Option<&str>,
) -> AppResult<Value> {
    let definition = action_type_repo::get(conn, action_type_id)?.ok_or_else(|| AppError::NotFound("Action Type".into()))?;
    if definition.workspace_id != workspace_id {
        return Err(AppError::NotFound("Action Type".into()));
    }
    if !definition.is_active {
        return Err(AppError::Validation(format!("Action Type '{}' is deactivated", definition.name)));
    }
    validate_input_params(&definition, input_params)?;
    access_service::require_capability(conn, actor_user_id, &definition.target_object_type, capability_from_str(&definition.required_capability)?, Some(record_id))?;

    let config: Value = serde_json::from_str(&definition.effect_config_json).unwrap_or_default();
    match definition.effect.as_str() {
        "set_status" => {
            let target_status = config.get("target_status").and_then(Value::as_str).unwrap_or("");
            let mut body = api_object_service::get_record(conn, workspace_id, &definition.target_object_type, record_id)?;
            set_json_field(&mut body, "status", target_status);
            api_object_service::update_record(conn, workspace_id, &definition.target_object_type, record_id, &body, actor_user_id)?;
        }
        "reassign_owner" => {
            let owner_param = config.get("owner_param").and_then(Value::as_str).unwrap_or("");
            let new_owner = input_params.get(owner_param).cloned().unwrap_or_default();
            let mut body = api_object_service::get_record(conn, workspace_id, &definition.target_object_type, record_id)?;
            set_json_field(&mut body, "owner_user_id", &new_owner);
            api_object_service::update_record(conn, workspace_id, &definition.target_object_type, record_id, &body, actor_user_id)?;
        }
        "set_fields" => {
            let field_sources = config.get("field_sources").and_then(Value::as_object).cloned().unwrap_or_default();
            let mut values = crate::models::custom_field::CustomFieldValues::new();
            for (field_key, param_key) in &field_sources {
                let param_key = param_key.as_str().unwrap_or("");
                values.insert(field_key.clone(), input_params.get(param_key).cloned().unwrap_or_default());
            }
            custom_field_service::set_entity_values(conn, &definition.target_object_type, record_id, &values, actor_user_id)?;
        }
        other => return Err(AppError::Validation(format!("Unknown Action Type effect '{other}'"))),
    }

    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "invoke_action",
        Some(&definition.target_object_type),
        Some(record_id),
        &format!("Invoked Action Type '{}' on {record_id}", definition.name),
        serde_json::to_string(input_params).ok().as_deref(),
    )?;

    api_object_service::get_record(conn, workspace_id, &definition.target_object_type, record_id)
}

fn set_json_field(body: &mut Value, key: &str, value: &str) {
    if let Some(obj) = body.as_object_mut() {
        obj.insert(key.to_string(), Value::String(value.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_effects_and_capabilities_stay_plain_validated_strings() {
        assert!(ACTION_EFFECTS.contains(&"set_status"));
        assert!(ACTION_CAPABILITIES.contains(&"assign"));
        assert!(capability_from_str("assign").is_ok());
        assert!(capability_from_str("not_a_capability").is_err());
    }
}
