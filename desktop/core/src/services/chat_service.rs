//! AI & Agentic Layer, Phase 5: the LLM Chat Assistant. Two modes:
//!
//! - **`"records"`** (any authenticated user): the exact same 7 tools
//!   MCP already exposes over `api_object_service` (`server/src/mcp.rs`)
//!   - a second caller of already-proven, already-tested logic, not new
//!     record-access code.
//! - **`"admin"`** (Administrator only): create/list tools over most of
//!   the admin configuration surface - Business Rules, Workflow
//!   Automation, Custom Objects/Fields/Relationships, Status
//!   Transitions, Integration Hub (Connections/Webhooks/Integration
//!   Jobs/API Clients, Connectors read-only), Dashboards, Apps, Custom
//!   Reports, Saved Views, Users, Numbering, and the workspace profile.
//!   Every tool wraps an **already-existing, already-validated** service
//!   `create`/`list` function - deserializing the tool's `arguments`
//!   straight into that service's own `*Input` struct, exactly the way
//!   `mcp.rs`'s own `create_record` already wraps
//!   `api_object_service::create_record`. Not built this pass:
//!   Connectors (importing one needs an uploaded OpenAPI file, not chat
//!   text - list-only here), and anything not named above (Screen
//!   Layouts, Deployment Management, ...) - named, not silently absent.
//!
//! **The one non-negotiable boundary**: `create_connection` and
//! `create_api_client` never accept or persist a plaintext secret. A
//! Connection's `secret_value` is never read from the tool's arguments
//! at all (silently dropped even if a model or user tries to supply
//! one) - the tool creates the Connection's shape (name, base URL, auth
//! mode) and the assistant is told to instruct the admin to add the
//! credential through the existing secure form, the same `type=
//! "password"` field every other credential in this codebase already
//! goes through. An API Client's secret is server-generated (never
//! user-supplied), so it's created for real, but the generated value is
//! never echoed into the tool's result - same "shown once" convention
//! the real Admin UI already uses, just never shown in chat at all.
//! Every other field of both tools works exactly like any other tool
//! here; this is the one deliberate exception, not a capability cut.
//!
//! The tool-calling loop (`send_message`): append the user's message,
//! then repeatedly call `ai_service::complete_with_tools` and execute
//! whatever it requests, feeding results back, until it answers in
//! plain text - capped at `MAX_ROUNDS` (a runaway-cost/loop guard, not a
//! real limit any legitimate exchange needs).

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::domain::{AppError, AppResult};
use crate::models::chat::ChatMessage;
use crate::repositories::chat_repo;
use crate::services::ai_service::{self, CompletionOutcome, RequestedToolCall, ToolSpec};
use crate::services::api_object_service;

const MAX_ROUNDS: u8 = 8;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn tool(name: &str, description: &str, schema: Value) -> ToolSpec {
    ToolSpec { name: name.to_string(), description: description.to_string(), input_schema: schema }
}

fn object_schema() -> Value {
    json!({"type": "object"})
}

// --- Records mode: re-exports mcp.rs's own 7 tools -------------------

fn record_tools() -> Vec<ToolSpec> {
    let object_key = json!({"type": "object", "properties": {
        "object_key": {"type": "string", "description": "An object key from list_objects, e.g. 'Company' or a custom object's key"},
    }});
    vec![
        tool("list_objects", "List every built-in and custom object this workspace exposes, with its label and whether it's custom.", object_schema()),
        tool("get_object_metadata", "Get an object's label and its custom field definitions (key, label, type, required). Arguments: object_key.", object_key.clone()),
        tool(
            "list_records",
            "List records for an object, paginated. Arguments: object_key (required), filter (object of exact-match field/value pairs), sort (array of field names, prefix '-' for descending), page, page_size (max 500).",
            object_schema(),
        ),
        tool("get_record", "Get a single record by id. Arguments: object_key, id.", object_schema()),
        tool(
            "create_record",
            "Create a record. Arguments: object_key, data (the record's fields as an object). Company, Contact, Product, Task and active Custom Objects only; Opportunity/Quote/Order/Invoice/Contract are read-only.",
            object_schema(),
        ),
        tool("update_record", "Update a record's fields by id. Arguments: object_key, id, data (only the fields being changed).", object_schema()),
        tool("archive_record", "Archive (soft-delete) a record by id. Arguments: object_key, id.", object_schema()),
    ]
}

