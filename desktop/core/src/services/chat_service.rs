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
//! plain text - capped at the run's round limit (WP-0121: 8 by default, set
//! per agent up to 25). A run that reaches its round or token limit is asked
//! once for a final answer with tools switched off, and that answer is kept
//! as a partial result instead of being thrown away.

use std::cell::Cell;

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::ai::AgentRunLimits;
use crate::models::ai_agent::{validate_output_schema, AiAgentDefinition};
use crate::models::ai_agent_policy::PolicyDecision;
use crate::models::ai_memory::{AgentMemoryContext, MemoryItemInput};
use crate::models::chat::ChatMessage;
use crate::repositories::action_type_repo;
use crate::repositories::{
    ai_agent_repo, ai_agent_version_repo, ai_token_usage_repo, audit_repo, chat_repo,
};
use crate::services::action_type_service;
use crate::services::ai_gateway_service;
use crate::services::ai_knowledge_service;
use crate::services::ai_memory_service;
use crate::services::ai_service::{
    self, CallOptions, CompletionOutcome, RequestedToolCall, ToolSpec,
};
use crate::services::api_object_service;
use crate::services::policy_engine_service;
use crate::services::tool_limits_service;
use crate::services::trace_service::{self, SpanStatus};

/// Why a run ended early with a partial answer (ADR-0007 §4 stop reasons).
pub const STOP_LIMIT_ROUNDS: &str = "limit_rounds";
pub const STOP_LIMIT_TOKENS: &str = "limit_tokens";

/// The system prompt for the one final-answer call a run makes when it
/// reaches a limit: the run's own prompt plus catalog `loop/final-answer`.
/// The instruction goes in the system prompt, not the conversation, so no
/// message the person did not write is added to their history.
fn final_answer_prompt(
    system_prompt: &str,
    stop_reason: &str,
    limits: &AgentRunLimits,
) -> AppResult<String> {
    let reason = match stop_reason {
        STOP_LIMIT_TOKENS => format!(
            "it used its {}-token limit",
            limits.run_token_limit.unwrap_or_default()
        ),
        _ => format!(
            "it used all {} of its tool-calling rounds",
            limits.max_rounds
        ),
    };
    let (section, _) = crate::prompts::render_active("loop/final-answer", &[("reason", &reason)])
        .map_err(AppError::Validation)?;
    Ok(format!("{system_prompt}\n\n{section}"))
}

/// The text of a final-answer reply. Tools are off for that call, so a
/// model that still asks for one gets nothing run and its text is empty.
fn final_answer_text(outcome: CompletionOutcome) -> String {
    match outcome {
        CompletionOutcome::Text(text) => text,
        CompletionOutcome::ToolCalls { .. } => String::new(),
    }
}
/// Malformed tool calls one run may make before it stops (WP-0122): each
/// one is answered with what was wrong, never run, and counts toward this.
pub const MAX_MALFORMED_TOOL_CALLS: u32 = 3;

/// The tool result a malformed call gets instead of running.
fn malformed_tool_result(problem: &str) -> String {
    format!(
        "Error: validation.arguments: {problem}. The tool was not run. Call it again with a JSON object that matches its parameters."
    )
}

fn too_many_malformed_calls() -> AppError {
    AppError::Classified {
        class: crate::domain::ErrorClass::Validation,
        code: "validation.arguments",
        message: format!(
            "The model sent {MAX_MALFORMED_TOOL_CALLS} malformed tool calls, so the run stopped"
        ),
        retry_after_ms: None,
        http_status: None,
    }
}

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn tool(name: &str, description: &str, schema: Value) -> ToolSpec {
    ToolSpec {
        name: name.to_string(),
        description: description.to_string(),
        input_schema: schema,
    }
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
        tool(
            "search_records",
            "Ranked full-text search across Custom Object records - finds records by relevance to free text, not just exact-match filters. Arguments: query (required), object_key (optional, scopes to one custom object), limit (optional, max 25). Built-in objects (Company, Contact, ...) aren't covered - use list_records for those.",
            object_schema(),
        ),
        tool(
            "get_related_records",
            "Get every record linked to one record via a Custom Relationship, in either direction (e.g. a Company's related Contacts and Opportunities). A relationship is a link, not a field on the record itself - get_object_metadata's own 'relationships' list names which relationships an object participates in and what each one is called; this tool follows them for one specific record. Arguments: object_key, id.",
            object_schema(),
        ),
        tool(
            "get_platform_overview",
            "Orientation for how Lanesra OS itself is built and how its pieces fit together - the primitives an app is assembled from, the recommended order to build a new business capability, and a live summary (object/relationship/app counts, the actual object list) of this specific workspace. Call this before proposing or building anything unfamiliar, rather than guessing at how the platform works. No arguments.",
            object_schema(),
        ),
        tool(
            "semantic_search_records",
            "Meaning-based search across Custom Object records - finds records related to a question or description even when they don't share its exact words (unlike search_records, which ranks by keyword/phrase match). Needs an OpenAI-compatible or Google Gemini provider configured with vector search reindexed (Admin -> LLM & MCP -> Vector Search) - fails clearly if neither is true yet. Arguments: query (required), object_key (optional, scopes to one custom object), limit (optional, max 25).",
            object_schema(),
        ),
    ]
}

fn required_str<'a>(args: &'a Value, key: &str) -> AppResult<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Validation(format!("Missing required argument '{key}'")))
}

fn required_object<'a>(args: &'a Value, key: &str) -> AppResult<&'a Value> {
    let v = args
        .get(key)
        .ok_or_else(|| AppError::Validation(format!("Missing required argument '{key}'")))?;
    if v.is_object() {
        Ok(v)
    } else {
        Err(AppError::Validation(format!("'{key}' must be an object")))
    }
}

/// The plain validated capability string (`models::access_role::Capability`'s
/// own 5 values, as everything else in this codebase keys them) a record
/// tool exercises - `None` for `list_objects`/`get_platform_overview`,
/// which name no single object type to check a scope against.
fn implied_capability(tool_name: &str) -> Option<&'static str> {
    match tool_name {
        "create_record" => Some("create"),
        "update_record" => Some("update"),
        "archive_record" => Some("delete"),
        "list_records"
        | "get_record"
        | "search_records"
        | "get_related_records"
        | "semantic_search_records"
        | "get_object_metadata" => Some("read"),
        _ => None,
    }
}

/// Ontology Layer epic, issue #344: `agent.object_type_scopes` empty
/// means unscoped - every record tool call dispatches exactly as it did
/// before this feature, by construction (this function is a no-op). Once
/// an admin adds a scope entry, every record tool naming an `object_key`
/// is checked against it - fail-closed, the same stance this codebase's
/// other gap-closing phases (e.g. Agent Access Governance, issue #245)
/// already took: no `object_key` argument at all is a deny too, not an
/// unchecked pass-through, since a scoped agent must always declare
/// which object type it's touching to be checked against its own scope.
fn check_object_type_scope(
    agent: &AiAgentDefinition,
    tool_name: &str,
    arguments: &Value,
) -> AppResult<()> {
    if agent.object_type_scopes.is_empty() {
        return Ok(());
    }
    let Some(capability) = implied_capability(tool_name) else {
        return Ok(());
    };
    let object_key = arguments.get("object_key").and_then(Value::as_str);
    let Some(object_key) = object_key else {
        return Err(AppError::Validation(format!("'{tool_name}' needs an object_key - this agent is scoped to specific object types and cannot be called without naming one")));
    };
    let scope = agent
        .object_type_scopes
        .iter()
        .find(|s| s.object_type == object_key);
    match scope {
        Some(s) if s.capabilities.iter().any(|c| c == capability) => Ok(()),
        Some(_) => Err(AppError::Validation(format!(
            "This agent's Ontology scope for '{object_key}' does not grant '{capability}'"
        ))),
        None => Err(AppError::Validation(format!(
            "This agent is not scoped to act on '{object_key}'"
        ))),
    }
}

