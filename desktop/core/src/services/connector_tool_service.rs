//! Integration Hub Tool Bridge: turns a workspace's opted-in Connector
//! Actions into `ToolSpec`s the AI Agent Foundry's `chat_service.rs` can
//! offer a named agent, and dispatches a model's call back into
//! `connector_execution_service::execute` - the same generic "run this
//! action" entry point a Tauri command and a server route already call
//! directly (see that module's own doc comment).
//!
//! Tool names are self-describing by prefix -
//! `connector_action:{connector_id}:{action_key}` for a read-only
//! (GET/HEAD/OPTIONS) action, `connector_write_action:{connector_id}:
//! {action_key}` for a mutating one - so `chat_service::tool_source`/
//! `agent_requires_admin` can classify a name without a DB round trip,
//! while this module remains the one place that actually resolves a name
//! against real, currently-enabled connector/action rows.
//!
//! Fails closed on an ambiguous schema, per the roadmap's own stated
//! requirement: an action is only ever turned into a tool when every one
//! of its params - including a request body, if it has one - has a
//! concrete, informative JSON-Schema type. `connector_service::import`
//! is what populates `ConnectorAction::request_schema_json` when a body
//! schema is confidently typed in the first place; this module adds one
//! more check on top (a present-but-empty `{"type":"object"}` schema is
//! still not informative enough to safely hand a model) rather than
//! guess at a shape it can't see.

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::domain::{AppError, AppResult};
use crate::models::integration::{AgentConnectorToolOption, Connector, ConnectorAction};
use crate::services::ai_service::ToolSpec;
use crate::services::{connector_execution_service, connector_service};

fn is_write_method(http_method: &str) -> bool {
    !matches!(http_method.to_uppercase().as_str(), "GET" | "HEAD" | "OPTIONS")
}

/// Whether a resolved body schema is informative enough to hand a model
/// as a tool parameter - rejects a schema that parsed but carries no
/// real shape (e.g. a bare `{"type":"object"}` with no `properties`,
/// which `is_locally_typed` at import time happily accepts since there
/// was nothing ambiguous to reject, but which is exactly the "vague
/// schema" case this bridge must not turn into a tool).
fn is_informative_schema(schema: &Value) -> bool {
    match schema.get("type").and_then(|t| t.as_str()) {
        Some("object") => schema.get("properties").and_then(|p| p.as_object()).is_some_and(|p| !p.is_empty()),
        Some("array") => schema.get("items").is_some(),
        Some("string" | "integer" | "number" | "boolean" | "null") => true,
        _ => false,
    }
}

fn tool_spec_for(connector: &Connector, action: &ConnectorAction) -> Option<ToolSpec> {
    let is_write = is_write_method(&action.http_method);
    if is_write && !connector.agent_write_tools_enabled {
        return None;
    }

    let mut properties = serde_json::Map::new();
    let mut required: Vec<Value> = Vec::new();
    for param in &action.params {
        let param_schema = if param.location == "body" {
            let schema_json = action.request_schema_json.as_deref()?;
            let schema: Value = serde_json::from_str(schema_json).ok()?;
            if !is_informative_schema(&schema) {
                return None;
            }
            schema
        } else {
            // Path/query/header params are always flat scalars in the
            // shapes this connector's OpenAPI parser understands - an
            // "object"/"array" `schema_type` here means the parser saw
            // something it couldn't safely represent (see connector_
            // service::schema_type_of), so fail closed on the whole
            // action rather than pass through an ambiguous type.
            if matches!(param.schema_type.as_str(), "object" | "array") {
                return None;
            }
            json!({"type": param.schema_type})
        };
        properties.insert(param.name.clone(), param_schema);
        if param.required {
            required.push(json!(param.name));
        }
    }

    let prefix = if is_write { "connector_write_action" } else { "connector_action" };
    let name = format!("{prefix}:{}:{}", connector.id, action.action_key);
    let description = format!(
        "{} - {} {} via the '{}' connector.{}",
        action.display_name,
        action.http_method.to_uppercase(),
        action.path_template,
        connector.name,
        if is_write { " This performs a real write against an external system - use only when the task genuinely calls for it." } else { "" }
    );
    let input_schema = json!({"type": "object", "properties": Value::Object(properties), "required": required});

    Some(ToolSpec { name, description, input_schema })
}