fn required_str<'a>(args: &'a Value, key: &str) -> AppResult<&'a str> {
    args.get(key).and_then(Value::as_str).ok_or_else(|| AppError::Validation(format!("Missing required argument '{key}'")))
}

fn required_object<'a>(args: &'a Value, key: &str) -> AppResult<&'a Value> {
    let v = args.get(key).ok_or_else(|| AppError::Validation(format!("Missing required argument '{key}'")))?;
    if v.is_object() { Ok(v) } else { Err(AppError::Validation(format!("'{key}' must be an object"))) }
}

fn dispatch_record_tool(conn: &Connection, workspace_id: &str, name: &str, arguments: &Value) -> AppResult<Value> {
    match name {
        "list_objects" => api_object_service::list_object_keys(conn, workspace_id).and_then(|v| serde_json::to_value(v).map_err(ser_err)),
        "get_object_metadata" => {
            let key = required_str(arguments, "object_key")?;
            api_object_service::get_metadata(conn, workspace_id, key).and_then(|v| serde_json::to_value(v).map_err(ser_err))
        }
        "list_records" => {
            let key = required_str(arguments, "object_key")?;
            let query: crate::models::integration::ApiListQuery = serde_json::from_value(arguments.clone()).unwrap_or_default();
            api_object_service::list_records(conn, workspace_id, key, &query).and_then(|v| serde_json::to_value(v).map_err(ser_err))
        }
        "get_record" => {
            let key = required_str(arguments, "object_key")?;
            let id = required_str(arguments, "id")?;
            api_object_service::get_record(conn, workspace_id, key, id)
        }
        "create_record" => {
            let key = required_str(arguments, "object_key")?;
            let data = required_object(arguments, "data")?;
            api_object_service::create_record(conn, workspace_id, key, data, None)
        }
        "update_record" => {
            let key = required_str(arguments, "object_key")?;
            let id = required_str(arguments, "id")?;
            let data = required_object(arguments, "data")?;
            api_object_service::update_record(conn, workspace_id, key, id, data, None)
        }
        "archive_record" => {
            let key = required_str(arguments, "object_key")?;
            let id = required_str(arguments, "id")?;
            api_object_service::archive_record(conn, workspace_id, key, id, None).map(|_| json!({"archived": true}))
        }
        other => Err(AppError::Validation(format!("Unknown tool '{other}'"))),
    }
}

fn ser_err(e: serde_json::Error) -> AppError {
    AppError::Validation(format!("could not serialize result: {e}"))
}

// --- Admin mode: one list/create pair per subsystem -------------------

