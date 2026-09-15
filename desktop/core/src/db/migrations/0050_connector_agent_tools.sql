-- Integration Hub Tool Bridge: lets an admin expose a Connector's own
-- Actions as tools an AI Agent Foundry agent can call, phased read-only
-- first with write access behind a further explicit opt-in - see
-- `connector_tool_service.rs` for how these three columns gate tool
-- generation and `chat_service.rs`'s `tool_source`/`agent_requires_admin`
-- for how a write-capable connector tool forces an agent Administrator-
-- only, the same pattern every admin-catalog tool already gets.
--
-- `agent_tools_enabled`: this connector's read-only (GET/HEAD/OPTIONS)
-- Actions become candidate agent tools when true (still individually
-- gated per-action by a resolvable `request_schema_json` for any action
-- with a body - see connector_service.rs's `is_locally_typed`). Off by
-- default, same "explicit opt-in" default `AiAgent.is_active`-shaped
-- flags in this codebase already use, not an overloaded status column
-- like `Connection.status`.
-- `agent_write_tools_enabled`: a second, separate opt-in - only once
-- both flags are true do this connector's mutating (POST/PUT/PATCH/
-- DELETE) Actions become candidate agent tools too.
-- `agent_reference_key`: the one Connection Reference used whenever any
-- of this connector's actions run as an agent tool - resolved with
-- `integration_connection_ref_repo::get_by_key`, same existence-only
-- validation Workflow Automation's own `call_connector_action` already
-- applies to a reference key. Required (checked at the service layer,
-- not the DB) whenever either flag above is set.
ALTER TABLE integration_connectors ADD COLUMN agent_tools_enabled INTEGER NOT NULL DEFAULT 0;
ALTER TABLE integration_connectors ADD COLUMN agent_write_tools_enabled INTEGER NOT NULL DEFAULT 0;
ALTER TABLE integration_connectors ADD COLUMN agent_reference_key TEXT;
