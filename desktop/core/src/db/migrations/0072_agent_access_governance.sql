-- Agent Access Governance (issue #245): closes the gap between the Tool-
-- Call Firewall's own claim ("every tool call") and reality, and gives
-- Access Control v1 actual teeth over AI-driven record writes - both
-- purely additive, zero behavior change for a workspace that never opens
-- either new screen.
--
-- `acts_as_user_id` is an agent's own optional bound identity: when set,
-- it's the effective actor `access_service::require_capability` checks
-- against for this agent's own record writes (instead of whichever human
-- happens to be chatting with it, if any). `NULL` (the default) changes
-- nothing - see `chat_service`'s own doc comment on where this is read.
ALTER TABLE ai_agents ADD COLUMN acts_as_user_id TEXT REFERENCES users(id);

-- Resolved the exact same way `exclude_restricted_memory` already is
-- (agent-specific policy row, else the workspace-default row, else
-- `false` - today's unscoped behavior, unchanged). When `true`,
-- `create_record`/`update_record`/`archive_record` calls this agent (or,
-- for the fixed "records"/"admin" assistants under the workspace-default
-- policy, the real signed-in user) makes are checked against
-- `access_service::require_capability` for real, instead of the
-- `actor_user_id: None` "unattributed/system" convention every AI-driven
-- write has always used.
ALTER TABLE ai_agent_policies ADD COLUMN enforce_record_access INTEGER NOT NULL DEFAULT 0;