fn admin_tools() -> Vec<ToolSpec> {
    vec![
        tool("list_business_rules", "List Business Rules for an object. Arguments: entity_type (e.g. 'Company').", object_schema()),
        tool(
            "create_business_rule",
            "Create a Business Rule. Arguments match BusinessRuleInput: entity_type, name, description, match_type ('all'|'any'), priority, effective_start_date, effective_end_date, conditions (array of {field_source:'builtin'|'custom', field_key, operator, value}), actions (array of {action_type, target_field_key, target_field_source, action_value, message}).",
            object_schema(),
        ),
        tool("list_workflows", "List Workflow Automation rules for an object. Arguments: entity_type.", object_schema()),
        tool(
            "create_workflow",
            "Create a Workflow Automation rule. Arguments match WorkflowDefinitionInput: entity_type, name, description, trigger_type (e.g. 'record_created', 'status_changed', 'date_reached'), trigger_status, trigger_field_key, trigger_field_source, trigger_offset_days, match_type, priority, conditions, actions (array of {action_type, params_json}).",
            object_schema(),
        ),
        tool("list_custom_objects", "List this workspace's Custom Objects.", object_schema()),
        tool("create_custom_object", "Define a new Custom Object. Arguments: singular_label, plural_label, icon (an emoji), prefix (ID prefix), digits (ID number width).", object_schema()),
        tool("list_custom_fields", "List custom field definitions for an object. Arguments: entity_type.", object_schema()),
        tool(
            "create_custom_field",
            "Add a custom field to an object. Arguments match CustomFieldDefinitionInput: entity_type, label, field_type ('text'|'number'|'date'|'boolean'|'select'), options (for select), required, show_in_list, sort_order, is_searchable, is_filterable, is_reportable, default_value, is_unique, help_text, placeholder.",
            object_schema(),
        ),
        tool("list_relationships", "List Custom Relationships defined in this workspace.", object_schema()),
        tool(
            "create_relationship",
            "Connect two object types. Arguments match RelationshipDefinitionInput: source_entity_type, target_entity_type, relationship_type ('one_to_one'|'many_to_one'|'many_to_many'), forward_label, reverse_label, is_required, show_related_list, delete_behavior ('restrict'|'archive'), sort_order.",
            object_schema(),
        ),
        tool("list_status_transitions", "List allowed status/stage transitions for an object. Arguments: entity_type.", object_schema()),
        tool("create_status_transition", "Restrict a status/stage change. Arguments match StatusTransitionInput: entity_type, from_status (omit/null for 'from any'), to_status.", object_schema()),
        tool("list_connections", "List Integration Hub Connections (external systems this workspace talks to).", object_schema()),
        tool(
            "create_connection",
            "Create an Integration Hub Connection's shape - name, type and auth mode only. Arguments: name, connection_type ('rest'|'webhook'|'sftp'|'postgres'|'odata'|'smtp'), base_url, auth_mode ('none'|'api_key'|'basic'|'bearer'|'custom_header'|'oauth2_client_credentials'|'oauth2_authorization_code'), config_json (a JSON string of extra settings, or '{}'). Never pass a credential here - after creation, tell the admin to add it via Integration Hub -> Connections -> Edit, the same secure form every other credential uses.",
            object_schema(),
        ),
        tool("list_webhooks", "List outbound Webhooks configured in this workspace.", object_schema()),
        tool(
            "create_webhook",
            "Create an outbound Webhook. Arguments match WebhookInput: name, connection_id (an existing Connection's id - list_connections first), event_types (array, e.g. ['Company.created']), object_scope, filter_json, payload_version, retry_policy_json.",
            object_schema(),
        ),
        tool("list_integration_jobs", "List scheduled Integration Jobs.", object_schema()),
        tool(
            "create_integration_job",
            "Create a recurring Integration Job. Arguments match IntegrationJobInput: name, external_object_id, target_object_key, match_key, cursor_field, interval_minutes.",
            object_schema(),
        ),
        tool("list_api_clients", "List inbound API clients (REST API credentials for external systems calling into this workspace).", object_schema()),
        tool(
            "create_api_client",
            "Issue a new API client. Arguments match ApiClientInput: name, scopes (array, e.g. ['objects.read','objects.write']), allowed_cidr, owner_user_id. The generated secret is never shown in chat - retrieve it once from Integration Hub -> API Access, same as creating one there.",
            object_schema(),
        ),
        tool("list_dashboards", "List named Dashboard layouts.", object_schema()),
        tool("create_dashboard", "Create a Dashboard layout. Arguments match DashboardLayoutInput: name, initial_kpi_keys (array of KPI keys), app_id.", object_schema()),
        tool("list_apps", "List Apps (named groupings of objects/screens/a dashboard).", object_schema()),
        tool("create_app", "Create an App. Arguments match AppDefinitionInput: name, icon (an emoji), description.", object_schema()),
        tool("list_custom_reports", "List saved Custom Reports.", object_schema()),
        tool(
            "create_custom_report",
            "Create a Custom Report. Arguments match CustomReportInput: name, entity_type, group_by_source ('builtin'|'custom'), group_by_field, aggregate ('count'|'sum'), sum_field_key.",
            object_schema(),
        ),
        tool("list_saved_views", "List Saved Views for an object. Arguments: object_key.", object_schema()),
        tool(
            "create_saved_view",
            "Save a filter/sort/column combination as a named view. Arguments match SavedViewInput: object_key, name, visibility ('private'|'shared'), filters (object of field/value pairs), sort_field, sort_direction, columns (array), group_by_field.",
            object_schema(),
        ),
        tool("list_users", "List users in this workspace.", object_schema()),
        tool("create_user", "Invite a new user. Arguments match NewUser: username, display_name, password, roles (array of one or more of 'Administrator'|'Manager'|'Sales'|'Finance'|'ReadOnly').", object_schema()),
        tool("list_numbering", "List each object's effective ID-numbering format (admin override or built-in default).", object_schema()),
        tool("set_numbering", "Set a custom ID-numbering format for an object. Arguments match NumberingOverrideInput: entity_type, prefix, digits.", object_schema()),
        tool("get_workspace_profile", "Get the workspace's business profile (name, currency, locale, timezone, tax rate).", object_schema()),
        tool(
            "update_workspace_profile",
            "Update the workspace's business profile. Arguments are a partial WorkspaceUpdate - only the fields being changed (business_name, legal_name, business_address, phone, currency_code, locale, timezone, default_tax_rate_bp); anything omitted keeps its current value.",
            object_schema(),
        ),
        tool("list_connectors", "List Integration Hub Connectors (OpenAPI-imported external APIs). Read-only - importing one needs an uploaded spec file, not chat.", object_schema()),
    ]
}

