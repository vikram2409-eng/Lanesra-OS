//! Integration Hub (spec §6): Connectors - a reusable set of Actions
//! derived from an OpenAPI 3.x document, imported once and then callable
//! by name (spec §6.3/§17) without re-parsing the spec on every use.
//!
//! Parsing is deliberately conservative: only the constructs this crate
//! actually needs to build a working HTTP call (path, method, operationId,
//! parameters, a single JSON request body) are understood. Anything this
//! parser can't safely represent - external `$ref` path items, `callbacks`,
//! `cookie`-located parameters, non-JSON request bodies, unresolved
//! `$ref` schemas - is *skipped or degraded*, never silently ignored: every
//! such case adds a human-readable line to `OpenApiImportPreview::warnings`
//! (spec §6.2: "Reject unsupported OpenAPI constructs with actionable
//! warnings"), and the admin chooses which of the operations that *did*
//! parse to actually expose as Actions (step 4 of the same spec section).
//!
//! `preview_import` and `import` both re-parse the spec text from scratch
//! rather than caching the preview server-side between the two calls -
//! same "no session/temp state" convention `data_exchange_service`'s CSV
//! dry-run already follows.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::integration::{Connector, ConnectorActionParam, ConnectorImportInput, DiscoveredOperation, OpenApiImportPreview};
use crate::repositories::{integration_connection_ref_repo, integration_connector_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

const HTTP_METHODS: &[&str] = &["get", "post", "put", "patch", "delete", "head", "options"];

fn parse_document(spec_text: &str, spec_format: &str) -> AppResult<serde_json::Value> {
    match spec_format {
        "json" => serde_json::from_str(spec_text).map_err(|e| AppError::Validation(format!("Invalid JSON: {e}"))),
        "yaml" => serde_yaml::from_str(spec_text).map_err(|e| AppError::Validation(format!("Invalid YAML: {e}"))),
        other => Err(AppError::Validation(format!("Unknown spec format '{other}' - expected 'json' or 'yaml'"))),
    }
}

fn schema_type_of(schema: &serde_json::Value, warnings: &mut Vec<String>, context: &str) -> String {
    if schema.get("$ref").is_some() {
        warnings.push(format!("{context}: schema uses $ref, which is not resolved - treated as 'object'"));
        return "object".to_string();
    }
    schema.get("type").and_then(|t| t.as_str()).unwrap_or("string").to_string()
}

fn parse_parameters(list: &[serde_json::Value], op_id: &str, warnings: &mut Vec<String>) -> Vec<ConnectorActionParam> {
    let mut params = Vec::new();
    for p in list {
        let (Some(name), Some(location)) = (p.get("name").and_then(|v| v.as_str()), p.get("in").and_then(|v| v.as_str())) else {
            warnings.push(format!("{op_id}: a parameter is missing 'name' or 'in' - skipped"));
            continue;
        };
        if location == "cookie" {
            warnings.push(format!("{op_id}: cookie-located parameter '{name}' is not supported - skipped"));
            continue;
        }
        if location != "path" && location != "query" && location != "header" {
            warnings.push(format!("{op_id}: parameter '{name}' has unknown location '{location}' - skipped"));
            continue;
        }
        let required = p.get("required").and_then(|v| v.as_bool()).unwrap_or(location == "path");
        let schema_type = p
            .get("schema")
            .map(|s| schema_type_of(s, warnings, &format!("{op_id}: parameter '{name}'")))
            .unwrap_or_else(|| "string".to_string());
        params.push(ConnectorActionParam { name: name.to_string(), location: location.to_string(), required, schema_type });
    }
    params
}

/// Whether `schema` is fully local (no `$ref`/`oneOf`/`anyOf`/`allOf`
/// anywhere in it - this parser doesn't resolve any of those, see
/// `schema_type_of`) and every declared property/item has a concrete
/// JSON-Schema type - i.e. safe to store verbatim as
/// `ConnectorAction::request_schema_json` and later hand an LLM as a
/// tool's real `input_schema` without guessing at a shape this parser
/// can't see. Depth-capped, same conservative spirit as the rest of this
/// parser - a schema nested deeper than this is treated as not
/// confidently typed rather than risk unbounded recursion on a
/// pathological spec.
fn is_locally_typed(schema: &serde_json::Value, depth: u8) -> bool {
    if depth > 6 {
        return false;
    }
    if schema.get("$ref").is_some() {
        return false;
    }
    if ["oneOf", "anyOf", "allOf"].iter().any(|k| schema.get(*k).is_some()) {
        return false;
    }
    match schema.get("type").and_then(|t| t.as_str()) {
        Some("object") => schema
            .get("properties")
            .and_then(|p| p.as_object())
            .map_or(true, |props| props.values().all(|p| is_locally_typed(p, depth + 1))),
        Some("array") => schema.get("items").map_or(true, |i| is_locally_typed(i, depth + 1)),
        Some("string" | "integer" | "number" | "boolean" | "null") => true,
        _ => false,
    }
}

fn parse_request_body(body: &serde_json::Value, op_id: &str, warnings: &mut Vec<String>) -> Option<(ConnectorActionParam, Option<serde_json::Value>)> {
    let required = body.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
    let content = body.get("content")?.as_object()?;
    let (media_type, schema) = if let Some(json_schema) = content.get("application/json") {
        ("application/json", json_schema.get("schema"))
    } else if let Some((mt, first)) = content.iter().next() {
        warnings.push(format!("{op_id}: request body uses media type '{mt}', not 'application/json' - treated as opaque JSON"));
        (mt.as_str(), first.get("schema"))
    } else {
        return None;
    };
    let schema_type = schema.map(|s| schema_type_of(s, warnings, &format!("{op_id}: request body ({media_type})"))).unwrap_or_else(|| "object".to_string());
    let param = ConnectorActionParam { name: "body".to_string(), location: "body".to_string(), required, schema_type };
    Some((param, schema.cloned()))
}

/// Parses an OpenAPI 3.x document and reports every operation it could
/// safely represent, plus a warning for every construct it had to skip or
/// degrade - nothing is saved yet (spec §6.2 step 3-4).
pub fn preview_import(spec_text: &str, spec_format: &str) -> AppResult<OpenApiImportPreview> {
    let doc = parse_document(spec_text, spec_format)?;
    let mut warnings = Vec::new();

    let title = doc.pointer("/info/title").and_then(|v| v.as_str()).unwrap_or("Untitled API").to_string();
    let version = doc.pointer("/info/version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();

    if doc.get("webhooks").is_some() {
        warnings.push("Top-level 'webhooks' (OpenAPI 3.1 callback-style webhooks) are not supported - ignored".to_string());
    }

    let mut operations = Vec::new();
    let Some(paths) = doc.get("paths").and_then(|p| p.as_object()) else {
        return Ok(OpenApiImportPreview { title, version, operations, warnings: {
            warnings.push("Document has no 'paths' object - nothing to import".to_string());
            warnings
        } });
    };

    for (path_template, path_item) in paths {
        if path_item.get("$ref").is_some() {
            warnings.push(format!("Path '{path_template}' is an external $ref and was not followed - skipped"));
            continue;
        }
        let Some(path_obj) = path_item.as_object() else { continue };
        let shared_params: Vec<serde_json::Value> = path_obj.get("parameters").and_then(|v| v.as_array()).cloned().unwrap_or_default();

        for method in HTTP_METHODS {
            let Some(operation) = path_obj.get(*method) else { continue };
            if operation.get("callbacks").is_some() {
                let op_id = operation.get("operationId").and_then(|v| v.as_str()).unwrap_or(path_template);
                warnings.push(format!("{op_id}: defines 'callbacks', which are not supported - operation skipped"));
                continue;
            }
            let synthesized_id;
            let operation_id = match operation.get("operationId").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => {
                    synthesized_id = format!("{}_{}", method, path_template.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>());
                    warnings.push(format!("{method} {path_template}: no 'operationId' - using synthesized id '{synthesized_id}'"));
                    &synthesized_id
                }
            };
            let summary = operation.get("summary").and_then(|v| v.as_str()).or_else(|| operation.get("description").and_then(|v| v.as_str())).map(|s| s.to_string());

            let mut own_params: Vec<serde_json::Value> = operation.get("parameters").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let mut all_params = shared_params.clone();
            all_params.append(&mut own_params);
            let mut params = parse_parameters(&all_params, operation_id, &mut warnings);
            let mut request_schema = None;
            if let Some(body) = operation.get("requestBody") {
                if let Some((body_param, schema)) = parse_request_body(body, operation_id, &mut warnings) {
                    params.push(body_param);
                    request_schema = schema;
                }
            }

            operations.push(DiscoveredOperation {
                operation_id: operation_id.to_string(),
                http_method: method.to_uppercase(),
                path_template: path_template.clone(),
                summary,
                params,
                request_schema,
            });
        }
    }

    Ok(OpenApiImportPreview { title, version, operations, warnings })
}

/// Saves the admin's chosen subset of a previewed import as a Connector +
/// its Actions (spec §6.2 step 4-5).
pub fn import(conn: &Connection, workspace_id: &str, input: &ConnectorImportInput, actor_user_id: Option<&str>) -> AppResult<Connector> {
    require_admin(conn, actor_user_id)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Connector name is required".into()));
    }
    let preview = preview_import(&input.spec_text, &input.spec_format)?;
    let selected: Vec<&DiscoveredOperation> = preview.operations.iter().filter(|op| input.selected_operation_ids.contains(&op.operation_id)).collect();
    if selected.is_empty() {
        return Err(AppError::Validation("Select at least one operation to import as an Action".into()));
    }
    let id = new_uuid();
    integration_connector_repo::insert(conn, &id, workspace_id, input.name.trim(), input.description.as_deref(), "rest", "openapi", Some(&input.spec_text), None, actor_user_id)?;
    for op in &selected {
        let params_json = serde_json::to_string(&op.params).unwrap_or_else(|_| "[]".to_string());
        let display_name = op.summary.clone().unwrap_or_else(|| op.operation_id.clone());
        // Only store the resolved body schema when it's confidently typed
        // (see `is_locally_typed`) - an untyped/ambiguous schema stays
        // `None` here exactly as it always has, and `connector_tool_
        // service::agent_tools` treats a `None` on a body-bearing action
        // as the concrete "fail closed" signal rather than guessing.
        let request_schema_json = op
            .request_schema
            .as_ref()
            .filter(|s| is_locally_typed(s, 0))
            .map(|s| serde_json::to_string(s).unwrap_or_default());
        integration_connector_repo::insert_action(
            conn,
            &new_uuid(),
            &id,
            &op.operation_id,
            &display_name,
            &op.http_method,
            &op.path_template,
            &params_json,
            request_schema_json.as_deref(),
            None,
        )?;
    }
    get(conn, workspace_id, &id)
}

