//! AI Agent Platform v2 (GitHub issue #170, backend half): the MCP
//! *client* role. See migration `0064_mcp_client.sql`'s own doc comment
//! for why an external MCP server is modeled as an ordinary
//! `integration_connections` row (`connection_type = "mcp"`) with this
//! module's `McpServer`/`McpTool` layered on top for the MCP-specific
//! bits - which tools the last discovery found, and which of those an
//! admin has actually opted into exposing to agents.

#[derive(Debug, Clone, serde::Serialize)]
pub struct McpServer {
    pub id: String,
    pub workspace_id: String,
    pub connection_id: String,
    pub name: String,
    pub base_url: Option<String>,
    pub auth_mode: String,
    pub agent_tools_enabled: bool,
    pub agent_write_tools_enabled: bool,
    pub last_discovered_at: Option<String>,
    pub last_discovery_status: Option<String>,
    pub last_discovery_message: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct McpServerInput {
    pub name: String,
    pub base_url: String,
    #[serde(default = "default_auth_mode")]
    pub auth_mode: String,
    #[serde(default)]
    pub secret_value: Option<String>,
}
fn default_auth_mode() -> String {
    "none".into()
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct McpServerUpdate {
    pub name: String,
    pub base_url: String,
    pub auth_mode: String,
    #[serde(default)]
    pub secret_value: Option<String>,
    pub agent_tools_enabled: bool,
    pub agent_write_tools_enabled: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct McpTool {
    pub id: String,
    pub mcp_server_id: String,
    pub tool_name: String,
    pub description: String,
    pub input_schema_json: String,
    pub is_write: bool,
    pub enabled: bool,
    pub discovered_at: String,
}

/// One tool a real `tools/list` call reported, before it's persisted -
/// `mcp_client_service::discover_tools` maps the external server's own
/// response into these, then `ai_mcp_repo::replace_tools` reconciles them
/// against whatever `is_write`/`enabled` an admin already set on a
/// same-named tool from a prior discovery (see that function's own doc
/// comment for exactly what's preserved across a re-discovery).
#[derive(Debug, Clone)]
pub struct DiscoveredTool {
    pub tool_name: String,
    pub description: String,
    pub input_schema_json: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct McpToolFlagsInput {
    pub is_write: bool,
    pub enabled: bool,
}

/// The eligible set of this workspace's MCP tools, described for an
/// admin's Actions-checklist UI - the MCP-client mirror of
/// `AgentConnectorToolOption`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentMcpToolOption {
    pub tool_name: String,
    pub mcp_server_id: String,
    pub mcp_server_name: String,
    pub tool_description: String,
    pub requires_admin: bool,
}