fn to_val<T: serde::Serialize>(r: AppResult<T>) -> AppResult<Value> {
    r.and_then(|v| serde_json::to_value(v).map_err(ser_err))
}

#[allow(clippy::too_many_lines)]
fn dispatch_admin_tool(conn: &Connection, workspace_id: &str, actor: Option<&str>, master_key: &[u8; 32], name: &str, arguments: &Value) -> AppResult<Value> {
    use crate::models::business_rule::BusinessRuleInput;
    use crate::models::custom_field::CustomFieldDefinitionInput;
    use crate::models::custom_object::CustomObjectDefinitionInput;
    use crate::models::custom_report::CustomReportInput;
    use crate::models::integration::{ApiClientInput, ConnectionInput, IntegrationJobInput, WebhookInput};
    use crate::models::numbering_override::NumberingOverrideInput;
    use crate::models::relationship::RelationshipDefinitionInput;
    use crate::models::saved_view::SavedViewInput;
    use crate::models::status_transition::StatusTransitionInput;
    use crate::models::user::NewUser;
    use crate::models::workflow::WorkflowDefinitionInput;
    use crate::services::{
        api_client_service, app_service, business_rule_service, connection_service, custom_object_service, custom_report_service,
        dashboard_layout_service, integration_job_service, numbering_service, relationship_service, saved_view_service,
        status_transition_service, user_service, webhook_service, workflow_service,
    };

    fn from_args<T: serde::de::DeserializeOwned>(arguments: &Value) -> AppResult<T> {
        serde_json::from_value(arguments.clone()).map_err(|e| AppError::Validation(format!("Invalid arguments for this tool: {e}")))
    }

    match name {
        "list_business_rules" => {
            let entity_type = required_str(arguments, "entity_type")?;
            to_val(business_rule_service::list_rules(conn, workspace_id, entity_type, true))
        }
        "create_business_rule" => {
            let input: BusinessRuleInput = from_args(arguments)?;
            to_val(business_rule_service::create_rule(conn, workspace_id, &input, actor))
        }
        "list_workflows" => {
            let entity_type = required_str(arguments, "entity_type")?;
            to_val(workflow_service::list_rules(conn, workspace_id, entity_type, actor))
        }
        "create_workflow" => {
            let input: WorkflowDefinitionInput = from_args(arguments)?;
            to_val(workflow_service::create_rule(conn, workspace_id, &input, actor))
        }
        "list_custom_objects" => to_val(custom_object_service::list(conn, workspace_id, true)),
        "create_custom_object" => {
            let input: CustomObjectDefinitionInput = from_args(arguments)?;
            to_val(custom_object_service::create(conn, workspace_id, &input, actor))
        }
        "list_custom_fields" => {
            let entity_type = required_str(arguments, "entity_type")?;
            to_val(super::custom_field_service::list_definitions(conn, workspace_id, entity_type, true))
        }
        "create_custom_field" => {
            let input: CustomFieldDefinitionInput = from_args(arguments)?;
            to_val(super::custom_field_service::create_definition(conn, workspace_id, &input, actor))
        }
        "list_relationships" => to_val(relationship_service::list(conn, workspace_id, true)),
        "create_relationship" => {
            let input: RelationshipDefinitionInput = from_args(arguments)?;
            to_val(relationship_service::create(conn, workspace_id, &input, actor))
        }
        "list_status_transitions" => {
            let entity_type = required_str(arguments, "entity_type")?;
            to_val(status_transition_service::list(conn, workspace_id, entity_type, actor))
        }
        "create_status_transition" => {
            let input: StatusTransitionInput = from_args(arguments)?;
            to_val(status_transition_service::create(conn, workspace_id, &input, actor))
        }
        "list_connections" => to_val(connection_service::list_for_workspace(conn, workspace_id)),
        "create_connection" => {
            // The one non-negotiable boundary this module's own doc
            // comment names: secret_value is never read from a tool
            // call's arguments, even if present - always None here,
            // regardless of what the model or user supplied.
            let mut input: ConnectionInput = from_args(arguments)?;
            input.secret_value = None;
            to_val(connection_service::create(conn, workspace_id, master_key, &input, actor))
        }
        "list_webhooks" => to_val(webhook_service::list_for_workspace(conn, workspace_id)),
        "create_webhook" => {
            let input: WebhookInput = from_args(arguments)?;
            to_val(webhook_service::create(conn, workspace_id, master_key, &input, actor))
        }
        "list_integration_jobs" => to_val(integration_job_service::list_for_workspace(conn, workspace_id)),
        "create_integration_job" => {
            let input: IntegrationJobInput = from_args(arguments)?;
            to_val(integration_job_service::create(conn, workspace_id, &input, actor))
        }
        "list_api_clients" => to_val(api_client_service::list_for_workspace(conn, workspace_id)),
        "create_api_client" => {
            let input: ApiClientInput = from_args(arguments)?;
            let issued = api_client_service::create(conn, workspace_id, &input, actor)?;
            // "Shown once" convention - never into chat history at all.
            Ok(json!({"client": issued.client, "note": "Credential generated - retrieve it once from Integration Hub -> API Access, not shown in chat."}))
        }
        "list_dashboards" => to_val(dashboard_layout_service::list_layouts(conn, workspace_id)),
        "create_dashboard" => {
            let input: crate::models::dashboard_layout::DashboardLayoutInput = from_args(arguments)?;
            to_val(dashboard_layout_service::create_layout(conn, workspace_id, &input, actor))
        }
        "list_apps" => to_val(app_service::list(conn, workspace_id)),
        "create_app" => {
            let input: crate::models::app_definition::AppDefinitionInput = from_args(arguments)?;
            to_val(app_service::create(conn, workspace_id, &input, actor))
        }
        "list_custom_reports" => to_val(custom_report_service::list(conn, workspace_id)),
        "create_custom_report" => {
            let input: CustomReportInput = from_args(arguments)?;
            to_val(custom_report_service::create(conn, workspace_id, &input, actor))
        }
        "list_saved_views" => {
            let object_key = required_str(arguments, "object_key")?;
            to_val(saved_view_service::list_for_object(conn, workspace_id, object_key, actor))
        }
        "create_saved_view" => {
            let input: SavedViewInput = from_args(arguments)?;
            to_val(saved_view_service::create(conn, workspace_id, &input, actor))
        }
        "list_users" => to_val(user_service::list(conn, workspace_id)),
        "create_user" => {
            let input: NewUser = from_args(arguments)?;
            to_val(user_service::create(conn, workspace_id, &input, actor))
        }
        "list_numbering" => to_val(numbering_service::list_effective(conn, workspace_id, actor)),
        "set_numbering" => {
            let input: NumberingOverrideInput = from_args(arguments)?;
            to_val(numbering_service::set_override(conn, workspace_id, &input, actor))
        }
        "get_workspace_profile" => to_val(Ok(crate::repositories::workspace_repo::get_current(conn).map_err(AppError::from)?)),
        "update_workspace_profile" => {
            let current = crate::repositories::workspace_repo::get_current(conn)
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Workspace".into()))?;
            // Only the fields the model actually supplied override the
            // current profile - WorkspaceUpdate has no optional business_name/
            // currency_code/etc., so a partial chat request is merged onto
            // the current values rather than requiring the model to restate
            // everything it isn't changing.
            let mut update = crate::models::workspace::WorkspaceUpdate {
                business_name: current.business_name,
                legal_name: current.legal_name,
                business_address: current.business_address,
                phone: current.phone,
                currency_code: current.currency_code,
                locale: current.locale,
                timezone: current.timezone,
                default_tax_rate_bp: current.default_tax_rate_bp,
            };
            if let Some(v) = arguments.get("business_name").and_then(Value::as_str) { update.business_name = v.to_string(); }
            if let Some(v) = arguments.get("legal_name").and_then(Value::as_str) { update.legal_name = Some(v.to_string()); }
            if let Some(v) = arguments.get("business_address").and_then(Value::as_str) { update.business_address = Some(v.to_string()); }
            if let Some(v) = arguments.get("phone").and_then(Value::as_str) { update.phone = Some(v.to_string()); }
            if let Some(v) = arguments.get("currency_code").and_then(Value::as_str) { update.currency_code = v.to_string(); }
            if let Some(v) = arguments.get("locale").and_then(Value::as_str) { update.locale = v.to_string(); }
            if let Some(v) = arguments.get("timezone").and_then(Value::as_str) { update.timezone = v.to_string(); }
            if let Some(v) = arguments.get("default_tax_rate_bp").and_then(Value::as_i64) { update.default_tax_rate_bp = v; }
            to_val(super::workspace_service::update(conn, &update, actor))
        }
        "list_connectors" => to_val(super::connector_service::list_for_workspace(conn, workspace_id)),
        other => Err(AppError::Validation(format!("Unknown tool '{other}'"))),
    }
}