fn build_entries(conn: &Connection, workspace_id: &str) -> AppResult<Vec<(Connector, ConnectorAction, ToolSpec)>> {
    let connectors = connector_service::list_for_workspace(conn, workspace_id)?;
    let mut entries = Vec::new();
    for connector in connectors {
        if !connector.agent_tools_enabled || connector.agent_reference_key.is_none() {
            continue;
        }
        for action in connector.actions.clone() {
            if let Some(spec) = tool_spec_for(&connector, &action) {
                entries.push((connector.clone(), action, spec));
            }
        }
    }
    Ok(entries)
}

/// Every currently-eligible Connector Action in this workspace, as a
/// `ToolSpec` ready to offer a model - see `chat_service::agent_tools`,
/// which chains this in alongside the fixed record/admin catalogs.
pub fn agent_tools(conn: &Connection, workspace_id: &str) -> AppResult<Vec<ToolSpec>> {
    Ok(build_entries(conn, workspace_id)?.into_iter().map(|(_, _, spec)| spec).collect())
}

/// Just the names from `agent_tools` - `ai_agent_service::validate_
/// action_names` uses this to reject a `connector_(write_)?action:`
/// entry that doesn't currently resolve to a real, enabled tool.
pub fn agent_tool_names(conn: &Connection, workspace_id: &str) -> AppResult<Vec<String>> {
    Ok(build_entries(conn, workspace_id)?.into_iter().map(|(_, _, spec)| spec.name).collect())
}

/// The same eligible set, described for an admin's Actions-checklist UI
/// rather than for the model - see `AgentConnectorToolOption`.
pub fn list_options(conn: &Connection, workspace_id: &str) -> AppResult<Vec<AgentConnectorToolOption>> {
    Ok(build_entries(conn, workspace_id)?
        .into_iter()
        .map(|(connector, action, spec)| {
            let requires_admin = is_write_method(&action.http_method);
            AgentConnectorToolOption {
                tool_name: spec.name,
                connector_id: connector.id,
                connector_name: connector.name,
                action_key: action.action_key,
                action_display_name: action.display_name,
                http_method: action.http_method,
                path_template: action.path_template,
                requires_admin,
            }
        })
        .collect())
}

fn parse_tool_name(name: &str) -> Option<(String, String)> {
    let rest = name.strip_prefix("connector_action:").or_else(|| name.strip_prefix("connector_write_action:"))?;
    let (connector_id, action_key) = rest.split_once(':')?;
    Some((connector_id.to_string(), action_key.to_string()))
}

/// Executes one connector-tool call a model made. Re-validates
/// eligibility against the current data rather than trusting that the
/// name was valid when the agent was last saved, so revoking a
/// connector's agent-tool opt-in (or its write opt-in) takes effect on
/// the next call, not just on the next save.
pub async fn dispatch(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], actor_user_id: Option<&str>, name: &str, arguments: &Value) -> AppResult<Value> {
    let (connector_id, action_key) = parse_tool_name(name).ok_or_else(|| AppError::Validation(format!("'{name}' is not a connector tool name")))?;
    let entries = build_entries(conn, workspace_id)?;
    let (connector, action, _) = entries
        .into_iter()
        .find(|(c, a, _)| c.id == connector_id && a.action_key == action_key)
        .ok_or_else(|| AppError::Validation(format!("'{name}' is not currently available as an agent tool")))?;
    let reference_key = connector.agent_reference_key.as_deref().ok_or_else(|| AppError::Validation("Connector has no agent connection reference set".into()))?;
    let result = connector_execution_service::execute(conn, workspace_id, master_key, &connector.id, &action.action_key, reference_key, arguments, actor_user_id).await?;
    serde_json::to_value(result).map_err(|e| AppError::Validation(format!("Could not serialize connector result: {e}")))
}
