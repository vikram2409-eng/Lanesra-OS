-- Migration 0092: per-tool taint exemption on agent policies (WP-0145)
-- Work package: WP-0145
-- Change: additive (one defaulted column on ai_agent_policies)
-- Data touched: none; existing policies exempt no tool ('[]')
-- Re-run safe: no (ALTER TABLE ADD COLUMN)
--
-- WP-0145 contains prompt injection through tool results: once a step has
-- read untrusted content (a connector, an external MCP tool, web or email),
-- a later write-tier or external-action tool call in that step needs
-- administrator approval by default. An Administrator can relax that for a
-- specific tool by listing its name here: a tool on this list runs after an
-- untrusted read without the taint-driven approval (the ordinary firewall
-- risk/blocklist still applies). Empty '[]' is the default, so a workspace
-- that never opens this screen sees the safe default - approval required.
ALTER TABLE ai_agent_policies
    ADD COLUMN taint_exempt_tool_names_json TEXT NOT NULL DEFAULT '[]';