fn tools_for_mode(mode: &str) -> AppResult<Vec<ToolSpec>> {
    match mode {
        "records" => Ok(record_tools()),
        "admin" => Ok(admin_tools()),
        other => Err(AppError::Validation(format!("Unknown chat mode '{other}'"))),
    }
}

fn system_prompt(mode: &str) -> &'static str {
    match mode {
        "records" => {
            "You are Lanesra OS's assistant for finding and working with this workspace's records - \
             companies, contacts, opportunities, and every other built-in or custom object. Use the \
             tools available to look things up before answering, and to make the changes a person asks \
             for. Be concise. When you create, update, or archive something, say plainly what you did."
        }
        "admin" => {
            "You are Lanesra OS's assistant for administrators - helping build and configure workflows, \
             business rules, custom objects and fields, integrations, and the rest of the admin surface. \
             Use the tools available to look at what's already configured before proposing or creating \
             something new, and to make the changes an admin asks for. Never ask for or accept a \
             Connection's or API client's credential in chat - create_connection never takes one; tell \
             the admin to add it afterward through the existing secure form. Be concise, and say plainly \
             what you created."
        }
        _ => "",
    }
}

/// Redacts a `create_connection` tool call's `secret_value` argument out
/// of an assistant turn's raw provider content, wherever one shows up -
/// before it is ever persisted or echoed back on a later round. Reads
/// each provider's shape structurally rather than branching on provider:
/// Anthropic's is the `content` block array itself (a matching block has
/// `type: "tool_use"`, `name: "create_connection"`, and an `input`
/// object); an OpenAI-compatible provider's is `message.tool_calls`
/// (a matching entry has `function.name == "create_connection"` and
/// `function.arguments` as a JSON *string* needing its own parse/redact/
/// re-stringify). A no-op for anything that isn't a `create_connection`
/// call - most assistant turns pass through unchanged.
fn redact_secret_tool_calls(raw_assistant: &Value) -> Value {
    let Some(items) = raw_assistant.as_array() else { return raw_assistant.clone() };
    Value::Array(items.iter().map(redact_one_tool_call).collect())
}

