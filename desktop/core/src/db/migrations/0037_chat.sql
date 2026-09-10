-- AI & Agentic Layer, Phase 5: the LLM Chat Assistant's persistence -
-- one conversation per (user, mode), reused across visits rather than
-- recreated each time (see services::chat_service's own doc comment).
-- `mode` is 'records' (any authenticated user, read/write over
-- api_object_service) or 'admin' (Administrator only, read/write over
-- the admin configuration surface).
CREATE TABLE chat_conversations (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mode TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_chat_conversations_user_mode ON chat_conversations (user_id, mode);

-- role is 'user' | 'assistant' | 'tool'. tool_calls_json carries the raw
-- assistant tool-call request(s) (provider call id, tool name,
-- arguments) when role='assistant' and the turn requested tools rather
-- than answering directly; tool_call_id links a role='tool' result back
-- to the specific call it answers - both mirror exactly what each
-- provider's own API needs echoed back on the next turn (see
-- ai_service::complete_with_tools's own doc comment), not a new shape
-- invented here.
CREATE TABLE chat_messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES chat_conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT,
    tool_calls_json TEXT,
    tool_call_id TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_chat_messages_conversation ON chat_messages (conversation_id, created_at);
