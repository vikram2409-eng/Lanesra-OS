-- AI & Agentic Layer, Phase 7g (part 2): vector search over Custom Object
-- records - real embeddings from the workspace's already-configured LLM
-- provider (openai_compatible/google_gemini only - Anthropic has no
-- embeddings API, see services::ai_service::embed_texts's own doc
-- comment), stored as plain BLOBs here and compared by brute-force cosine
-- similarity in Rust (services::vector_search_service). This reverses
-- this project's own long-stated "no vector database, bm25 is enough at
-- this scale" position (see migration 0042_context_layer.sql's own doc
-- comment) - a deliberate choice for this pass, not a silent one; see
-- vector_search_service's own doc comment for what changed. No sqlite-vec
-- or other native extension either - still no new dependency, same
-- reasoning bm25() was chosen for over one originally.
--
-- A synchronous SQL trigger can't make an HTTP call, so - unlike
-- record_search_fts's triggers, which compute the indexed text inline -
-- the triggers below only ever enqueue a pending_record_embeddings row;
-- the real provider call happens only when
-- vector_search_service::drain_pending_embeddings runs (desktop's
-- job_scheduler tick, alongside ai_orchestration_service::
-- drain_pending_runs, or the admin's own "Reindex now" action). This is
-- the same "enqueue now, drain later" shape ai_agent_pending_runs
-- (migration 0040) already established.

-- Own admin dial on ai_settings, same "own field, own setter" shape
-- otlp_endpoint (migration 0047) already uses. NULL/blank means "use a
-- sensible per-provider default" (see embed_texts's own doc comment),
-- not "no embedding model" - there's no separate on/off switch, since
-- reindexing simply never runs for a workspace with no provider key.
ALTER TABLE ai_settings ADD COLUMN embedding_model TEXT;

-- One row per embedded Custom Object record - `record_id` is the primary
-- key (a record has at most one embedding, always its latest), not a
-- surrogate id, since nothing ever needs to reference an embedding row
-- except by the record it belongs to.
CREATE TABLE record_embeddings (
    workspace_id TEXT NOT NULL,
    object_key TEXT NOT NULL,
    record_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL,
    dim INTEGER NOT NULL,
    model TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_record_embeddings_workspace_object ON record_embeddings (workspace_id, object_key);

-- The reindex queue - `record_id` is again the primary key, so rapid
-- consecutive edits to the same record before the next drain collapse
-- into one pending row rather than piling up (drain always reads the
-- record's current text fresh, so only "does this record need
-- reindexing at all" matters, not how many times it changed).
CREATE TABLE pending_record_embeddings (
    workspace_id TEXT NOT NULL,
    object_key TEXT NOT NULL,
    record_id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_pending_record_embeddings_workspace ON pending_record_embeddings (workspace_id, created_at);

-- custom_records: insert enqueues reindexing (a brand-new record has no
-- field values yet, but still gets a name/number worth embedding);
-- update enqueues again while active, or - the moment it's archived -
-- drops both the pending work and any embedding it already had (an
-- archived record should never surface in a semantic search result, the
-- same rule record_search_fts's own trigger already enforces for FTS);
-- delete removes both outright.
CREATE TRIGGER trg_pending_embed_ins_record AFTER INSERT ON custom_records
BEGIN
    INSERT INTO pending_record_embeddings (workspace_id, object_key, record_id, created_at)
    VALUES (NEW.workspace_id, NEW.object_key, NEW.id, datetime('now'))
    ON CONFLICT (record_id) DO NOTHING;
END;

CREATE TRIGGER trg_pending_embed_upd_record AFTER UPDATE ON custom_records
BEGIN
    DELETE FROM record_embeddings WHERE record_id = NEW.id AND NEW.archived_at IS NOT NULL;
    DELETE FROM pending_record_embeddings WHERE record_id = NEW.id AND NEW.archived_at IS NOT NULL;
    INSERT INTO pending_record_embeddings (workspace_id, object_key, record_id, created_at)
    SELECT NEW.workspace_id, NEW.object_key, NEW.id, datetime('now')
    WHERE NEW.archived_at IS NULL
    ON CONFLICT (record_id) DO NOTHING;
END;

CREATE TRIGGER trg_pending_embed_del_record AFTER DELETE ON custom_records
BEGIN
    DELETE FROM record_embeddings WHERE record_id = OLD.id;
    DELETE FROM pending_record_embeddings WHERE record_id = OLD.id;
END;

-- custom_field_values: same is_searchable-only guard record_search_fts's
-- own trigger uses (see that migration's doc comment on the same
-- known limitation: toggling is_searchable on/off doesn't retroactively
-- reindex until the next write). Only enqueues - the actual text is
-- recomputed fresh at drain time, not snapshotted here.
CREATE TRIGGER trg_pending_embed_ins_fieldval AFTER INSERT ON custom_field_values
WHEN (SELECT is_searchable FROM custom_field_definitions WHERE id = NEW.definition_id) = 1
BEGIN
    INSERT INTO pending_record_embeddings (workspace_id, object_key, record_id, created_at)
    SELECT r.workspace_id, r.object_key, r.id, datetime('now')
    FROM custom_records r WHERE r.id = NEW.entity_id AND r.archived_at IS NULL
    ON CONFLICT (record_id) DO NOTHING;
END;

CREATE TRIGGER trg_pending_embed_upd_fieldval AFTER UPDATE ON custom_field_values
WHEN (SELECT is_searchable FROM custom_field_definitions WHERE id = NEW.definition_id) = 1
   OR (SELECT is_searchable FROM custom_field_definitions WHERE id = OLD.definition_id) = 1
BEGIN
    INSERT INTO pending_record_embeddings (workspace_id, object_key, record_id, created_at)
    SELECT r.workspace_id, r.object_key, r.id, datetime('now')
    FROM custom_records r WHERE r.id = NEW.entity_id AND r.archived_at IS NULL
    ON CONFLICT (record_id) DO NOTHING;
END;

CREATE TRIGGER trg_pending_embed_del_fieldval AFTER DELETE ON custom_field_values
WHEN (SELECT is_searchable FROM custom_field_definitions WHERE id = OLD.definition_id) = 1
BEGIN
    INSERT INTO pending_record_embeddings (workspace_id, object_key, record_id, created_at)
    SELECT r.workspace_id, r.object_key, r.id, datetime('now')
    FROM custom_records r WHERE r.id = OLD.entity_id AND r.archived_at IS NULL
    ON CONFLICT (record_id) DO NOTHING;
END;