const REDACTED_SECRET: &str = "<redacted - never stored in chat history>";

fn redact_one_tool_call(item: &Value) -> Value {
    let mut item = item.clone();
    let is_anthropic_create_connection =
        item.get("type").and_then(Value::as_str) == Some("tool_use") && item.get("name").and_then(Value::as_str) == Some("create_connection");
    if is_anthropic_create_connection {
        if let Some(input) = item.get_mut("input").and_then(Value::as_object_mut) {
            if input.contains_key("secret_value") {
                input.insert("secret_value".to_string(), json!(REDACTED_SECRET));
            }
        }
        return item;
    }
    let is_openai_create_connection = item.get("function").and_then(|f| f.get("name")).and_then(Value::as_str) == Some("create_connection");
    if is_openai_create_connection {
        if let Some(function) = item.get_mut("function").and_then(Value::as_object_mut) {
            if let Some(mut parsed) = function.get("arguments").and_then(Value::as_str).and_then(|s| serde_json::from_str::<Value>(s).ok()) {
                if let Some(obj) = parsed.as_object_mut() {
                    if obj.contains_key("secret_value") {
                        obj.insert("secret_value".to_string(), json!(REDACTED_SECRET));
                    }
                }
                if let Ok(new_args) = serde_json::to_string(&parsed) {
                    function.insert("arguments".to_string(), json!(new_args));
                }
            }
        }
    }
    item
}

