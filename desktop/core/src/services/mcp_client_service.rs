//! AI Agent Platform v2 (GitHub issue #170, backend half): the MCP
//! **client** role - Lanesra calling OUT to an external MCP server, the
//! mirror image of the MCP **server** role `desktop/server/src/mcp.rs`
//! already exposes. That endpoint's own doc comment is this client's
//! contract too: `initialize`/`tools/list`/`tools/call` over one
//! stateless `POST`, JSON-RPC 2.0, `Authorization: Bearer`-style auth,
//! and a tool-level failure returned as `{"content":[...], "isError":
//! true}` rather than a broken connection - handled the same
//! `AppResult::Err` way `chat_service`'s tool loop already turns any
//! failed tool call into an `"Error: ..."` message the model can see and
//! react to.
//!
//! An external MCP server is an ordinary `integration_connections` row
//! (`connection_type = "mcp"`) wrapped by an `ai_mcp_servers` row for the
//! MCP-specific bits - see migration `0064_mcp_client.sql`'s own doc
//! comment. Everything below that isn't a plain CRUD wrapper mirrors
//! `connector_tool_service.rs` file-for-file: same name-prefix
//! self-description (`mcp_tool:{server_id}:{tool_name}` read-only,
//! `mcp_write_tool:{server_id}:{tool_name}` write), same
//! re-resolve-against-live-data-at-call-time discipline, same
//! `agent_tools`/`agent_tool_names`/`list_options`/`dispatch` shape -
//! `chat_service::tool_source`/`agent_requires_admin` and
//! `ai_agent_service::validate_action_names` gain one more classified
//! source (`"mcp_read"`/`"mcp_write"`) exactly the way they already
//! handle `"connector_read"`/`"connector_write"`.

use std::time::Duration;

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::domain::{AppError, AppResult};
use crate::models::ai_mcp::{AgentMcpToolOption, DiscoveredTool, McpServer, McpServerInput, McpServerUpdate, McpTool};
use crate::models::integration::{ConnectionInput, ConnectionUpdate};
use crate::repositories::ai_mcp_repo;
use crate::services::ai_service::ToolSpec;
use crate::services::connection_service;
use crate::services::integration_log_service::{self, FinishOutcome};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

const REQUEST_TIMEOUT_MS: u64 = 10_000;

fn get_owned(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<McpServer> {
    let server = ai_mcp_repo::get_server(conn, id)?.ok_or_else(|| AppError::NotFound("MCP server".into()))?;
    if server.workspace_id != workspace_id {
        return Err(AppError::NotFound("MCP server".into()));
    }
    Ok(server)
}

pub fn create_server(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], input: &McpServerInput, actor_user_id: Option<&str>) -> AppResult<McpServer> {
    require_admin(conn, actor_user_id)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name is required".into()));
    }
    if input.base_url.trim().is_empty() {
        return Err(AppError::Validation("Server URL is required".into()));
    }
    let connection = connection_service::create(
        conn,
        workspace_id,
        master_key,
        &ConnectionInput {
            name: input.name.trim().to_string(),
            connection_type: "mcp".into(),
            base_url: Some(input.base_url.trim().to_string()),
            auth_mode: input.auth_mode.clone(),
            secret_value: input.secret_value.clone(),
            config_json: "{}".into(),
            owner_user_id: None,
        },
        actor_user_id,
    )?;
    ai_mcp_repo::create_server(conn, workspace_id, &connection.id, input.name.trim(), actor_user_id).map_err(AppError::from)
}

pub fn get_server(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<McpServer> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)
}

pub fn list_servers(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<McpServer>> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_mcp_repo::list_servers_for_workspace(conn, workspace_id)?)
}

pub fn update_server(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], id: &str, input: &McpServerUpdate, actor_user_id: Option<&str>) -> AppResult<McpServer> {
    require_admin(conn, actor_user_id)?;
    let server = get_owned(conn, workspace_id, id)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name is required".into()));
    }
    if input.base_url.trim().is_empty() {
        return Err(AppError::Validation("Server URL is required".into()));
    }
    connection_service::update(
        conn,
        workspace_id,
        master_key,
        &server.connection_id,
        &ConnectionUpdate {
            name: input.name.trim().to_string(),
            base_url: Some(input.base_url.trim().to_string()),
            auth_mode: input.auth_mode.clone(),
            secret_value: input.secret_value.clone(),
            config_json: "{}".into(),
            owner_user_id: None,
            status: "active".into(),
        },
        actor_user_id,
    )?;
    ai_mcp_repo::update_server(conn, id, input.name.trim(), input.agent_tools_enabled, input.agent_write_tools_enabled, actor_user_id)?;
    get_owned(conn, workspace_id, id)
}

