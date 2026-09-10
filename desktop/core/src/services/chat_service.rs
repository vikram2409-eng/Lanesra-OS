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

use std::cell::Cell;

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::domain::{AppError, AppResult};
use crate::models::ai_agent::AiAgentDefinition;
use crate::models::chat::ChatMessage;
use crate::repositories::{ai_agent_repo, ai_token_usage_repo, chat_repo};
use crate::services::ai_gateway_service;
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
        tool("list_ai_agents", "List AI Agent Foundry agents defined in this workspace.", object_schema()),
        tool(
            "create_ai_agent",
            "Create a named AI Agent. Arguments match AiAgentInput: name, description, icon (an emoji), system_prompt (its persona/instructions), action_names (array of tool names this agent may call - any name from this very tool list, e.g. 'list_records','create_record','list_business_rules'; including any admin-catalog name makes the agent Administrator-only), delegate_agent_ids (array of other active agent ids it may call via delegate_to_agent), skill_ids (array of ai_skills ids to attach - see list_ai_skills).",
            object_schema(),
        ),
        tool("list_ai_skills", "List the AI Agent Foundry's reusable Skills library.", object_schema()),
        tool(
            "create_ai_skill",
            "Create a reusable Skill an Agent can attach and load on demand. Arguments match AiSkillInput: name, description (what a model sees before deciding to use it), instructions_md (the full content, only loaded when used).",
            object_schema(),
        ),
        tool("list_ai_agent_pipelines", "List Agent Pipelines (an ordered chain of Agents) defined in this workspace.", object_schema()),
        tool(
            "create_ai_agent_pipeline",
            "Create an Agent Pipeline. Arguments match AiAgentPipelineInput: name, description, steps (array of {agent_id, input_template} in order - input_template may reference {{previous_output}} for step 2+, or {{trigger_input}} for step 1).",
            object_schema(),
        ),
        tool("run_ai_agent", "Run a single AI Agent once with the given input and return its final answer. Arguments: agent_id, input.", object_schema()),
        tool("run_ai_agent_pipeline", "Run an Agent Pipeline once with the given trigger input and return the overall result. Arguments: pipeline_id, input.", object_schema()),
    ]
}

/// AI & Agentic Layer, Phase 6: which of the two fixed catalogs above a
/// tool name belongs to - the AI Agent Foundry's per-agent `action_names`
/// checklist is picked straight from these names, not a third catalog, so
/// a named Agent's own tool routing (`dispatch_agent_tool`) needs to know
/// which dispatcher a given name goes to instead of assuming one fixed
/// `mode`. Returns `None` for an unknown name (rejected at
/// `ai_agent_service`'s validation, before it ever reaches here).
pub(crate) fn tool_source(name: &str) -> Option<&'static str> {
    if record_tools().iter().any(|t| t.name == name) {
        Some("record")
    } else if admin_tools().iter().any(|t| t.name == name) {
        Some("admin")
    } else {
        None
    }
}

/// An Agent needs Administrator the moment any one of its own
/// `action_names` resolves to an admin-catalog tool - checked once before
/// its loop starts, the same `mode == "admin"` gate `send_message` already
/// has, just keyed off a computed set instead of a literal mode string.
pub(crate) fn agent_requires_admin(action_names: &[String]) -> bool {
    action_names.iter().any(|n| tool_source(n) == Some("admin"))
}

fn to_val<T: serde::Serialize>(r: AppResult<T>) -> AppResult<Value> {
    r.and_then(|v| serde_json::to_value(v).map_err(ser_err))
}