pub fn get(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<Connector> {
    let mut connector = integration_connector_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Connector".into()))?;
    if connector.workspace_id != workspace_id {
        return Err(AppError::NotFound("Connector".into()));
    }
    connector.actions = integration_connector_repo::list_actions(conn, id)?;
    Ok(connector)
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Connector>> {
    let mut connectors = integration_connector_repo::list_for_workspace(conn, workspace_id)?;
    for connector in &mut connectors {
        connector.actions = integration_connector_repo::list_actions(conn, &connector.id)?;
    }
    Ok(connectors)
}

pub fn delete(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get(conn, workspace_id, id)?;
    Ok(integration_connector_repo::delete(conn, id)?)
}

/// Integration Hub Tool Bridge: an admin's per-connector agent-tool
/// exposure settings (spec: read-only Actions first, write-capable ones
/// behind this further explicit opt-in - see migration 0050's own
/// comment). `agent_reference_key` is required, and must name a real
/// Connection Reference in this workspace, whenever either flag is set -
/// the same existence-only validation Workflow Automation's own
/// `call_connector_action` already applies to a reference key, no
/// stricter type-check invented here.
pub fn update_agent_tool_settings(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    agent_tools_enabled: bool,
    agent_write_tools_enabled: bool,
    agent_reference_key: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<Connector> {
    require_admin(conn, actor_user_id)?;
    get(conn, workspace_id, id)?;
    if agent_tools_enabled || agent_write_tools_enabled {
        let key = agent_reference_key
            .filter(|k| !k.trim().is_empty())
            .ok_or_else(|| AppError::Validation("A connection reference is required to expose this connector's actions to agents".into()))?;
        if integration_connection_ref_repo::get_by_key(conn, workspace_id, key)?.is_none() {
            return Err(AppError::Validation(format!("Unknown connection reference '{key}'")));
        }
    }
    integration_connector_repo::update_agent_tool_settings(conn, id, agent_tools_enabled, agent_write_tools_enabled, agent_reference_key, actor_user_id)?;
    get(conn, workspace_id, id)
}