pub fn delete_server(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let server = get_owned(conn, workspace_id, id)?;
    ai_mcp_repo::delete_server(conn, id)?;
    connection_service::delete(conn, workspace_id, &server.connection_id, actor_user_id)
}

pub fn list_tools(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<McpTool>> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    Ok(ai_mcp_repo::list_tools(conn, id)?)
}

pub fn set_tool_flags(conn: &Connection, workspace_id: &str, id: &str, tool_name: &str, is_write: bool, enabled: bool, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    let updated = ai_mcp_repo::set_tool_flags(conn, id, tool_name, is_write, enabled)?;
    if updated == 0 {
        return Err(AppError::NotFound("MCP tool".into()));
    }
    Ok(())
}

/// One raw JSON-RPC 2.0 round trip against `server`'s own endpoint -
/// shared by `discover_tools` (`initialize` + `tools/list`) and
/// `call_tool` (`tools/call`). Returns the `result` value on a JSON-RPC
/// success, or an `Err` built from either a protocol-level `error` object
/// or a genuine network/transport failure - the caller can't tell those
/// apart and doesn't need to, per this module's own doc comment on how a
/// failed call already flows back to the model either way.
async fn rpc_call(base_url: &str, auth_mode: &str, secret: Option<&str>, method: &str, params: Value) -> AppResult<Value> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(REQUEST_TIMEOUT_MS))
        .build()
        .map_err(|e| AppError::Validation(format!("Could not build HTTP client: {e}")))?;
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
    let mut builder = client.post(base_url).json(&body);
    builder = connection_service::apply_auth(builder, auth_mode, secret);
    let response = builder.send().await.map_err(|e| AppError::Validation(format!("Could not reach {base_url}: {e}")))?;
    let status = response.status();
    let parsed: Value = response.json().await.map_err(|e| AppError::Validation(format!("'{method}' returned a response that wasn't valid JSON: {e}")))?;
    if !status.is_success() {
        return Err(AppError::Validation(format!("'{method}' returned HTTP {status}")));
    }
    if let Some(error) = parsed.get("error") {
        let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown MCP protocol error");
        return Err(AppError::Validation(format!("MCP server rejected '{method}': {message}")));
    }
    Ok(parsed.get("result").cloned().unwrap_or(Value::Null))
}

/// Real `initialize` + `tools/list` handshake against this server's own
/// endpoint - `ai_mcp_repo::replace_tools` reconciles the result against
/// whatever `is_write`/`enabled` flags an admin already set on a
/// same-named tool from a prior discovery, so re-discovering never resets
/// an admin's own classification/allowlist choices.
pub async fn discover_tools(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<McpTool>> {
    require_admin(conn, actor_user_id)?;
    let server = get_owned(conn, workspace_id, id)?;
    let base_url = server.base_url.as_deref().ok_or_else(|| AppError::Validation("This MCP server has no URL configured".into()))?;
    let secret = connection_service::resolve_secret(conn, master_key, &server.connection_id)?;

    let outcome = async {
        rpc_call(base_url, &server.auth_mode, secret.as_deref(), "initialize", json!({})).await?;
        let list_result = rpc_call(base_url, &server.auth_mode, secret.as_deref(), "tools/list", json!({})).await?;
        let tools = list_result.get("tools").and_then(|t| t.as_array()).cloned().unwrap_or_default();
        let discovered: Vec<DiscoveredTool> = tools
            .into_iter()
            .filter_map(|t| {
                let tool_name = t.get("name").and_then(|n| n.as_str())?.to_string();
                let description = t.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();
                let input_schema_json = t.get("inputSchema").map(|s| s.to_string()).unwrap_or_else(|| "{}".to_string());
                Some(DiscoveredTool { tool_name, description, input_schema_json })
            })
            .collect();
        AppResult::Ok(discovered)
    }
    .await;

    match outcome {
        Ok(discovered) => {
            let n = discovered.len();
            let tools = ai_mcp_repo::replace_tools(conn, id, &discovered)?;
            ai_mcp_repo::set_discovery_result(conn, id, "connected", &format!("Discovered {n} tool{}", if n == 1 { "" } else { "s" }))?;
            Ok(tools)
        }
        Err(e) => {
            ai_mcp_repo::set_discovery_result(conn, id, "failed", &e.to_string())?;
            Err(e)
        }
    }
}

fn tool_prefix(is_write: bool) -> &'static str {
    if is_write { "mcp_write_tool" } else { "mcp_tool" }
}

fn tool_spec_for(server: &McpServer, tool: &McpTool) -> ToolSpec {
    let input_schema = serde_json::from_str(&tool.input_schema_json).unwrap_or_else(|_| json!({"type": "object", "properties": {}}));
    ToolSpec {
        name: format!("{}:{}:{}", tool_prefix(tool.is_write), server.id, tool.tool_name),
        description: format!(
            "{} - a tool from the external MCP server '{}'.{}",
            if tool.description.is_empty() { tool.tool_name.as_str() } else { tool.description.as_str() },
            server.name,
            if tool.is_write { " This performs a real action against an external system - use only when the task genuinely calls for it." } else { "" }
        ),
        input_schema,
    }
}