#[allow(clippy::too_many_lines)]
async fn dispatch_admin_tool(conn: &Connection, workspace_id: &str, actor: Option<&str>, master_key: &[u8; 32], name: &str, arguments: &Value) -> AppResult<Value> {
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
        "list_ai_agents" => to_val(super::ai_agent_service::list(conn, workspace_id, true)),
        "create_ai_agent" => {
            let input: crate::models::ai_agent::AiAgentInput = from_args(arguments)?;
            to_val(super::ai_agent_service::create(conn, workspace_id, &input, actor))
        }
        "list_ai_skills" => to_val(super::ai_agent_service::list_skills(conn, workspace_id, true)),
        "create_ai_skill" => {
            let input: crate::models::ai_agent::AiSkillInput = from_args(arguments)?;
            to_val(super::ai_agent_service::create_skill(conn, workspace_id, &input, actor))
        }
        "list_ai_agent_pipelines" => to_val(super::ai_orchestration_service::list_pipelines(conn, workspace_id, true)),
        "create_ai_agent_pipeline" => {
            let input: crate::models::ai_agent_pipeline::AiAgentPipelineInput = from_args(arguments)?;
            to_val(super::ai_orchestration_service::create_pipeline(conn, workspace_id, &input, actor))
        }
        "run_ai_agent" => {
            let agent_id = required_str(arguments, "agent_id")?;
            let input = required_str(arguments, "input")?;
            to_val(super::ai_orchestration_service::run_manual(conn, workspace_id, master_key, "agent", agent_id, input, actor).await)
        }
        "run_ai_agent_pipeline" => {
            let pipeline_id = required_str(arguments, "pipeline_id")?;
            let input = required_str(arguments, "input")?;
            to_val(super::ai_orchestration_service::run_manual(conn, workspace_id, master_key, "pipeline", pipeline_id, input, actor).await)
        }
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
    let conversation = chat_repo::get_or_create_conversation(conn, workspace_id, user_id, mode, "")?;

    let mut appended = Vec::new();
    appended.push(chat_repo::append_message(conn, &conversation.id, "user", Some(text), None, None)?);

    for _round in 0..MAX_ROUNDS {
        let history = chat_repo::list_messages(conn, &conversation.id)?;
        let (outcome, usage) = ai_service::complete_with_tools(conn, workspace_id, master_key, system_prompt(mode), &tools, &history).await?;
        // Phase 7a: recorded for Gateway health-view visibility even on
        // this plain (no-agent) path - agent_id "" is the same "not an
        // agent run" sentinel `ai_token_usage_repo`'s own doc comment
        // describes, matching `chat_conversations.agent_id`'s convention.
        let _ = ai_token_usage_repo::increment(conn, workspace_id, "", user_id, usage.input_tokens, usage.output_tokens);
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
                    let result = execute_tool(conn, workspace_id, master_key, Some(user_id), mode, &call).await;
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

async fn execute_tool(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], actor: Option<&str>, mode: &str, call: &RequestedToolCall) -> AppResult<Value> {
    match mode {
        "records" => dispatch_record_tool(conn, workspace_id, &call.name, &call.arguments),
        "admin" => dispatch_admin_tool(conn, workspace_id, actor, master_key, &call.name, &call.arguments).await,
        other => Err(AppError::Validation(format!("Unknown chat mode '{other}'"))),
    }
}

pub fn get_history(conn: &Connection, workspace_id: &str, user_id: &str, mode: &str) -> AppResult<Vec<ChatMessage>> {
    let conversation = chat_repo::get_or_create_conversation(conn, workspace_id, user_id, mode, "")?;
    Ok(chat_repo::list_messages(conn, &conversation.id)?)
}

// === AI & Agentic Layer, Phase 6: the AI Agent Foundry =====================
//
// A named `AiAgentDefinition` (`ai_agent_service.rs` owns its CRUD) is a
// persona layered over a per-agent checklist of individual tool names
// (`action_names`, picked straight from `record_tools()`/`admin_tools()`
// above - see `tool_source`), persistent Memory, a Skills library, and
// delegation to other agents (hierarchy). `run_agent_once` is the one
// shared tool-calling-loop core: it works entirely over an in-memory
// `Vec<ChatMessage>` (no `chat_repo` persistence of its own), so both
// `send_agent_message` below (persists the result into a real
// conversation, same as `send_message`) and, from Phase 6b, a Pipeline
// step or a Trigger-fired run (records only a step summary, not a full
// transcript) can reuse it without forcing one persistence shape on both.

thread_local! {
    // Bounds Agent-to-Agent delegation depth - the exact same RAII-guard
    // shape `workflow_service::MAX_WORKFLOW_DEPTH`/`DepthGuard` already
    // uses for its own recursion (self-referential workflow chains). A
    // depth guard, not full cycle detection at save time, is the actual
    // safety net that matters here (see migration 0038's own comment).
    static AGENT_DELEGATION_DEPTH: Cell<u8> = const { Cell::new(0) };
}
const MAX_DELEGATION_DEPTH: u8 = 4;

struct DelegationDepthGuard;
impl DelegationDepthGuard {
    fn enter() -> Option<DelegationDepthGuard> {
        let depth = AGENT_DELEGATION_DEPTH.with(Cell::get);
        if depth >= MAX_DELEGATION_DEPTH {
            return None;
        }
        AGENT_DELEGATION_DEPTH.with(|d| d.set(depth + 1));
        Some(DelegationDepthGuard)
    }
}
impl Drop for DelegationDepthGuard {
    fn drop(&mut self) {
        AGENT_DELEGATION_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    }
}

/// A message this loop produced itself, never persisted - `id`/
/// `conversation_id`/`created_at` are meaningless placeholders; only
/// `role`/`content`/`tool_calls`/`tool_call_id` matter, since those are
/// all `ai_service::complete_with_tools`'s own message-building functions
/// ever read from a `ChatMessage`.
fn synthetic_message(role: &str, content: Option<&str>, tool_calls: Option<&Value>, tool_call_id: Option<&str>) -> ChatMessage {
    ChatMessage {
        id: String::new(),
        conversation_id: String::new(),
        role: role.to_string(),
        content: content.map(String::from),
        tool_calls: tool_calls.cloned(),
        tool_call_id: tool_call_id.map(String::from),
        created_at: String::new(),
    }
}

/// This agent's own tool list: whichever of its `action_names` actually
/// resolve to a real tool (an unknown name was already rejected at
/// `ai_agent_service` validation, but a name can also go stale if the
/// tool it named is later removed from the fixed catalogs - silently
/// dropped here rather than erroring, same "the agent still works, just
/// with one fewer action" tolerance a removed custom field's rules
/// already get), plus the always-available `update_memory`, plus
/// `use_skill`/`delegate_to_agent` only when this agent actually has
/// skills/delegates attached (no point offering a tool with nothing
/// behind it).
fn agent_tools(conn: &Connection, agent: &AiAgentDefinition) -> AppResult<Vec<ToolSpec>> {
    let all_record = record_tools();
    let all_admin = admin_tools();
    let mut tools: Vec<ToolSpec> = agent
        .action_names
        .iter()
        .filter_map(|name| all_record.iter().chain(all_admin.iter()).find(|t| &t.name == name).cloned())
        .collect();

    tools.push(tool(
        "update_memory",
        "Overwrite your own persistent memory with the complete updated document (not a diff - write the whole thing each time, including what you're keeping from before). Arguments: content.",
        object_schema(),
    ));

    if !agent.skill_ids.is_empty() {
        let mut names = Vec::new();
        for skill_id in &agent.skill_ids {
            if let Some(skill) = ai_agent_repo::get_skill(conn, skill_id).map_err(AppError::from)? {
                names.push(skill.name);
            }
        }
        tools.push(tool(
            "use_skill",
            &format!("Load the full instructions for one of your attached skills. Arguments: name (one of: {}).", names.join(", ")),
            object_schema(),
        ));
    }

    if !agent.delegate_agent_ids.is_empty() {
        let mut names = Vec::new();
        for delegate_id in &agent.delegate_agent_ids {
            if let Some(delegate) = ai_agent_repo::get(conn, delegate_id).map_err(AppError::from)? {
                if delegate.is_active {
                    names.push(delegate.name);
                }
            }
        }
        tools.push(tool(
            "delegate_to_agent",
            &format!(
                "Delegate a task to one of your sub-agents and get its final answer back. Arguments: agent_name (one of: {}), input (the task or question to give it).",
                names.join(", ")
            ),
            object_schema(),
        ));
    }

    Ok(tools)
}

/// This agent's persona, plus its persistent Memory (if it's written
/// anything yet) and a short name+description catalog of its attached
/// Skills - the same "short description up front, full content only on
/// demand via use_skill" shape this session's own Skill tool uses.
fn agent_system_prompt(conn: &Connection, agent: &AiAgentDefinition) -> AppResult<String> {
    let mut prompt = agent.system_prompt.clone();
    if !agent.memory_md.trim().is_empty() {
        prompt.push_str("\n\nYour persistent memory from prior runs (revise it with update_memory whenever something worth remembering happens):\n\n");
        prompt.push_str(&agent.memory_md);
    }
    if !agent.skill_ids.is_empty() {
        prompt.push_str("\n\nSkills available to you - call use_skill by name when one is relevant:\n");
        for skill_id in &agent.skill_ids {
            if let Some(skill) = ai_agent_repo::get_skill(conn, skill_id).map_err(AppError::from)? {
                prompt.push_str(&format!("- {}: {}\n", skill.name, skill.description));
            }
        }
    }
    Ok(prompt)
}

/// Dispatches one requested tool call for a named Agent - the three
/// Foundry-specific tools directly, anything else routed to whichever of
/// `dispatch_record_tool`/`dispatch_admin_tool` `tool_source` says it
/// belongs to (an Agent mixes freely from both catalogs, unlike the fixed
/// `"records"`/`"admin"` assistants above). `delegate_to_agent` is the
/// one genuinely recursive case - boxed since `run_agent_once` calling
/// this calling `run_agent_once` again is otherwise an infinitely-sized
/// future.
fn execute_agent_tool<'a>(
    conn: &'a Connection,
    workspace_id: &'a str,
    master_key: &'a [u8; 32],
    actor: Option<&'a str>,
    agent: &'a AiAgentDefinition,
    call: &'a RequestedToolCall,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Value>> + 'a>> {
    Box::pin(async move {
        match call.name.as_str() {
            "update_memory" => {
                let content = required_str(&call.arguments, "content")?;
                ai_agent_repo::update_memory(conn, &agent.id, content).map_err(AppError::from)?;
                Ok(json!({"memory_updated": true}))
            }
            "use_skill" => {
                let skill_name = required_str(&call.arguments, "name")?;
                for skill_id in &agent.skill_ids {
                    if let Some(skill) = ai_agent_repo::get_skill(conn, skill_id).map_err(AppError::from)? {
                        if skill.name == skill_name {
                            return Ok(json!({"instructions": skill.instructions_md}));
                        }
                    }
                }
                Err(AppError::Validation(format!("'{skill_name}' isn't one of this agent's attached skills")))
            }
            "delegate_to_agent" => {
                let agent_name = required_str(&call.arguments, "agent_name")?;
                let input = required_str(&call.arguments, "input")?;
                let mut target = None;
                for delegate_id in &agent.delegate_agent_ids {
                    if let Some(candidate) = ai_agent_repo::get(conn, delegate_id).map_err(AppError::from)? {
                        if candidate.name == agent_name && candidate.is_active {
                            target = Some(candidate);
                            break;
                        }
                    }
                }
                let target = target.ok_or_else(|| AppError::Validation(format!("'{agent_name}' isn't one of this agent's delegate sub-agents")))?;
                let seed = vec![synthetic_message("user", Some(input), None, None)];
                let outcome = run_agent_once(conn, workspace_id, master_key, actor, &target, seed).await?;
                Ok(json!({"answer": outcome.final_text}))
            }
            other => match tool_source(other) {
                Some("record") => dispatch_record_tool(conn, workspace_id, other, &call.arguments),
                Some("admin") => dispatch_admin_tool(conn, workspace_id, actor, master_key, other, &call.arguments).await,
                _ => Err(AppError::Validation(format!("Unknown tool '{other}'"))),
            },
        }
    })
}

/// What `run_agent_once` produced - `produced` is every message the loop
/// generated (assistant/tool turns), in order, NOT including
/// `seed_history` - the caller decides whether to persist all of them
/// (`send_agent_message` does) or just log a summary (a Pipeline/Trigger
/// run, Phase 6b).
pub struct AgentRunOutcome {
    pub final_text: String,
    pub produced: Vec<ChatMessage>,
}

/// The tool-calling loop, run once against a single `AiAgentDefinition`,
/// starting from `seed_history` (which already includes whatever prior
/// conversation and the newest user turn belong in context - an empty
/// history plus one seed message for a one-shot Pipeline/Trigger run, or
/// a full persisted conversation plus the newest message for
/// `send_agent_message`). Entirely in-memory - see this section's own
/// doc comment for why persistence is the caller's job, not this
/// function's.
/// Convenience wrapper over `run_agent_once` for a caller that only ever
/// needs a single fresh input - Phase 6b's Pipeline/Trigger runs, which
/// aren't a multi-turn conversation and never seed more than one message.
pub async fn run_agent_once_with_text(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor: Option<&str>,
    agent: &AiAgentDefinition,
    input_text: &str,
) -> AppResult<AgentRunOutcome> {
    let seed = vec![synthetic_message("user", Some(input_text), None, None)];
    run_agent_once(conn, workspace_id, master_key, actor, agent, seed).await
}

pub async fn run_agent_once(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor: Option<&str>,
    agent: &AiAgentDefinition,
    seed_history: Vec<ChatMessage>,
) -> AppResult<AgentRunOutcome> {
    let _depth_guard = DelegationDepthGuard::enter()
        .ok_or_else(|| AppError::Validation("Delegation depth limit reached - simplify this agent's delegation chain".into()))?;
    if agent_requires_admin(&agent.action_names) {
        require_admin(conn, actor)?;
    }
    let tools = agent_tools(conn, agent)?;
    let system_prompt = agent_system_prompt(conn, agent)?;

    let mut history = seed_history;
    let mut produced = Vec::new();

    for _round in 0..MAX_ROUNDS {
        // Phase 7a: every agent run goes through the Gateway, not
        // straight to `ai_service::complete_with_tools` - it resolves
        // this agent's `model_routing` policy (primary/fallback/
        // local_fallback failover, System/Agent budget ceilings, DLP
        // forced air-gapping) and records real token usage itself; an
        // agent with no routing configured gets exactly its pre-7a
        // behavior back (see `ai_gateway_service`'s own doc comment).
        let gateway_outcome = ai_gateway_service::dispatch(conn, workspace_id, master_key, agent, actor, &system_prompt, &tools, &history).await?;
        let outcome = gateway_outcome.outcome;
        match outcome {
            CompletionOutcome::Text(text) => {
                produced.push(synthetic_message("assistant", Some(&text), None, None));
                return Ok(AgentRunOutcome { final_text: text, produced });
            }
            CompletionOutcome::ToolCalls { raw_assistant, calls } => {
                let persisted_assistant = redact_secret_tool_calls(&raw_assistant);
                let assistant_msg = synthetic_message("assistant", None, Some(&persisted_assistant), None);
                history.push(assistant_msg.clone());
                produced.push(assistant_msg);
                for call in calls {
                    let result = execute_agent_tool(conn, workspace_id, master_key, actor, agent, &call).await;
                    let content = match result {
                        Ok(value) => value.to_string(),
                        Err(e) => format!("Error: {e}"),
                    };
                    let tool_msg = synthetic_message("tool", Some(&content), None, Some(&call.id));
                    history.push(tool_msg.clone());
                    produced.push(tool_msg);
                }
            }
        }
    }
    Err(AppError::Validation("This is taking more steps than expected - try asking again, or break the request into smaller parts.".into()))
}

/// Chatting with one specific named Agent - the third `chat_conversations`
/// mode alongside `send_message`'s fixed `"records"`/`"admin"`. A close
/// structural sibling of `send_message` (same "persist the user's message,
/// run the loop, persist what it produced" shape), not a forced
/// abstraction over it - see this section's own doc comment.
pub async fn send_agent_message(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], user_id: &str, agent_id: &str, text: &str) -> AppResult<Vec<ChatMessage>> {
    if text.trim().is_empty() {
        return Err(AppError::Validation("Say something first".into()));
    }
    let agent = ai_agent_repo::get(conn, agent_id).map_err(AppError::from)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    if !agent.is_active {
        return Err(AppError::Validation("This agent has been deactivated".into()));
    }
    if agent_requires_admin(&agent.action_names) {
        require_admin(conn, Some(user_id))?;
    }

    let conversation = chat_repo::get_or_create_conversation(conn, workspace_id, user_id, "agent", agent_id)?;
    let mut appended = Vec::new();
    appended.push(chat_repo::append_message(conn, &conversation.id, "user", Some(text), None, None)?);

    let history = chat_repo::list_messages(conn, &conversation.id)?;
    let outcome = run_agent_once(conn, workspace_id, master_key, Some(user_id), &agent, history).await?;
    for msg in outcome.produced {
        appended.push(chat_repo::append_message(
            conn,
            &conversation.id,
            &msg.role,
            msg.content.as_deref(),
            msg.tool_calls.as_ref(),
            msg.tool_call_id.as_deref(),
        )?);
    }
    Ok(appended)
}

pub fn get_agent_history(conn: &Connection, workspace_id: &str, user_id: &str, agent_id: &str) -> AppResult<Vec<ChatMessage>> {
    let conversation = chat_repo::get_or_create_conversation(conn, workspace_id, user_id, "agent", agent_id)?;
    Ok(chat_repo::list_messages(conn, &conversation.id)?)
}