/// `access_actor` is Agent Access Governance's (issue #245) effective
/// write-actor - see `effective_write_actor`'s own doc comment. `None`
/// unless a policy has opted this call into real Access Control v1
/// enforcement; passed straight through to `api_object_service`'s own
/// already-existing `actor_user_id` parameter on the three record-write
/// branches below, unchanged everywhere else.
async fn dispatch_record_tool(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    name: &str,
    arguments: &Value,
    access_actor: Option<&str>,
) -> AppResult<Value> {
    match name {
        "list_objects" => api_object_service::list_object_keys(conn, workspace_id)
            .and_then(|v| serde_json::to_value(v).map_err(ser_err)),
        "get_object_metadata" => {
            let key = required_str(arguments, "object_key")?;
            api_object_service::get_metadata(conn, workspace_id, key)
                .and_then(|v| serde_json::to_value(v).map_err(ser_err))
        }
        "list_records" => {
            let key = required_str(arguments, "object_key")?;
            let query: crate::models::integration::ApiListQuery =
                serde_json::from_value(arguments.clone()).unwrap_or_default();
            api_object_service::list_records(conn, workspace_id, key, &query)
                .and_then(|v| serde_json::to_value(v).map_err(ser_err))
        }
        "get_record" => {
            let key = required_str(arguments, "object_key")?;
            let id = required_str(arguments, "id")?;
            api_object_service::get_record(conn, workspace_id, key, id)
        }
        "create_record" => {
            let key = required_str(arguments, "object_key")?;
            let data = required_object(arguments, "data")?;
            api_object_service::create_record(conn, workspace_id, key, data, access_actor)
        }
        "update_record" => {
            let key = required_str(arguments, "object_key")?;
            let id = required_str(arguments, "id")?;
            let data = required_object(arguments, "data")?;
            api_object_service::update_record(conn, workspace_id, key, id, data, access_actor)
        }
        "archive_record" => {
            let key = required_str(arguments, "object_key")?;
            let id = required_str(arguments, "id")?;
            api_object_service::archive_record(conn, workspace_id, key, id, access_actor)
                .map(|_| json!({"archived": true}))
        }
        "search_records" => {
            let query = required_str(arguments, "query")?;
            let object_key = arguments.get("object_key").and_then(Value::as_str);
            let limit = arguments.get("limit").and_then(Value::as_i64).unwrap_or(10);
            let hits = crate::services::search_service::search_custom_records(
                conn,
                workspace_id,
                query,
                object_key,
                limit,
            )?;
            serde_json::to_value(hits).map_err(ser_err)
        }
        "get_related_records" => {
            let key = required_str(arguments, "object_key")?;
            let id = required_str(arguments, "id")?;
            api_object_service::related_records(conn, workspace_id, key, id)
                .and_then(|v| serde_json::to_value(v).map_err(ser_err))
        }
        "get_platform_overview" => platform_overview(conn, workspace_id),
        "semantic_search_records" => {
            let query = required_str(arguments, "query")?;
            let object_key = arguments.get("object_key").and_then(Value::as_str);
            let limit = arguments.get("limit").and_then(Value::as_i64).unwrap_or(10);
            let hits = super::vector_search_service::semantic_search_records(
                conn,
                workspace_id,
                master_key,
                query,
                object_key,
                limit,
            )
            .await?;
            serde_json::to_value(hits).map_err(ser_err)
        }
        other => Err(AppError::Validation(format!("Unknown tool '{other}'"))),
    }
}

fn ser_err(e: serde_json::Error) -> AppError {
    AppError::Validation(format!("could not serialize result: {e}"))
}

/// Static explainer of how Lanesra OS's own admin/data-model primitives
/// compose - the context gap this closes: nothing else automatically
/// teaches an agent (or the admin chat assistant) how the platform is
/// built, only the literal name/description/argument-shape of each tool.
/// Deliberately not baked into every agent's system prompt by default
/// (that's still whatever the admin who created it wrote - see
/// `agent_system_prompt`'s own doc comment); an agent calls this the same
/// way it calls any other tool, when it decides orientation would help.
const PLATFORM_EXPLAINER: &str = "Lanesra OS is a no-code business platform: every capability - the built-in CRM included - is assembled from a small set of primitives that compose, not hard-coded per business type. The recommended order to add a new one: \
(1) Custom Object - define the record type itself (label, icon, ID prefix/digit width). \
(2) Custom Fields - add fields to that Custom Object, or to most built-in objects (Company, Contact, ...). \
(3) Relationships - connect two object types (a link, not a field on either record - see get_object_metadata's own 'relationships' list for an object's schema, and get_related_records to follow one for a specific record). \
(4) Business Rules - require/hide/lock/default a field, or block the save outright, based on conditions. \
(5) Workflow Automation - fire an action (create a task, update a related record, send a notification, run an AI Agent, ...) when a record is created, a field changes, or a date arrives. \
(6) App - bundle a set of objects (plus their published Screen layouts and a Dashboard) into a named, role-gated app with its own navigation. \
(7) Dashboard - KPI tiles, charts and record lists for that App or the workspace default. \
None of these steps is required in isolation - most workspaces use only Custom Objects and Fields, with no App/Dashboard/Automation at all; build only what the request actually needs.";

fn platform_overview(conn: &Connection, workspace_id: &str) -> AppResult<Value> {
    let objects = api_object_service::list_object_keys(conn, workspace_id)?;
    let custom_object_count = objects.iter().filter(|o| o.is_custom).count();
    let relationship_count = super::relationship_service::list(conn, workspace_id, true)?.len();
    let app_count = super::app_service::list(conn, workspace_id)?.len();
    Ok(json!({
        "how_lanesra_os_is_built": PLATFORM_EXPLAINER,
        "this_workspace": {
            "object_count": objects.len(),
            "custom_object_count": custom_object_count,
            "relationship_count": relationship_count,
            "app_count": app_count,
            "objects": objects.iter().map(|o| &o.object_key).collect::<Vec<_>>(),
        },
    }))
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
/// Integration Hub Tool Bridge: a connector-derived tool name is
/// self-describing by prefix (`connector_tool_service`'s own doc comment)
/// - "connector_read"/"connector_write" classify it without a DB round
///   trip, same as the two fixed catalogs below. Whether the name actually
///   resolves to a real, currently-enabled connector action is checked
///   where it matters (`ai_agent_service::validate_action_names` at save
///   time, `connector_tool_service::dispatch` at call time), not here.
///   Agent Access Governance (issue #245): the Foundry-only tool names
///   `execute_agent_tool` dispatches directly, never through
///   `dispatch_record_tool`/`dispatch_admin_tool` - `tool_source` used to
///   return `None` for every one of these (see `tool_registry_service::
/// default_risk_for`'s old doc comment), the exact reason the Tool-Call
///   Firewall never saw them. Classified as their own source so a workspace
///   can still block/risk-gate `delegate_to_agent`, say, without this
///   function pretending they're record/admin tools.
const AGENT_INTERNAL_TOOLS: &[&str] = &[
    "update_memory",
    "remember",
    "get_memory",
    "search_knowledge",
    "use_skill",
    "delegate_to_agent",
    "invoke_action",
];

pub(crate) fn tool_source(name: &str) -> Option<&'static str> {
    if record_tools().iter().any(|t| t.name == name) {
        Some("record")
    } else if admin_tools().iter().any(|t| t.name == name) {
        Some("admin")
    } else if name.starts_with("connector_action:") {
        Some("connector_read")
    } else if name.starts_with("connector_write_action:") {
        Some("connector_write")
    } else if name.starts_with("mcp_tool:") {
        Some("mcp_read")
    } else if name.starts_with("mcp_write_tool:") {
        Some("mcp_write")
    } else if AGENT_INTERNAL_TOOLS.contains(&name) {
        Some("agent_internal")
    } else {
        None
    }
}

/// An Agent needs Administrator the moment any one of its own
/// `action_names` resolves to an admin-catalog tool, or a write-capable
/// Connector Action tool - checked once before its loop starts, the same
/// `mode == "admin"` gate `send_message` already has, just keyed off a
/// computed set instead of a literal mode string.
pub(crate) fn agent_requires_admin(action_names: &[String]) -> bool {
    action_names.iter().any(|n| {
        matches!(
            tool_source(n),
            Some("admin" | "connector_write" | "mcp_write")
        )
    })
}

fn to_val<T: serde::Serialize>(r: AppResult<T>) -> AppResult<Value> {
    r.and_then(|v| serde_json::to_value(v).map_err(ser_err))
}

