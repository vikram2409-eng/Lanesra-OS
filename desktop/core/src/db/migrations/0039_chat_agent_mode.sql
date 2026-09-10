-- AI & Agentic Layer, Phase 6: extends Phase 5's chat persistence
-- (migration 0037) with a third mode, 'agent' - chatting with a specific
-- named AI Agent from the Foundry, rather than the fixed general
-- records/admin assistants. One conversation per (user, mode, agent_id),
-- same "own your data" reuse as before; agent_id is '' (not NULL, so the
-- unique index keeps working) for mode='records'/'admin', and a real
-- ai_agents.id for mode='agent'.
ALTER TABLE chat_conversations ADD COLUMN agent_id TEXT NOT NULL DEFAULT '';

DROP INDEX idx_chat_conversations_user_mode;
CREATE UNIQUE INDEX idx_chat_conversations_user_mode_agent ON chat_conversations (user_id, mode, agent_id);
