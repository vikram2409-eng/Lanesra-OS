-- AI Agent Platform v2, Phase 4 (GitHub issue #169): Memory Architecture &
-- Document RAG.
--
-- Memory Architecture: today's per-agent `memory_md` (+ its own
-- `ai_agent_memory_history` snapshot log, migration 0042) stays exactly as
-- it is - that's the "Agent Memory" arm of the four-type model the spec
-- asks for, unchanged. `ai_memory_items` below is new storage for the
-- three genuinely new types - Session, Working and Entity Memory - each a
-- discrete, itemized fact rather than one overwritten blob, since that's
-- what "inspect and delete one retained memory" (this issue's own
-- acceptance language) actually requires. See
-- `services::ai_memory_service`'s own doc comment for exactly what each
-- type is scoped by and how long it lives.
--
-- Document RAG: `ai_knowledge_collections` (a named, addressable grouping -
-- "the same way a Saved View is already an agent knowledge scope", this
-- issue's own words) hold `ai_knowledge_sources` (an admin-provided text
-- document), each chunked into `ai_knowledge_chunks` and embedded via the
-- workspace's already-configured provider - see
-- `services::ai_knowledge_service`'s own doc comment for the honest scope
-- line on what "ingestion" means in this pass (plain text, not binary
-- file parsing).
CREATE TABLE ai_memory_items (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    -- 'session' | 'working' | 'entity'. See models::ai_memory::MEMORY_TYPES.
    memory_type TEXT NOT NULL,
    -- The agent this item was produced by/for. NULL is reserved (not used
    -- by any writer in this pass) for a future workspace-wide item not
    -- tied to one agent's own capture.
    agent_id TEXT REFERENCES ai_agents(id) ON DELETE CASCADE,
    -- Session Memory's scope key: this agent's ongoing relationship with
    -- one user (see ai_memory_service's own doc comment on why that's the
    -- honest session boundary this codebase has today, absent a first-class
    -- conversation/session-id primitive). NULL for working/entity items.
    session_key TEXT,
    -- Working Memory's scope key: a real ai_runs.id (Execution Graph) or
    -- ai_agent_runs.id (Pipeline) - i.e. only ever set inside an actual
    -- durable run, never an interactive chat message. NULL for
    -- session/entity items.
    run_id TEXT,
    -- Entity Memory's scope: a durable fact tied to one record, addressed
    -- the same (entity_type, entity_id) shape entity_registry::resolve
    -- already uses. Both set together or both NULL.
    entity_type TEXT,
    entity_id TEXT,
    content TEXT NOT NULL,
    -- Where this fact came from - free text (e.g. "agent_inference",
    -- "user_provided", "tool_result") for the audit/inspection view, not a
    -- closed enum this pass needs to validate against.
    source TEXT NOT NULL DEFAULT 'agent_inference',
    -- 0.0-1.0, NULL when the writer didn't supply one.
    confidence REAL,
    -- 'standard' | 'sensitive' | 'restricted'. See models::ai_memory::
    -- CLASSIFICATIONS. 'restricted' is excluded from durable persistence by
    -- a workspace/agent's policy default - see
    -- services::policy_engine_service::evaluate_memory_write.
    classification TEXT NOT NULL DEFAULT 'standard',
    created_at TEXT NOT NULL,
    created_by TEXT,
    -- NULL = never expires (Entity Memory's normal case - a durable fact
    -- persists until an admin/user explicitly forgets it). Session/Working
    -- items are normally written with a real expires_at (a short TTL) so
    -- ai_memory_service::sweep_expired has something to actually reclaim.
    expires_at TEXT,
    last_used_at TEXT
);
CREATE INDEX idx_ai_memory_items_workspace_type ON ai_memory_items (workspace_id, memory_type);
CREATE INDEX idx_ai_memory_items_entity ON ai_memory_items (entity_type, entity_id);
CREATE INDEX idx_ai_memory_items_session ON ai_memory_items (agent_id, session_key);
CREATE INDEX idx_ai_memory_items_run ON ai_memory_items (agent_id, run_id);
CREATE INDEX idx_ai_memory_items_expires ON ai_memory_items (expires_at) WHERE expires_at IS NOT NULL;

CREATE TABLE ai_knowledge_collections (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT
);
CREATE INDEX idx_ai_knowledge_collections_workspace ON ai_knowledge_collections (workspace_id);

CREATE TABLE ai_knowledge_sources (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    -- NULL = not in any collection - still searchable workspace-wide,
    -- matching a Saved View's own optional-grouping shape.
    collection_id TEXT REFERENCES ai_knowledge_collections(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    -- The full source text this pass ingests - see this migration's own
    -- top doc comment on why that's the honest scope line, not a file
    -- reference. Re-chunked and re-embedded in place on update
    -- (services::ai_knowledge_service::update_source), which is what
    -- "re-index on version change" means here.
    content TEXT NOT NULL,
    -- 'indexed' | 'failed'. A source only exists once its chunks are
    -- embedded (create_source is one atomic step - see that function's own
    -- doc comment on why this pass doesn't queue-and-drain the way record
    -- embeddings do), so 'failed' is set only if re-embedding on an update
    -- errors after the source row already existed.
    status TEXT NOT NULL DEFAULT 'indexed',
    chunk_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT
);
CREATE INDEX idx_ai_knowledge_sources_workspace ON ai_knowledge_sources (workspace_id);
CREATE INDEX idx_ai_knowledge_sources_collection ON ai_knowledge_sources (collection_id);

CREATE TABLE ai_knowledge_chunks (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES ai_knowledge_sources(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    -- NULL until embedded; a source's chunks are embedded synchronously as
    -- part of create/update (no pending-queue table here - see this
    -- migration's own top doc comment).
    embedding BLOB,
    dim INTEGER,
    model TEXT,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_ai_knowledge_chunks_source ON ai_knowledge_chunks (source_id);
CREATE INDEX idx_ai_knowledge_chunks_workspace ON ai_knowledge_chunks (workspace_id);

-- Phase 2's Policy Engine gate for Entity/Session Memory persistence (issue
-- #169's own "ties into PR 2's Policy Engine" dependency line). Defaults to
-- excluding 'restricted'-classified content from durable memory, the safe
-- default this issue's own scope text asks for ("Restricted-classified
-- fields may be excluded from durable memory by policy") - an admin may
-- relax it per-agent or workspace-wide exactly like every other
-- ai_agent_policies column already works (agent-specific row wins, else the
-- workspace default row, else this column's own DEFAULT).
ALTER TABLE ai_agent_policies ADD COLUMN exclude_restricted_memory INTEGER NOT NULL DEFAULT 1;
