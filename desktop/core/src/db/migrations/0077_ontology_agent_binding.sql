-- Next-Gen program, Ontology Layer epic, issue #344: AI Agent Ontology
-- Binding - purely additive, zero behavior change for every agent that
-- predates this feature. `'[]'` (the default) means "unscoped" - exactly
-- today's all-or-nothing-by-tool-name behavior, preserved unchanged; an
-- agent only gets narrower once an admin actually sets one of these.
--
-- `object_type_scopes_json`: `[{"object_type": "Company", "capabilities":
-- ["read","update"]}, ...]` - checked in chat_service's dispatch for the
-- generic record tools (list_records/create_record/update_record/...)
-- before they ever reach api_object_service.
--
-- `action_type_ids_json`: `["<action_type_definitions.id>", ...]` - which
-- specific Action Types (issue #343) this agent's new `invoke_action`
-- tool may call.
ALTER TABLE ai_agents ADD COLUMN object_type_scopes_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE ai_agents ADD COLUMN action_type_ids_json TEXT NOT NULL DEFAULT '[]';