/// Appends the user's message, then loops calling
/// `ai_service::complete_with_tools` and executing whatever it
/// requests, feeding results back as `role: "tool"` messages, until a
/// turn answers in plain text - capped at `MAX_ROUNDS`. Returns every
/// message this call appended (the user's own, plus everything the
/// assistant/tools produced), so the caller can render just the new
/// turns without re-fetching the whole conversation.
///
/// `mode == "admin"` requires Administrator, checked once here before
/// the loop starts - not relied on only implicitly inside each tool
/// (even though every admin service function already gates itself too).
pub async fn send_message(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], user_id: &str, mode: &str, text: &str) -> AppResult<Vec<ChatMessage>> {
    if text.trim().is_empty() {
        return Err(AppError::Validation("Say something first".into()));
    }
    if mode == "admin" {
        require_admin(conn, Some(user_id))?;
    }
    let tools = tools_for_mode(mode)?;
    let conversation = chat_repo::get_or_create_conversation(conn, workspace_id, user_id, mode)?;

    let mut appended = Vec::new();
    appended.push(chat_repo::append_message(conn, &conversation.id, "user", Some(text), None, None)?);

    for _round in 0..MAX_ROUNDS {
        let history = chat_repo::list_messages(conn, &conversation.id)?;
        let outcome = ai_service::complete_with_tools(conn, workspace_id, master_key, system_prompt(mode), &tools, &history).await?;
        match outcome {
            CompletionOutcome::Text(text) => {
                appended.push(chat_repo::append_message(conn, &conversation.id, "assistant", Some(&text), None, None)?);
                return Ok(appended);
            }
            CompletionOutcome::ToolCalls { raw_assistant, calls } => {
                // The one non-negotiable boundary this module's own doc
                // comment names, enforced a second time here: even though
                // dispatch_admin_tool's own create_connection arm already
                // never reads secret_value out of a tool call's arguments,
                // the *raw* assistant turn - echoed back to the provider
                // verbatim on every later round, and otherwise persisted
                // as-is - would still carry whatever secret a model put in
                // its own tool call if this didn't redact it first. Applies
                // structurally to whichever provider's raw shape this is
                // (see `redact_secret_tool_calls`'s own doc comment), not
                // only in admin mode, since it's a no-op for any tool call
                // that isn't create_connection.
                let persisted_assistant = redact_secret_tool_calls(&raw_assistant);
                appended.push(chat_repo::append_message(conn, &conversation.id, "assistant", None, Some(&persisted_assistant), None)?);
                for call in calls {
                    let result = execute_tool(conn, workspace_id, master_key, Some(user_id), mode, &call);
                    let content = match result {
                        Ok(value) => value.to_string(),
                        Err(e) => format!("Error: {e}"),
                    };
                    appended.push(chat_repo::append_message(conn, &conversation.id, "tool", Some(&content), None, Some(&call.id))?);
                }
            }
        }
    }
    Err(AppError::Validation("This is taking more steps than expected - try asking again, or break the request into smaller parts.".into()))
}

fn execute_tool(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], actor: Option<&str>, mode: &str, call: &RequestedToolCall) -> AppResult<Value> {
    match mode {
        "records" => dispatch_record_tool(conn, workspace_id, &call.name, &call.arguments),
        "admin" => dispatch_admin_tool(conn, workspace_id, actor, master_key, &call.name, &call.arguments),
        other => Err(AppError::Validation(format!("Unknown chat mode '{other}'"))),
    }
}

pub fn get_history(conn: &Connection, workspace_id: &str, user_id: &str, mode: &str) -> AppResult<Vec<ChatMessage>> {
    let conversation = chat_repo::get_or_create_conversation(conn, workspace_id, user_id, mode)?;
    Ok(chat_repo::list_messages(conn, &conversation.id)?)
}