#[allow(clippy::too_many_lines)]
async fn dispatch_admin_tool(
    conn: &Connection,
    workspace_id: &str,
    actor: Option<&str>,
    master_key: &[u8; 32],
    name: &str,
    arguments: &Value,
) -> AppResult<Value> {
    use crate::models::business_rule::BusinessRuleInput;
    use crate::models::custom_field::CustomFieldDefinitionInput;
    use crate::models::custom_object::CustomObjectDefinitionInput;
    use crate::models::custom_report::CustomReportInput;
    use crate::models::integration::{
        ApiClientInput, ConnectionInput, IntegrationJobInput, WebhookInput,
    };
    use crate::models::numbering_override::NumberingOverrideInput;
    use crate::models::relationship::RelationshipDefinitionInput;
    use crate::models::saved_view::SavedViewInput;
    use crate::models::status_transition::StatusTransitionInput;
    use crate::models::user::NewUser;
    use crate::models::workflow::WorkflowDefinitionInput;
    use crate::services::{
        api_client_service, app_service, business_rule_service, connection_service,
        custom_object_service, custom_report_service, dashboard_layout_service,
        integration_job_service, numbering_service, relationship_service, saved_view_service,
        status_transition_service, user_service, webhook_service, workflow_service,
    };

    fn from_args<T: serde::de::DeserializeOwned>(arguments: &Value) -> AppResult<T> {
        serde_json::from_value(arguments.clone())
            .map_err(|e| AppError::Validation(format!("Invalid arguments for this tool: {e}")))
    }

    match name {
        "list_business_rules" => {
            let entity_type = required_str(arguments, "entity_type")?;
            to_val(business_rule_service::list_rules(
                conn,
                workspace_id,
                entity_type,
                true,
            ))
        }
        "create_business_rule" => {
            let input: BusinessRuleInput = from_args(arguments)?;
            to_val(business_rule_service::create_rule(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_workflows" => {
            let entity_type = required_str(arguments, "entity_type")?;
            to_val(workflow_service::list_rules(
                conn,
                workspace_id,
                entity_type,
                actor,
            ))
        }
        "create_workflow" => {
            let input: WorkflowDefinitionInput = from_args(arguments)?;
            to_val(workflow_service::create_rule(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_custom_objects" => to_val(custom_object_service::list(conn, workspace_id, true)),
        "create_custom_object" => {
            let input: CustomObjectDefinitionInput = from_args(arguments)?;
            to_val(custom_object_service::create(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_custom_fields" => {
            let entity_type = required_str(arguments, "entity_type")?;
            to_val(super::custom_field_service::list_definitions(
                conn,
                workspace_id,
                entity_type,
                true,
            ))
        }
        "create_custom_field" => {
            let input: CustomFieldDefinitionInput = from_args(arguments)?;
            to_val(super::custom_field_service::create_definition(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_relationships" => to_val(relationship_service::list(conn, workspace_id, true)),
        "create_relationship" => {
            let input: RelationshipDefinitionInput = from_args(arguments)?;
            to_val(relationship_service::create(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_status_transitions" => {
            let entity_type = required_str(arguments, "entity_type")?;
            to_val(status_transition_service::list(
                conn,
                workspace_id,
                entity_type,
                actor,
            ))
        }
        "create_status_transition" => {
            let input: StatusTransitionInput = from_args(arguments)?;
            to_val(status_transition_service::create(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_connections" => to_val(connection_service::list_for_workspace(conn, workspace_id)),
        "create_connection" => {
            // The one non-negotiable boundary this module's own doc
            // comment names: secret_value is never read from a tool
            // call's arguments, even if present - always None here,
            // regardless of what the model or user supplied.
            let mut input: ConnectionInput = from_args(arguments)?;
            input.secret_value = None;
            to_val(connection_service::create(
                conn,
                workspace_id,
                master_key,
                &input,
                actor,
            ))
        }
        "list_webhooks" => to_val(webhook_service::list_for_workspace(conn, workspace_id)),
        "create_webhook" => {
            let input: WebhookInput = from_args(arguments)?;
            to_val(webhook_service::create(
                conn,
                workspace_id,
                master_key,
                &input,
                actor,
            ))
        }
        "list_integration_jobs" => to_val(integration_job_service::list_for_workspace(
            conn,
            workspace_id,
        )),
        "create_integration_job" => {
            let input: IntegrationJobInput = from_args(arguments)?;
            to_val(integration_job_service::create(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_api_clients" => to_val(api_client_service::list_for_workspace(conn, workspace_id)),
        "create_api_client" => {
            let input: ApiClientInput = from_args(arguments)?;
            let issued = api_client_service::create(conn, workspace_id, &input, actor)?;
            // "Shown once" convention - never into chat history at all.
            Ok(
                json!({"client": issued.client, "note": "Credential generated - retrieve it once from Integration Hub -> API Access, not shown in chat."}),
            )
        }
        "list_dashboards" => to_val(dashboard_layout_service::list_layouts(conn, workspace_id)),
        "create_dashboard" => {
            let input: crate::models::dashboard_layout::DashboardLayoutInput =
                from_args(arguments)?;
            to_val(dashboard_layout_service::create_layout(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_apps" => to_val(app_service::list(conn, workspace_id)),
        "create_app" => {
            let input: crate::models::app_definition::AppDefinitionInput = from_args(arguments)?;
            to_val(app_service::create(conn, workspace_id, &input, actor))
        }
        "list_custom_reports" => to_val(custom_report_service::list(conn, workspace_id)),
        "create_custom_report" => {
            let input: CustomReportInput = from_args(arguments)?;
            to_val(custom_report_service::create(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_saved_views" => {
            let object_key = required_str(arguments, "object_key")?;
            to_val(saved_view_service::list_for_object(
                conn,
                workspace_id,
                object_key,
                actor,
            ))
        }
        "create_saved_view" => {
            let input: SavedViewInput = from_args(arguments)?;
            to_val(saved_view_service::create(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_users" => to_val(user_service::list(conn, workspace_id)),
        "create_user" => {
            let input: NewUser = from_args(arguments)?;
            to_val(user_service::create(conn, workspace_id, &input, actor))
        }
        "list_numbering" => to_val(numbering_service::list_effective(conn, workspace_id, actor)),
        "set_numbering" => {
            let input: NumberingOverrideInput = from_args(arguments)?;
            to_val(numbering_service::set_override(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "get_workspace_profile" => to_val(Ok(crate::repositories::workspace_repo::get_current(
            conn,
        )
        .map_err(AppError::from)?)),
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
            if let Some(v) = arguments.get("business_name").and_then(Value::as_str) {
                update.business_name = v.to_string();
            }
            if let Some(v) = arguments.get("legal_name").and_then(Value::as_str) {
                update.legal_name = Some(v.to_string());
            }
            if let Some(v) = arguments.get("business_address").and_then(Value::as_str) {
                update.business_address = Some(v.to_string());
            }
            if let Some(v) = arguments.get("phone").and_then(Value::as_str) {
                update.phone = Some(v.to_string());
            }
            if let Some(v) = arguments.get("currency_code").and_then(Value::as_str) {
                update.currency_code = v.to_string();
            }
            if let Some(v) = arguments.get("locale").and_then(Value::as_str) {
                update.locale = v.to_string();
            }
            if let Some(v) = arguments.get("timezone").and_then(Value::as_str) {
                update.timezone = v.to_string();
            }
            if let Some(v) = arguments.get("default_tax_rate_bp").and_then(Value::as_i64) {
                update.default_tax_rate_bp = v;
            }
            to_val(super::workspace_service::update(conn, &update, actor))
        }
        "list_connectors" => to_val(super::connector_service::list_for_workspace(
            conn,
            workspace_id,
        )),
        "list_ai_agents" => to_val(super::ai_agent_service::list(conn, workspace_id, true)),
        "create_ai_agent" => {
            let input: crate::models::ai_agent::AiAgentInput = from_args(arguments)?;
            to_val(super::ai_agent_service::create(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_ai_skills" => to_val(super::ai_agent_service::list_skills(
            conn,
            workspace_id,
            true,
        )),
        "create_ai_skill" => {
            let input: crate::models::ai_agent::AiSkillInput = from_args(arguments)?;
            to_val(super::ai_agent_service::create_skill(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "list_ai_agent_pipelines" => to_val(super::ai_orchestration_service::list_pipelines(
            conn,
            workspace_id,
            true,
        )),
        "create_ai_agent_pipeline" => {
            let input: crate::models::ai_agent_pipeline::AiAgentPipelineInput =
                from_args(arguments)?;
            to_val(super::ai_orchestration_service::create_pipeline(
                conn,
                workspace_id,
                &input,
                actor,
            ))
        }
        "run_ai_agent" => {
            let agent_id = required_str(arguments, "agent_id")?;
            let input = required_str(arguments, "input")?;
            to_val(
                super::ai_orchestration_service::run_manual(
                    conn,
                    workspace_id,
                    master_key,
                    "agent",
                    agent_id,
                    input,
                    actor,
                )
                .await,
            )
        }
        "run_ai_agent_pipeline" => {
            let pipeline_id = required_str(arguments, "pipeline_id")?;
            let input = required_str(arguments, "input")?;
            to_val(
                super::ai_orchestration_service::run_manual(
                    conn,
                    workspace_id,
                    master_key,
                    "pipeline",
                    pipeline_id,
                    input,
                    actor,
                )
                .await,
            )
        }
        other => Err(AppError::Validation(format!("Unknown tool '{other}'"))),
    }
}

fn tools_for_mode(mode: &str) -> AppResult<Vec<ToolSpec>> {
    match mode {
        "records" => Ok(record_tools()),
        // The records catalog too - see `execute_tool`'s own doc comment
        // on why an admin-gated assistant should have at least everything
        // a records-mode one does, not a disjoint set.
        "admin" => Ok(record_tools().into_iter().chain(admin_tools()).collect()),
        other => Err(AppError::Validation(format!("Unknown chat mode '{other}'"))),
    }
}

fn system_prompt(mode: &str) -> AppResult<String> {
    let id = match mode {
        "records" => "assistant/records",
        "admin" => "assistant/admin",
        other => return Err(AppError::Validation(format!("Unknown chat mode '{other}'"))),
    };
    crate::prompts::render_active(id, &[])
        .map(|(text, _)| text)
        .map_err(AppError::Validation)
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
    let Some(items) = raw_assistant.as_array() else {
        return raw_assistant.clone();
    };
    Value::Array(items.iter().map(redact_one_tool_call).collect())
}

const REDACTED_SECRET: &str = "<redacted - never stored in chat history>";

fn redact_one_tool_call(item: &Value) -> Value {
    let mut item = item.clone();
    let is_anthropic_create_connection = item.get("type").and_then(Value::as_str)
        == Some("tool_use")
        && item.get("name").and_then(Value::as_str) == Some("create_connection");
    if is_anthropic_create_connection {
        if let Some(input) = item.get_mut("input").and_then(Value::as_object_mut) {
            if input.contains_key("secret_value") {
                input.insert("secret_value".to_string(), json!(REDACTED_SECRET));
            }
        }
        return item;
    }
    let is_openai_create_connection = item
        .get("function")
        .and_then(|f| f.get("name"))
        .and_then(Value::as_str)
        == Some("create_connection");
    if is_openai_create_connection {
        if let Some(function) = item.get_mut("function").and_then(Value::as_object_mut) {
            if let Some(mut parsed) = function
                .get("arguments")
                .and_then(Value::as_str)
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
            {
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
/// turn answers in plain text - capped at the round limit, then one final answer (WP-0121). Returns every
/// message this call appended (the user's own, plus everything the
/// assistant/tools produced), so the caller can render just the new
/// turns without re-fetching the whole conversation.
///
/// `mode == "admin"` requires Administrator, checked once here before
/// the loop starts - not relied on only implicitly inside each tool
/// (even though every admin service function already gates itself too).
pub async fn send_message(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    user_id: &str,
    mode: &str,
    text: &str,
) -> AppResult<Vec<ChatMessage>> {
    if text.trim().is_empty() {
        return Err(AppError::Validation("Say something first".into()));
    }
    if mode == "admin" {
        require_admin(conn, Some(user_id))?;
    }
    let tools = tools_for_mode(mode)?;
    let conversation =
        chat_repo::get_or_create_conversation(conn, workspace_id, user_id, mode, "")?;

    let mut appended = Vec::new();
    appended.push(chat_repo::append_message(
        conn,
        &conversation.id,
        "user",
        Some(text),
        None,
        None,
    )?);

    let mut malformed_calls = 0u32;
    let limits = AgentRunLimits::default();
    let sys_prompt = system_prompt(mode)?;
    for _round in 0..limits.max_rounds {
        let history = chat_repo::list_messages(conn, &conversation.id)?;
        let (outcome, usage) = ai_service::complete_with_tools(
            conn,
            workspace_id,
            master_key,
            &sys_prompt,
            &tools,
            &history,
            CallOptions::default(),
        )
        .await?;
        // Phase 7a: recorded for Gateway health-view visibility even on
        // this plain (no-agent) path - agent_id "" is the same "not an
        // agent run" sentinel `ai_token_usage_repo`'s own doc comment
        // describes, matching `chat_conversations.agent_id`'s convention.
        let _ = ai_token_usage_repo::increment(
            conn,
            workspace_id,
            "",
            user_id,
            usage.input_tokens,
            usage.output_tokens,
        );
        match outcome {
            CompletionOutcome::Text(text) => {
                appended.push(chat_repo::append_message(
                    conn,
                    &conversation.id,
                    "assistant",
                    Some(&text),
                    None,
                    None,
                )?);
                return Ok(appended);
            }
            CompletionOutcome::ToolCalls {
                raw_assistant,
                calls,
            } => {
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
                appended.push(chat_repo::append_message(
                    conn,
                    &conversation.id,
                    "assistant",
                    None,
                    Some(&persisted_assistant),
                    None,
                )?);
                for call in calls {
                    if let Err(problem) = ai_service::check_tool_call(&call, &tools) {
                        malformed_calls += 1;
                        appended.push(chat_repo::append_message(
                            conn,
                            &conversation.id,
                            "tool",
                            Some(&malformed_tool_result(&problem)),
                            None,
                            Some(&call.id),
                        )?);
                        if malformed_calls >= MAX_MALFORMED_TOOL_CALLS {
                            return Err(too_many_malformed_calls());
                        }
                        continue;
                    }
                    let result =
                        execute_tool(conn, workspace_id, master_key, Some(user_id), mode, &call)
                            .await;
                    let content = match result {
                        Ok(value) => value.to_string(),
                        Err(e) => format!("Error: {e}"),
                    };
                    appended.push(chat_repo::append_message(
                        conn,
                        &conversation.id,
                        "tool",
                        Some(&content),
                        None,
                        Some(&call.id),
                    )?);
                }
            }
        }
    }
    // WP-0121: the round limit was reached. Ask once for a final answer
    // with tools off and keep it, rather than discarding the work so far.
    let history = chat_repo::list_messages(conn, &conversation.id)?;
    let (outcome, usage) = ai_service::complete_with_tools(
        conn,
        workspace_id,
        master_key,
        &final_answer_prompt(&sys_prompt, STOP_LIMIT_ROUNDS, &limits)?,
        &tools,
        &history,
        CallOptions {
            tools_off: true,
            ..CallOptions::default()
        },
    )
    .await?;
    let _ = ai_token_usage_repo::increment(
        conn,
        workspace_id,
        "",
        user_id,
        usage.input_tokens,
        usage.output_tokens,
    );
    appended.push(chat_repo::append_message(
        conn,
        &conversation.id,
        "assistant",
        Some(&final_answer_text(outcome)),
        None,
        None,
    )?);
    Ok(appended)
}

/// The Tool-Call Firewall: checked by both `execute_tool` and
/// `execute_agent_tool` immediately before dispatching, never inside a
/// dispatcher itself - `policy_engine_service::evaluate` is where the
/// actual decision is computed, this is only where it's enforced.
/// `Ok(())` means dispatch as normal; `Err` carries the exact message the
/// model/user sees in place of the tool's real result. `agent_id` is
/// `None` for the fixed "records"/"admin" chat assistants, which have no
/// agent identity of their own - only a workspace-default policy can
/// govern those.
fn apply_tool_firewall(
    conn: &Connection,
    workspace_id: &str,
    agent_id: Option<&str>,
    actor: Option<&str>,
    name: &str,
    arguments: &Value,
) -> AppResult<()> {
    let source = tool_source(name);
    let decision = policy_engine_service::evaluate(conn, workspace_id, agent_id, name, source)?;
    // WP-0140 AC-2: a call the policy allows is checked against the tool,
    // connector and workspace rate limits before it can run.
    let rate = match &decision {
        PolicyDecision::Allow => {
            super::tool_limits_service::check_and_count_call(conn, workspace_id, name)
        }
        _ => Ok(()),
    };
    // WP-0111: the firewall's verdict is its own span under the tool call.
    let (verdict, risk) = match &decision {
        PolicyDecision::Allow if rate.is_err() => ("rate_limited", None),
        PolicyDecision::Allow => ("allow", None),
        PolicyDecision::Deny { risk_level } => ("deny", Some(risk_level.as_str())),
        PolicyDecision::RequireApproval { risk_level } => {
            ("require_approval", Some(risk_level.as_str()))
        }
    };
    trace_service::start(
        conn,
        "policy",
        "tool_call_firewall",
        json!({"lanesra.policy.verdict": verdict, "lanesra.policy.risk": risk, "lanesra.tool.name": name}),
    )
    .end(
        conn,
        if verdict == "allow" {
            SpanStatus::Ok
        } else {
            SpanStatus::Error
        },
        match &rate {
            Err(e) => Value::Object(trace_service::error_attributes(e)),
            Ok(()) => json!({}),
        },
    );
    rate?;
    match decision {
        PolicyDecision::Allow => Ok(()),
        PolicyDecision::Deny { risk_level } => Err(AppError::PolicyBlocked(format!(
            "'{name}' is blocked by this workspace's agent policy (risk level: {}).",
            risk_level.as_str()
        ))),
        PolicyDecision::RequireApproval { risk_level } => {
            policy_engine_service::record_pending_tool_call(
                conn,
                workspace_id,
                agent_id,
                name,
                arguments,
                risk_level,
                actor,
            )?;
            Err(AppError::PolicyBlocked(format!(
                "'{name}' requires administrator approval under this workspace's agent policy (risk level: {}) - a pending approval has been recorded for an administrator to review.",
                risk_level.as_str()
            )))
        }
    }
}

async fn execute_tool(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor: Option<&str>,
    mode: &str,
    call: &RequestedToolCall,
) -> AppResult<Value> {
    if mode == "records" || mode == "admin" {
        apply_tool_firewall(conn, workspace_id, None, actor, &call.name, &call.arguments)?;
    }
    // Agent Access Governance (issue #245): the fixed "records"/"admin"
    // assistants have no agent identity, so only the workspace-default
    // policy can opt them into real Access Control v1 enforcement - see
    // `effective_write_actor`'s own doc comment.
    let access_actor = effective_write_actor(conn, workspace_id, None, actor, None)?;
    match mode {
        "records" => {
            dispatch_record_tool(
                conn,
                workspace_id,
                master_key,
                &call.name,
                &call.arguments,
                access_actor.as_deref(),
            )
            .await
        }
        // Admin mode's own tool list is the records catalog plus the
        // admin one (see `tools_for_mode`) - an admin who can create a
        // Business Rule can obviously also read what get_object_metadata/
        // get_related_records/get_platform_overview already expose to
        // every other agent, so route by which catalog a name actually
        // belongs to rather than assuming the whole mode maps to one
        // dispatcher.
        "admin" => match tool_source(&call.name) {
            Some("record") => {
                dispatch_record_tool(
                    conn,
                    workspace_id,
                    master_key,
                    &call.name,
                    &call.arguments,
                    access_actor.as_deref(),
                )
                .await
            }
            _ => {
                dispatch_admin_tool(
                    conn,
                    workspace_id,
                    actor,
                    master_key,
                    &call.name,
                    &call.arguments,
                )
                .await
            }
        },
        other => Err(AppError::Validation(format!("Unknown chat mode '{other}'"))),
    }
}

/// Agent Access Governance (issue #245): the actor
/// `dispatch_record_tool`'s record-write branches check Access Control
/// v1 against. `None` (today's "unattributed/system" convention,
/// unchanged) unless the policy that actually governs this call (agent-
/// specific, else the workspace default, else none configured at all)
/// has `enforce_record_access` on - a workspace that's never opened this
/// screen sees exactly today's behavior either way. Once on: `acts_as`
/// (an agent's own bound identity) wins if set, else whichever real
/// human is chatting right now (`actor`); a scheduled/background run
/// with neither stays unattributed even with the toggle on, since
/// there's no identity to check against.
fn effective_write_actor(
    conn: &Connection,
    workspace_id: &str,
    agent_id: Option<&str>,
    actor: Option<&str>,
    acts_as: Option<&str>,
) -> AppResult<Option<String>> {
    let enforce = policy_engine_service::resolve_policy(conn, workspace_id, agent_id)?
        .map(|p| p.enforce_record_access)
        .unwrap_or(false);
    if !enforce {
        return Ok(None);
    }
    Ok(acts_as
        .map(String::from)
        .or_else(|| actor.map(String::from)))
}

pub fn get_history(
    conn: &Connection,
    workspace_id: &str,
    user_id: &str,
    mode: &str,
) -> AppResult<Vec<ChatMessage>> {
    let conversation =
        chat_repo::get_or_create_conversation(conn, workspace_id, user_id, mode, "")?;
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
/// Also the depth bound `ai_agent_hierarchy_service` resolves a Pipeline's
/// static delegation tree to - the runtime guard and the hierarchy view
/// must agree on the same number, so it's exposed rather than duplicated.
pub(crate) const MAX_DELEGATION_DEPTH: u8 = 4;

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
fn synthetic_message(
    role: &str,
    content: Option<&str>,
    tool_calls: Option<&Value>,
    tool_call_id: Option<&str>,
) -> ChatMessage {
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
/// The names of every tool `agent` is offered (WP-0140 caps bind to these).
pub(crate) fn agent_tool_names(
    conn: &Connection,
    agent: &AiAgentDefinition,
) -> AppResult<Vec<String>> {
    Ok(agent_tools(conn, agent)?
        .into_iter()
        .map(|t| t.name)
        .collect())
}

fn agent_tools(conn: &Connection, agent: &AiAgentDefinition) -> AppResult<Vec<ToolSpec>> {
    let all_record = record_tools();
    let all_admin = admin_tools();
    // Workspace-scoped, unlike the two fixed catalogs above, so it can't
    // be a bare `fn() -> Vec<ToolSpec>` - built fresh per call, same as
    // they are.
    let all_connector = super::connector_tool_service::agent_tools(conn, &agent.workspace_id)?;
    let all_mcp = super::mcp_client_service::agent_tools(conn, &agent.workspace_id)?;
    let mut tools: Vec<ToolSpec> = agent
        .action_names
        .iter()
        .filter_map(|name| {
            all_record
                .iter()
                .chain(all_admin.iter())
                .chain(all_connector.iter())
                .chain(all_mcp.iter())
                .find(|t| &t.name == name)
                .cloned()
        })
        .collect();

    tools.push(tool(
        "update_memory",
        "Overwrite your own persistent memory with the complete updated document (not a diff - write the whole thing each time, including what you're keeping from before). Arguments: content.",
        object_schema(),
    ));

    // AI Agent Platform v2, Phase 4: the three itemized memory types
    // (Session/Working/Entity) on top of update_memory's own workspace-wide
    // Agent Memory above - see models::ai_memory's own doc comment for
    // exactly what each type is scoped by. Always offered, like
    // update_memory - `remember`/`get_memory` simply error with a clear
    // message if the calling context can't scope the requested type (e.g.
    // `working` memory outside a real Pipeline/Graph run).
    tools.push(tool(
        "remember",
        "Save a durable fact as one of three scoped memory types (distinct from update_memory's single persistent document above). Arguments: memory_type ('session' = this conversation with this user, 'working' = this specific automated run only, 'entity' = a durable fact about one CRM record, tied to it forever), content, entity_type + entity_id (required only for memory_type='entity', e.g. entity_type='Company'), confidence (0.0-1.0, optional), classification ('standard'|'sensitive'|'restricted', default 'standard' - 'restricted' content may be excluded by this workspace's policy).",
        object_schema(),
    ));
    tools.push(tool(
        "get_memory",
        "Recall previously remembered items of one scoped memory type. Arguments: memory_type ('session'|'working'|'entity'), entity_type + entity_id (required only for memory_type='entity').",
        object_schema(),
    ));
    tools.push(tool(
        "search_knowledge",
        "Search this workspace's curated Knowledge Sources (documents an administrator has added) for passages relevant to a question - always cite the source name in your answer when you use a result. Arguments: query, collection_id (optional, narrows to one named collection).",
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

    // Ontology Layer epic, issue #344: this agent's own governed Action
    // Types (issue #343) - only offered when it actually has at least one,
    // same "no point offering a tool with nothing behind it" reasoning
    // `use_skill`/`delegate_to_agent` above already follow. Each name is
    // listed with its description so the model can decide which one (if
    // any) fits, the same way `use_skill`'s own prompt lists skill names.
    if !agent.action_type_ids.is_empty() {
        let mut descriptions = Vec::new();
        for action_type_id in &agent.action_type_ids {
            if let Some(def) =
                action_type_repo::get(conn, action_type_id).map_err(AppError::from)?
            {
                if def.is_active {
                    let params = if def.parameters.is_empty() {
                        "no parameters".to_string()
                    } else {
                        def.parameters
                            .iter()
                            .map(|p| {
                                format!(
                                    "{} ({}{})",
                                    p.key,
                                    p.field_type,
                                    if p.required { ", required" } else { "" }
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    descriptions.push(format!(
                        "'{}' on {} - {} [params: {params}]",
                        def.name, def.target_object_type, def.description
                    ));
                }
            }
        }
        if !descriptions.is_empty() {
            tools.push(tool(
                "invoke_action",
                &format!(
                    "Invoke one of your granted Action Types against a specific record - always goes through that object's real write path, with its own required capability and any Business Rules enforced exactly as a human's edit would be. Arguments: action_type_name (one of the names listed below), record_id, params (an object of parameter key -> string value, per the Action Type's own parameters). Your granted Action Types: {}",
                    descriptions.join("; ")
                ),
                object_schema(),
            ));
        }
    }

    Ok(tools)
}

/// This agent's persona, plus its persistent Memory (if it's written
/// anything yet) and a short name+description catalog of its attached
/// Skills - the same "short description up front, full content only on
/// demand via use_skill" shape this session's own Skill tool uses.
///
/// AI Agent Platform v2, Phase 4: Session Memory is injected ambiently
/// here (a short, bounded list) since it's this agent's own ongoing
/// relationship with this specific user - ordinary conversational context,
/// the same way `memory_md` already is. Entity and Working Memory are
/// deliberately NOT dumped in here - they're queried on demand via
/// `get_memory` - the "never dump an entire table into a prompt"
/// structured-retrieval discipline this issue's own scope text asks for;
/// Render a catalog section template with a single variable (WP-0130, ADR-0011).
fn render_catalog_section(id: &str, var_name: &str, var_value: &str) -> AppResult<String> {
    crate::prompts::render_active(id, &[(var_name, var_value)])
        .map(|(text, _)| text)
        .map_err(AppError::Validation)
}

/// unlike Session Memory, they aren't naturally singular/small per turn.
fn agent_system_prompt(
    conn: &Connection,
    agent: &AiAgentDefinition,
    memory_context: &AgentMemoryContext,
) -> AppResult<String> {
    let mut prompt = agent.system_prompt.clone();
    if !agent.memory_md.trim().is_empty() {
        let section = render_catalog_section("loop/memory-section", "memory_md", &agent.memory_md)?;
        prompt.push_str("\n\n");
        prompt.push_str(&section);
    }
    if let Ok(items) =
        ai_memory_service::list_context(conn, &agent.id, "session", None, None, memory_context)
    {
        if !items.is_empty() {
            let items_text: String = items
                .iter()
                .map(|item| format!("- {}\n", item.content))
                .collect();
            let section =
                render_catalog_section("loop/session-memory-section", "items", &items_text)?;
            prompt.push_str("\n\n");
            prompt.push_str(&section);
        }
    }
    if !agent.guardrails_md.trim().is_empty() {
        let section = render_catalog_section(
            "loop/guardrails-section",
            "guardrails_md",
            &agent.guardrails_md,
        )?;
        prompt.push_str("\n\n");
        prompt.push_str(&section);
    }
    if !agent.skill_ids.is_empty() {
        let mut skills_text = String::new();
        for skill_id in &agent.skill_ids {
            if let Some(skill) = ai_agent_repo::get_skill(conn, skill_id).map_err(AppError::from)? {
                skills_text.push_str(&format!("- {}: {}\n", skill.name, skill.description));
            }
        }
        let section = render_catalog_section("loop/skills-section", "skills", &skills_text)?;
        prompt.push_str("\n\n");
        prompt.push_str(&section);
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
    memory_context: &'a AgentMemoryContext,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Value>> + 'a>> {
    Box::pin(async move {
        // WP-0143 (RAID I-16): an agent may call only the tools it was
        // offered (`agent_tools`). Checked before the policy firewall, so an
        // unlisted call never even queues an approval.
        if !agent_tools(conn, agent)?
            .iter()
            .any(|t| t.name == call.name)
        {
            return Err(AppError::classified(
                ErrorClass::Validation,
                "validation.unknown_tool",
                format!("'{}' isn't one of this agent's tools", call.name),
            ));
        }
        // Agent Access Governance (issue #245): checked once, here, for
        // every tool name this agent can call - including the Foundry-
        // internal ones matched directly below, which used to dispatch
        // before the firewall ever saw them. The inner `other` branch no
        // longer repeats this call.
        apply_tool_firewall(
            conn,
            workspace_id,
            Some(&agent.id),
            actor,
            &call.name,
            &call.arguments,
        )?;
        match call.name.as_str() {
            "update_memory" => {
                let content = required_str(&call.arguments, "content")?;
                ai_agent_repo::update_memory(conn, &agent.id, content, "agent")
                    .map_err(AppError::from)?;
                Ok(json!({"memory_updated": true}))
            }
            "remember" => {
                let input: MemoryItemInput = serde_json::from_value(call.arguments.clone())
                    .map_err(|e| AppError::Validation(format!("invalid arguments: {e}")))?;
                let item = ai_memory_service::remember(
                    conn,
                    workspace_id,
                    &agent.id,
                    &input,
                    memory_context,
                    "agent",
                )?;
                Ok(json!({"remembered": true, "id": item.id}))
            }
            "get_memory" => {
                let memory_type = required_str(&call.arguments, "memory_type")?;
                let entity_type = call.arguments.get("entity_type").and_then(|v| v.as_str());
                let entity_id = call.arguments.get("entity_id").and_then(|v| v.as_str());
                let items = ai_memory_service::list_context(
                    conn,
                    &agent.id,
                    memory_type,
                    entity_type,
                    entity_id,
                    memory_context,
                )?;
                Ok(
                    json!({"items": items.iter().map(|i| json!({"content": i.content, "source": i.source, "confidence": i.confidence, "created_at": i.created_at})).collect::<Vec<_>>()}),
                )
            }
            "search_knowledge" => {
                let query = required_str(&call.arguments, "query")?;
                let collection_id = call.arguments.get("collection_id").and_then(|v| v.as_str());
                let hits = ai_knowledge_service::search_knowledge(
                    conn,
                    workspace_id,
                    master_key,
                    query,
                    collection_id,
                    5,
                )
                .await?;
                Ok(
                    json!({"results": hits.iter().map(|h| json!({"source_name": h.source_name, "content": h.content, "similarity": h.similarity})).collect::<Vec<_>>()}),
                )
            }
            "use_skill" => {
                let skill_name = required_str(&call.arguments, "name")?;
                for skill_id in &agent.skill_ids {
                    if let Some(skill) =
                        ai_agent_repo::get_skill(conn, skill_id).map_err(AppError::from)?
                    {
                        if skill.name == skill_name {
                            return Ok(json!({"instructions": skill.instructions_md}));
                        }
                    }
                }
                Err(AppError::Validation(format!(
                    "'{skill_name}' isn't one of this agent's attached skills"
                )))
            }
            "delegate_to_agent" => {
                let agent_name = required_str(&call.arguments, "agent_name")?;
                let input = required_str(&call.arguments, "input")?;
                let mut target = None;
                for delegate_id in &agent.delegate_agent_ids {
                    if let Some(candidate) =
                        ai_agent_repo::get(conn, delegate_id).map_err(AppError::from)?
                    {
                        if candidate.name == agent_name && candidate.is_active {
                            target = Some(candidate);
                            break;
                        }
                    }
                }
                let target = target.ok_or_else(|| {
                    AppError::Validation(format!(
                        "'{agent_name}' isn't one of this agent's delegate sub-agents"
                    ))
                })?;
                let seed = vec![synthetic_message("user", Some(input), None, None)];
                // WP-0111: a delegation is a child step of the tool call.
                let step_span = trace_service::start(
                    conn,
                    "step",
                    &format!("delegate:{}", target.name),
                    json!({
                        "lanesra.step.kind": "delegation",
                        "lanesra.agent.id": target.id,
                        "lanesra.agent.name": target.name,
                        "lanesra.agent.version_id": target.current_version_id,
                        "lanesra.delegated_by": agent.id,
                    }),
                );
                // Delegation preserves the same memory context - a
                // sub-agent's `remember`/`get_memory` calls scope to the
                // same session/run its supervisor is already running in.
                let outcome = run_agent_once(
                    conn,
                    workspace_id,
                    master_key,
                    actor,
                    &target,
                    seed,
                    memory_context,
                )
                .await;
                step_span.end_result(
                    conn,
                    &outcome,
                    json!({"lanesra.stop_reason": outcome.as_ref().ok().and_then(|o| o.stop_reason)}),
                );
                let outcome = outcome?;
                Ok(json!({"answer": outcome.final_text}))
            }
            "invoke_action" => {
                let action_type_name = required_str(&call.arguments, "action_type_name")?;
                let mut matched_id = None;
                for candidate_id in &agent.action_type_ids {
                    if let Some(def) =
                        action_type_repo::get(conn, candidate_id).map_err(AppError::from)?
                    {
                        if def.is_active && def.name == action_type_name {
                            matched_id = Some(def.id);
                            break;
                        }
                    }
                }
                let action_type_id = matched_id.ok_or_else(|| {
                    AppError::Validation(format!(
                        "'{action_type_name}' isn't one of this agent's granted Action Types"
                    ))
                })?;
                let record_id = required_str(&call.arguments, "record_id")?;
                let params: std::collections::HashMap<String, String> = call
                    .arguments
                    .get("params")
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    })
                    .unwrap_or_default();
                // Action Types are a governed primitive by design (issue
                // #343's own `required_capability` check) - always
                // attributed to a real actor when one exists, independent
                // of whether `enforce_record_access` has opted generic
                // record writes into attribution, unlike the "record"
                // branch below. An Assign-gated Action Type must still
                // deny an agent/actor without Assign even when this
                // agent's own Ontology scope only grants `update`.
                let invoking_actor = agent.acts_as_user_id.as_deref().or(actor);
                action_type_service::invoke(
                    conn,
                    workspace_id,
                    &action_type_id,
                    record_id,
                    &params,
                    invoking_actor,
                )
            }
            other => match tool_source(other) {
                Some(
                    src @ ("record" | "admin" | "connector_read" | "connector_write" | "mcp_read"
                    | "mcp_write"),
                ) => match src {
                    "record" => {
                        check_object_type_scope(agent, other, &call.arguments)?;
                        let access_actor = effective_write_actor(
                            conn,
                            workspace_id,
                            Some(&agent.id),
                            actor,
                            agent.acts_as_user_id.as_deref(),
                        )?;
                        dispatch_record_tool(
                            conn,
                            workspace_id,
                            master_key,
                            other,
                            &call.arguments,
                            access_actor.as_deref(),
                        )
                        .await
                    }
                    "admin" => {
                        dispatch_admin_tool(
                            conn,
                            workspace_id,
                            actor,
                            master_key,
                            other,
                            &call.arguments,
                        )
                        .await
                    }
                    "connector_read" | "connector_write" => {
                        super::connector_tool_service::dispatch(
                            conn,
                            workspace_id,
                            master_key,
                            actor,
                            other,
                            &call.arguments,
                        )
                        .await
                    }
                    _ => {
                        super::mcp_client_service::dispatch(
                            conn,
                            workspace_id,
                            master_key,
                            actor,
                            other,
                            &call.arguments,
                        )
                        .await
                    }
                },
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
/// WP-0111: the text a trace keeps as a message's body.
fn message_text(m: &ChatMessage) -> String {
    match (&m.content, &m.tool_calls) {
        (Some(c), _) => c.clone(),
        (None, Some(calls)) => calls.to_string(),
        (None, None) => String::new(),
    }
}

/// Opens a round's model_call span (WP-0111), keeping the newest message
/// sent as its input body.
fn start_model_call_span(
    conn: &Connection,
    agent: &AiAgentDefinition,
    round: u32,
    tools_off: bool,
    history: &[ChatMessage],
) -> trace_service::Span {
    let mut span = trace_service::start(
        conn,
        "model_call",
        "model_call",
        json!({
            "lanesra.round": round,
            "lanesra.agent.id": agent.id,
            "lanesra.tools_off": tools_off,
        }),
    );
    if span.is_recording() {
        if let Some(last) = history.last() {
            span.body("input", &message_text(last));
        }
    }
    span
}

/// The model call's ADR-0008 attributes: OpenTelemetry GenAI names for the
/// model and token counts, and which tier answered. No cost: prices come
/// with the model catalog (WP-0146).
fn record_model_call(
    span: &mut trace_service::Span,
    outcome: &ai_gateway_service::GatewayOutcome,
    started: std::time::Instant,
) {
    if !span.is_recording() {
        return;
    }
    span.set("gen_ai.system", json!(outcome.provider));
    span.set("gen_ai.request.model", json!(outcome.model));
    span.set(
        "gen_ai.usage.input_tokens",
        json!(outcome.usage.input_tokens),
    );
    span.set(
        "gen_ai.usage.output_tokens",
        json!(outcome.usage.output_tokens),
    );
    span.set("lanesra.served_by", json!(outcome.served_by));
    span.set(
        "lanesra.latency_ms",
        json!(started.elapsed().as_millis() as i64),
    );
    match &outcome.outcome {
        CompletionOutcome::Text(text) => {
            span.set("lanesra.model.reply", json!("text"));
            span.body("output", text);
        }
        CompletionOutcome::ToolCalls { calls, .. } => {
            span.set("lanesra.model.reply", json!("tool_calls"));
            span.set("lanesra.model.tool_calls", json!(calls.len()));
            let names: Vec<&str> = calls.iter().map(|c| c.name.as_str()).collect();
            span.body("output", &json!(names).to_string());
        }
    }
}

/// Closes `span` as failed with `e`'s class and code, and hands `e` back.
fn end_span_with_error(conn: &Connection, span: trace_service::Span, e: AppError) -> AppError {
    span.end(
        conn,
        SpanStatus::Error,
        Value::Object(trace_service::error_attributes(&e)),
    );
    e
}

#[derive(Debug)]
pub struct AgentRunOutcome {
    pub final_text: String,
    pub produced: Vec<ChatMessage>,
    /// AI Agent Platform v2, Phase 6a: how many tool calls in this run were
    /// denied or queued for approval by the Tool-Call Firewall
    /// (`AppError::PolicyBlocked`) - what the eval harness's
    /// `policy_compliance` evaluator type checks, deterministically, no
    /// judge call needed.
    pub policy_violations: i64,
    /// WP-0121: `None` when the model finished on its own; otherwise the
    /// limit it reached (`STOP_LIMIT_ROUNDS` or `STOP_LIMIT_TOKENS`), and
    /// `final_text` is the partial answer it gave with tools switched off.
    pub stop_reason: Option<&'static str>,
}

impl AgentRunOutcome {
    /// Whether the answer is partial (ADR-0007 `completed_partial`).
    pub fn is_partial(&self) -> bool {
        self.stop_reason.is_some()
    }
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
    memory_context: &AgentMemoryContext,
) -> AppResult<AgentRunOutcome> {
    let seed = vec![synthetic_message("user", Some(input_text), None, None)];
    run_agent_once(
        conn,
        workspace_id,
        master_key,
        actor,
        agent,
        seed,
        memory_context,
    )
    .await
}

pub async fn run_agent_once(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor: Option<&str>,
    agent: &AiAgentDefinition,
    seed_history: Vec<ChatMessage>,
    memory_context: &AgentMemoryContext,
) -> AppResult<AgentRunOutcome> {
    let _depth_guard = DelegationDepthGuard::enter().ok_or_else(|| {
        AppError::Validation(
            "Delegation depth limit reached - simplify this agent's delegation chain".into(),
        )
    })?;
    if agent_requires_admin(&agent.action_names) {
        // WP-0109: inside a run, the run's effective identity decides (an
        // unattended run acts as its Agent Identity); outside one, the
        // person chatting must be an Administrator, as before.
        match crate::principals::current_run_identity() {
            Some(identity) => identity.require_admin(conn, workspace_id)?,
            None => require_admin(conn, actor)?,
        }
    }
    let tools = agent_tools(conn, agent)?;
    let system_prompt = agent_system_prompt(conn, agent, memory_context)?;
    // AI Agent Platform v2, Phase 1: this run's Structured Output contract,
    // if the agent's current Published version declares one - `None` is
    // exactly today's free-form-text behavior. Resolved once up front
    // (immutable for the life of this run, same as `tools`/`system_prompt`
    // above), not re-read per round.
    let output_schema: Option<Value> = agent
        .current_version_id
        .as_deref()
        .and_then(|id| ai_agent_version_repo::get(conn, id).ok().flatten())
        .and_then(|v| v.output_schema);

    let mut history = seed_history;
    let mut produced = Vec::new();
    let mut structured_output_repaired = false;
    // Phase 7c: loop-detection guardrail (`guardrails.md`'s own "if the
    // same skill is invoked 3 times with identical parameters, terminate
    // the loop and escalate" example) - tracks only the case of a round
    // requesting exactly one tool call identical in name+arguments to the
    // previous two; a round with zero or multiple calls, or a call that
    // varies at all, resets the streak. Checked *before* executing the
    // would-be 3rd repeat, not after, so a runaway destructive call never
    // actually runs a third time.
    let mut repeat_streak: u8 = 0;
    let mut malformed_calls = 0u32;
    let mut last_call: Option<(String, Value)> = None;
    // AI Agent Platform v2, Phase 6a: counts `AppError::PolicyBlocked`
    // outcomes only, not every tool error - a tool that genuinely fails
    // (a bad argument, a deleted record) isn't a policy violation.
    let mut policy_violations: i64 = 0;
    // WP-0140: calls per tool in this step, for the binding caps.
    let mut calls_this_step: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
    // WP-0121: the agent's limits, and the tokens this run has used so far.
    let limits = AgentRunLimits::for_routing(agent.model_routing.as_ref());
    let mut run_tokens: i64 = 0;
    let mut stop_reason = STOP_LIMIT_ROUNDS;

    for round in 0..limits.max_rounds {
        // WP-0111: one model_call span per round; the round's tool calls
        // nest under it, so it closes after them.
        let mut model_span = start_model_call_span(conn, agent, round + 1, false, &history);
        // Phase 7a: every agent run goes through the Gateway, not
        // straight to `ai_service::complete_with_tools` - it resolves
        // this agent's `model_routing` policy (primary/fallback/
        // local_fallback failover, System/Agent budget ceilings, DLP
        // forced air-gapping) and records real token usage itself; an
        // agent with no routing configured gets exactly its pre-7a
        // behavior back (see `ai_gateway_service`'s own doc comment).
        let call_started = std::time::Instant::now();
        let gateway_outcome = match ai_gateway_service::dispatch(
            conn,
            workspace_id,
            master_key,
            agent,
            actor,
            &system_prompt,
            &tools,
            &history,
            false,
        )
        .await
        {
            Ok(outcome) => outcome,
            Err(e) => return Err(end_span_with_error(conn, model_span, e)),
        };
        record_model_call(&mut model_span, &gateway_outcome, call_started);
        run_tokens += gateway_outcome.usage.input_tokens + gateway_outcome.usage.output_tokens;
        let outcome = gateway_outcome.outcome;
        match outcome {
            CompletionOutcome::Text(text) => {
                if let Some(schema) = &output_schema {
                    let violations = match serde_json::from_str::<Value>(&text) {
                        Ok(parsed) => validate_output_schema(schema, &parsed),
                        Err(e) => vec![format!("Response is not valid JSON: {e}")],
                    };
                    trace_service::start(
                        conn,
                        "guardrail",
                        "output_schema",
                        json!({"lanesra.guardrail.kind": "output_schema"}),
                    )
                    .end(
                        conn,
                        if violations.is_empty() {
                            SpanStatus::Ok
                        } else {
                            SpanStatus::Error
                        },
                        json!({
                            "lanesra.guardrail.verdict": if violations.is_empty() { "pass" } else { "fail" },
                            "lanesra.guardrail.violations": violations.len(),
                        }),
                    );
                    if !violations.is_empty() && !structured_output_repaired {
                        // One repair retry (spec'd, not indefinite): tell
                        // the model exactly what's wrong and let the next
                        // round of this same loop re-dispatch - if it's
                        // still wrong after that, the second attempt's
                        // text is returned as-is rather than looping
                        // forever on a model that can't self-correct.
                        structured_output_repaired = true;
                        let assistant_msg = synthetic_message("assistant", Some(&text), None, None);
                        history.push(assistant_msg.clone());
                        produced.push(assistant_msg);
                        let (repair_prompt, _) = crate::prompts::render_active(
                            "loop/output-repair",
                            &[("violations", &violations.join("\n"))],
                        )
                        .map_err(AppError::Validation)?;
                        let repair_msg =
                            synthetic_message("user", Some(&repair_prompt), None, None);
                        history.push(repair_msg.clone());
                        produced.push(repair_msg);
                        model_span.end(conn, SpanStatus::Ok, json!({}));
                        continue;
                    }
                }
                produced.push(synthetic_message("assistant", Some(&text), None, None));
                model_span.end(conn, SpanStatus::Ok, json!({}));
                return Ok(AgentRunOutcome {
                    final_text: text,
                    produced,
                    policy_violations,
                    stop_reason: None,
                });
            }
            CompletionOutcome::ToolCalls {
                raw_assistant,
                calls,
            } => {
                match calls.first().filter(|_| calls.len() == 1) {
                    Some(only_call)
                        if last_call.as_ref().is_some_and(|(n, a)| {
                            *n == only_call.name && *a == only_call.arguments
                        }) =>
                    {
                        repeat_streak += 1;
                        if repeat_streak >= 3 {
                            let e = AppError::Validation(format!(
                                "This agent tried to call '{}' with the same input 3 times in a row - stopping to avoid a runaway loop. Escalate to a human if this keeps happening.",
                                only_call.name
                            ));
                            return Err(end_span_with_error(conn, model_span, e));
                        }
                    }
                    Some(only_call) => {
                        repeat_streak = 1;
                        last_call = Some((only_call.name.clone(), only_call.arguments.clone()));
                    }
                    None => {
                        repeat_streak = 0;
                        last_call = None;
                    }
                }
                let persisted_assistant = redact_secret_tool_calls(&raw_assistant);
                let assistant_msg =
                    synthetic_message("assistant", None, Some(&persisted_assistant), None);
                history.push(assistant_msg.clone());
                produced.push(assistant_msg);
                for call in calls {
                    let mut tool_span = trace_service::start(
                        conn,
                        "tool_call",
                        &call.name,
                        json!({
                            "lanesra.tool.name": call.name,
                            "lanesra.tool.source": tool_source(&call.name),
                        }),
                    );
                    tool_span.body("input", &call.arguments.to_string());
                    if let Err(problem) = ai_service::check_tool_call(&call, &tools) {
                        malformed_calls += 1;
                        let tool_msg = synthetic_message(
                            "tool",
                            Some(&malformed_tool_result(&problem)),
                            None,
                            Some(&call.id),
                        );
                        history.push(tool_msg.clone());
                        produced.push(tool_msg);
                        tool_span.end(
                            conn,
                            SpanStatus::Error,
                            json!({"lanesra.tool.outcome": "malformed", "lanesra.error.code": "validation.arguments"}),
                        );
                        if malformed_calls >= MAX_MALFORMED_TOOL_CALLS {
                            return Err(end_span_with_error(
                                conn,
                                model_span,
                                too_many_malformed_calls(),
                            ));
                        }
                        continue;
                    }
                    // WP-0140 AC-1: the binding's caps, checked before the call.
                    let calls_so_far = calls_this_step.entry(call.name.clone()).or_insert(0);
                    let cap = match tool_limits_service::check_binding(
                        conn,
                        &agent.id,
                        &call.name,
                        *calls_so_far,
                    )? {
                        tool_limits_service::CapCheck::Refuse { kind, error } => {
                            let tool_msg = synthetic_message(
                                "tool",
                                Some(&format!("Error: {error}. The tool was not run.")),
                                None,
                                Some(&call.id),
                            );
                            history.push(tool_msg.clone());
                            produced.push(tool_msg);
                            let mut attrs = trace_service::error_attributes(&error);
                            attrs.insert("lanesra.tool.outcome".into(), json!("capped"));
                            attrs.insert("lanesra.tool.cap".into(), json!(kind));
                            tool_span.end(conn, SpanStatus::Error, Value::Object(attrs));
                            continue;
                        }
                        tool_limits_service::CapCheck::Run(cap) => cap,
                    };
                    *calls_so_far += 1;
                    let result = execute_agent_tool(
                        conn,
                        workspace_id,
                        master_key,
                        actor,
                        agent,
                        &call,
                        memory_context,
                    )
                    .await;
                    let ok = result.is_ok();
                    let blocked = matches!(&result, Err(AppError::PolicyBlocked(_)));
                    if blocked {
                        policy_violations += 1;
                    }
                    // A refusal by policy is not the tool failing.
                    let refused =
                        blocked || matches!(&result, Err(e) if e.class() == ErrorClass::Policy);
                    let tool_outcome = match (&result, refused) {
                        (Ok(_), _) => "ok",
                        (Err(_), true) => "blocked",
                        (Err(_), false) => "error",
                    };
                    let tool_error = result.as_ref().err().map(trace_service::error_attributes);
                    let mut content = match result {
                        Ok(value) => value.to_string(),
                        Err(e) => format!("Error: {e}"),
                    };
                    if let Some(cap) = &cap {
                        if !refused && tool_limits_service::record_binding_outcome(conn, cap, ok)? {
                            tool_span.set("lanesra.tool.cap", json!("disabled_after_failures"));
                        }
                        if let Some(max) = cap.max_result_chars {
                            if ok && content.chars().count() as i64 > max {
                                let (replacement, stored_id) =
                                    tool_limits_service::store_large_result(
                                        conn,
                                        workspace_id,
                                        &agent.id,
                                        &call.name,
                                        &content,
                                        max,
                                    )?;
                                tool_span.set("lanesra.tool.cap", json!("result_size"));
                                tool_span.set("lanesra.tool.result_ref", json!(stored_id));
                                tool_span.set(
                                    "lanesra.tool.result_chars",
                                    json!(content.chars().count()),
                                );
                                content = replacement;
                            }
                        }
                    }
                    // AI Agent Platform v2, Phase 1: agent identity attached
                    // to the audit trail alongside the initiating user, not
                    // in place of it - the entity-service call this tool
                    // dispatches to (e.g. `company_service::update`) already
                    // wrote its own `actor`-attributed audit event via the
                    // same `audit_repo::record`; this is an additional,
                    // additive row so "which agent" is answerable too. Never
                    // fails the tool call itself if this write fails.
                    let _ = audit_repo::record(
                        conn,
                        workspace_id,
                        actor,
                        "ai_agent_tool_call",
                        Some("ai_agent"),
                        Some(&agent.id),
                        &format!(
                            "Agent '{}' called '{}'{}",
                            agent.name,
                            call.name,
                            if ok { "" } else { " (failed)" }
                        ),
                        // WP-0109 AC-1: the agent and the principal it acted as.
                        Some(
                            &json!({
                                "agent_id": agent.id,
                                "tool": call.name,
                                "effective_principal": crate::principals::audit_principal(actor),
                            })
                            .to_string(),
                        ),
                    );
                    tool_span.body("output", &content);
                    let mut tool_attrs = tool_error.unwrap_or_default();
                    tool_attrs.insert("lanesra.tool.outcome".into(), json!(tool_outcome));
                    tool_span.end(
                        conn,
                        if ok {
                            SpanStatus::Ok
                        } else {
                            SpanStatus::Error
                        },
                        Value::Object(tool_attrs),
                    );
                    let tool_msg = synthetic_message("tool", Some(&content), None, Some(&call.id));
                    history.push(tool_msg.clone());
                    produced.push(tool_msg);
                }
                model_span.end(conn, SpanStatus::Ok, json!({}));
            }
        }
        if limits
            .run_token_limit
            .is_some_and(|limit| run_tokens >= limit)
        {
            stop_reason = STOP_LIMIT_TOKENS;
            break;
        }
    }
    // WP-0121: a limit was reached. Ask once for a final answer with tools
    // off and keep it as the run's partial result.
    let mut model_span = start_model_call_span(conn, agent, limits.max_rounds + 1, true, &history);
    model_span.set("lanesra.stop_reason", json!(stop_reason));
    let call_started = std::time::Instant::now();
    let gateway_outcome = match ai_gateway_service::dispatch(
        conn,
        workspace_id,
        master_key,
        agent,
        actor,
        &final_answer_prompt(&system_prompt, stop_reason, &limits)?,
        &tools,
        &history,
        true,
    )
    .await
    {
        Ok(outcome) => outcome,
        Err(e) => return Err(end_span_with_error(conn, model_span, e)),
    };
    record_model_call(&mut model_span, &gateway_outcome, call_started);
    model_span.end(conn, SpanStatus::Ok, json!({}));
    let text = final_answer_text(gateway_outcome.outcome);
    produced.push(synthetic_message("assistant", Some(&text), None, None));
    Ok(AgentRunOutcome {
        final_text: text,
        produced,
        policy_violations,
        stop_reason: Some(stop_reason),
    })
}

/// Chatting with one specific named Agent - the third `chat_conversations`
/// mode alongside `send_message`'s fixed `"records"`/`"admin"`. A close
/// structural sibling of `send_message` (same "persist the user's message,
/// run the loop, persist what it produced" shape), not a forced
/// abstraction over it - see this section's own doc comment.
pub async fn send_agent_message(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    user_id: &str,
    agent_id: &str,
    text: &str,
) -> AppResult<Vec<ChatMessage>> {
    if text.trim().is_empty() {
        return Err(AppError::Validation("Say something first".into()));
    }
    let agent = ai_agent_repo::get(conn, agent_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Agent".into()))?;
    if !agent.is_active {
        return Err(AppError::Validation(
            "This agent has been deactivated".into(),
        ));
    }
    if agent_requires_admin(&agent.action_names) {
        require_admin(conn, Some(user_id))?;
    }

    let conversation =
        chat_repo::get_or_create_conversation(conn, workspace_id, user_id, "agent", agent_id)?;
    let mut appended = Vec::new();
    appended.push(chat_repo::append_message(
        conn,
        &conversation.id,
        "user",
        Some(text),
        None,
        None,
    )?);

    let history = chat_repo::list_messages(conn, &conversation.id)?;
    // AI Agent Platform v2, Phase 4: this conversation's own durable id is
    // exactly the "session" Session Memory is scoped by - this agent's
    // ongoing relationship with this user, the same conversation row
    // `get_or_create_conversation` already keeps stable across messages.
    let memory_context = AgentMemoryContext {
        session_key: Some(conversation.id.clone()),
        run_id: None,
    };
    let outcome = run_agent_once(
        conn,
        workspace_id,
        master_key,
        Some(user_id),
        &agent,
        history,
        &memory_context,
    )
    .await?;
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

pub fn get_agent_history(
    conn: &Connection,
    workspace_id: &str,
    user_id: &str,
    agent_id: &str,
) -> AppResult<Vec<ChatMessage>> {
    let conversation =
        chat_repo::get_or_create_conversation(conn, workspace_id, user_id, "agent", agent_id)?;
    Ok(chat_repo::list_messages(conn, &conversation.id)?)
}
