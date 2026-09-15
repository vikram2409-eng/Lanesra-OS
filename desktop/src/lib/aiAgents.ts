import type { AiAgentDefinition } from "./types";

// AI & Agentic Layer, Phase 6: a client-side mirror of
// chat_service.rs's own `agent_requires_admin` - so the main "Assistant"
// nav page can filter which agents to offer a non-admin user without a
// round trip. Purely a UI convenience: `send_agent_message` re-checks for
// real regardless (defense in depth, same as everywhere else in this
// codebase). Kept as a flat name list here (not the full ADMIN_ACTIONS
// label pairs AiAgentsAdmin.tsx's form needs) - same "hardcoded mirror of
// the server's own catalog, synced by hand" precedent the online demo's
// MCP_TOOLS list already established for server/src/mcp.rs.
const ADMIN_ACTION_NAMES = new Set([
  "list_business_rules",
  "create_business_rule",
  "list_workflows",
  "create_workflow",
  "list_custom_objects",
  "create_custom_object",
  "list_custom_fields",
  "create_custom_field",
  "list_relationships",
  "create_relationship",
  "list_status_transitions",
  "create_status_transition",
  "list_connections",
  "create_connection",
  "list_webhooks",
  "create_webhook",
  "list_integration_jobs",
  "create_integration_job",
  "list_api_clients",
  "create_api_client",
  "list_dashboards",
  "create_dashboard",
  "list_apps",
  "create_app",
  "list_custom_reports",
  "create_custom_report",
  "list_saved_views",
  "create_saved_view",
  "list_users",
  "create_user",
  "list_numbering",
  "set_numbering",
  "get_workspace_profile",
  "update_workspace_profile",
  "list_connectors",
  "list_ai_agents",
  "create_ai_agent",
  "list_ai_skills",
  "create_ai_skill",
]);

// Integration Hub Tool Bridge: a write-capable Connector Action tool
// self-describes by name prefix (`connector_tool_service.rs`'s own doc
// comment - "connector_write_action:{connector_id}:{action_key}"), so
// this mirror can classify it without a round trip, same as the fixed
// ADMIN_ACTION_NAMES set above.
const CONNECTOR_WRITE_ACTION_PREFIX = "connector_write_action:";

export function agentRequiresAdmin(actionNames: string[]): boolean {
  return actionNames.some((n) => ADMIN_ACTION_NAMES.has(n) || n.startsWith(CONNECTOR_WRITE_ACTION_PREFIX));
}

export function agentUsableBy(agent: AiAgentDefinition, isAdmin: boolean): boolean {
  return agent.is_active && (isAdmin || !agentRequiresAdmin(agent.action_names));
}