fn build_entries(conn: &Connection, workspace_id: &str) -> AppResult<Vec<(McpServer, McpTool, ToolSpec)>> {
    Ok(ai_mcp_repo::list_enabled_tools_for_workspace(conn, workspace_id)?
        .into_iter()
        .filter(|(server, tool)| !tool.is_write || server.agent_write_tools_enabled)
        .map(|(server, tool)| {
            let spec = tool_spec_for(&server, &tool);
            (server, tool, spec)
        })
        .collect())
}

/// Every currently-eligible MCP tool in this workspace, as a `ToolSpec`
/// ready to offer a model - see `chat_service::agent_tools`, which chains
/// this in alongside the fixed record/admin catalogs and the Connector
/// Tool Bridge.
pub fn agent_tools(conn: &Connection, workspace_id: &str) -> AppResult<Vec<ToolSpec>> {
    Ok(build_entries(conn, workspace_id)?.into_iter().map(|(_, _, spec)| spec).collect())
}

/// Just the names - `ai_agent_service::validate_action_names` uses this
/// to reject an `mcp_(write_)?tool:` entry that doesn't currently resolve
/// to a real, enabled tool.
pub fn agent_tool_names(conn: &Connection, workspace_id: &str) -> AppResult<Vec<String>> {
    Ok(build_entries(conn, workspace_id)?.into_iter().map(|(_, _, spec)| spec.name).collect())
}

/// The same eligible set, described for an admin's Actions-checklist UI.
pub fn list_options(conn: &Connection, workspace_id: &str) -> AppResult<Vec<AgentMcpToolOption>> {
    Ok(build_entries(conn, workspace_id)?
        .into_iter()
        .map(|(server, tool, spec)| AgentMcpToolOption {
            tool_name: spec.name,
            mcp_server_id: server.id,
            mcp_server_name: server.name,
            tool_description: tool.description,
            requires_admin: tool.is_write,
        })
        .collect())
}

fn parse_tool_name(name: &str) -> Option<(String, String)> {
    let rest = name.strip_prefix("mcp_tool:").or_else(|| name.strip_prefix("mcp_write_tool:"))?;
    let (server_id, tool_name) = rest.split_once(':')?;
    Some((server_id.to_string(), tool_name.to_string()))
}

/// Executes one real outbound `tools/call` a model made. Re-validates
/// eligibility against the current data rather than trusting that the
/// name was valid when the agent was last saved - revoking a server's
/// agent-tool opt-in (or a tool's own allowlist/write flag) takes effect
/// on the next call, not just on the next save, the same discipline
/// `connector_tool_service::dispatch` already established. Logged through
/// the same unified `integration_executions` log every other outbound
/// call in this subsystem uses (`integration_log_service`), never a
/// second, MCP-only log.
pub async fn dispatch(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], actor_user_id: Option<&str>, name: &str, arguments: &Value) -> AppResult<Value> {
    let (server_id, tool_name) = parse_tool_name(name).ok_or_else(|| AppError::Validation(format!("'{name}' is not an MCP tool name")))?;
    let entries = build_entries(conn, workspace_id)?;
    let (server, _, _) = entries
        .into_iter()
        .find(|(s, t, _)| s.id == server_id && t.tool_name == tool_name)
        .ok_or_else(|| AppError::Validation(format!("'{name}' is not currently available as an agent tool")))?;
    let execution_id = integration_log_service::start(conn, workspace_id, "mcp_tool_call", None, Some(&server.id), "outbound", actor_user_id);

    let outcome = async {
        let base_url = server.base_url.as_deref().ok_or_else(|| AppError::Validation("This MCP server has no URL configured".into()))?;
        let secret = connection_service::resolve_secret(conn, master_key, &server.connection_id)?;
        let result = rpc_call(base_url, &server.auth_mode, secret.as_deref(), "tools/call", json!({"name": tool_name, "arguments": arguments})).await?;
        if result.get("isError").and_then(|v| v.as_bool()).unwrap_or(false) {
            let message = result
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|a| a.first())
                .and_then(|f| f.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("The external MCP tool reported an error")
                .to_string();
            return Err(AppError::Validation(message));
        }
        AppResult::Ok(result)
    }
    .await;

    match &outcome {
        Ok(_) => integration_log_service::finish(conn, &execution_id, &FinishOutcome { status: "success".into(), records_written: 1, ..Default::default() }),
        Err(e) => integration_log_service::finish(conn, &execution_id, &FinishOutcome { status: "failed".into(), records_failed: 1, error_message: Some(e.to_string()), ..Default::default() }),
    }
    outcome
}
