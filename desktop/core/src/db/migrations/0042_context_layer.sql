-- AI & Agentic Layer, Phase 7b: the Context Layer - real history for an
-- agent's persistent Memory (today's memory_md is overwritten wholesale
-- with no trail - see ai_agent_repo::update_memory), plus a SQLite
-- FTS5-backed search over Custom Object records so an agent can find
-- records by ranked free-text relevance instead of only exact-match
-- `list_records` filters or the unranked `LIKE` scan search_service's own
-- global_search already does. Deliberately no external vector database
-- or embeddings - ranking below is FTS5's built-in bm25() lexical
-- relevance, never embedding similarity; this table indexes the exact
-- same fields global_search's custom-object pass already scans
-- (custom_records.primary_name/display_number, plus any is_searchable
-- custom field's value_text) - a ranked evolution of that code path, not
-- a parallel system. Built-in entities (Company, Contact, ...) are out
-- of scope for this pass - named here, not silently absent - since they
-- don't share custom_records/custom_field_values; see
-- services::search_service's own new search_custom_records doc comment.

-- One snapshot every time an agent's memory_md changes, whichever path
-- changed it (the agent's own always-available update_memory tool, or an
-- admin's direct edit via ai_agent_service::set_memory) - the audit
-- trail ai_agent_repo::update_memory's plain UPDATE never had. changed_by
-- is 'agent' or a user id, the same "who/what changed this" sentinel-
-- string convention chat_conversations.agent_id/ai_token_usage already
-- use rather than a nullable actor column.
CREATE TABLE ai_agent_memory_history (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    memory_md TEXT NOT NULL,
    changed_by TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_ai_agent_memory_history_agent ON ai_agent_memory_history (agent_id, created_at);

-- Not contentless: stores the combined text itself rather than mapping
-- back to a separate content table by rowid, since a contentless index's
-- rowid bookkeeping across two source tables (custom_records AND
-- custom_field_values) is needless complexity at this scale - one row
-- per custom record, fully rebuilt (delete + insert) whenever the record
-- or any of its searchable field values changes, via the triggers below.
-- `title` is UNINDEXED and holds only `primary_name`, kept separate from
-- the searchable `text` blob purely so a query result can show a clean
-- display title without a join back to custom_records.
CREATE VIRTUAL TABLE record_search_fts USING fts5(
    workspace_id UNINDEXED,
    object_key UNINDEXED,
    record_id UNINDEXED,
    title UNINDEXED,
    text
);

-- One-time backfill for every existing, non-archived custom record.
INSERT INTO record_search_fts (workspace_id, object_key, record_id, title, text)
SELECT
    r.workspace_id,
    r.object_key,
    r.id,
    r.primary_name,
    r.primary_name || ' ' || r.display_number || ' ' || COALESCE((
        SELECT GROUP_CONCAT(v.value_text, ' ')
        FROM custom_field_values v
        JOIN custom_field_definitions d ON d.id = v.definition_id
        WHERE v.entity_id = r.id AND d.is_searchable = 1
    ), '')
FROM custom_records r
WHERE r.archived_at IS NULL;

-- custom_records: insert adds a row (a brand-new record has no field
-- values yet); update fully recomputes it (also covers archiving, since
-- archiving is an UPDATE setting archived_at - the WHERE guard below
-- means an archived record simply isn't reinserted, dropping it from
-- search); delete removes it.
CREATE TRIGGER trg_record_fts_ins_record AFTER INSERT ON custom_records
BEGIN
    INSERT INTO record_search_fts (workspace_id, object_key, record_id, title, text)
    VALUES (NEW.workspace_id, NEW.object_key, NEW.id, NEW.primary_name, NEW.primary_name || ' ' || NEW.display_number);
END;

CREATE TRIGGER trg_record_fts_upd_record AFTER UPDATE ON custom_records
BEGIN
    DELETE FROM record_search_fts WHERE record_id = OLD.id;
    INSERT INTO record_search_fts (workspace_id, object_key, record_id, title, text)
    SELECT NEW.workspace_id, NEW.object_key, NEW.id, NEW.primary_name,
        NEW.primary_name || ' ' || NEW.display_number || ' ' || COALESCE((
            SELECT GROUP_CONCAT(v.value_text, ' ')
            FROM custom_field_values v
            JOIN custom_field_definitions d ON d.id = v.definition_id
            WHERE v.entity_id = NEW.id AND d.is_searchable = 1
        ), '')
    WHERE NEW.archived_at IS NULL;
END;

CREATE TRIGGER trg_record_fts_del_record AFTER DELETE ON custom_records
BEGIN
    DELETE FROM record_search_fts WHERE record_id = OLD.id;
END;

-- custom_field_values: any insert/update/delete of a searchable field's
-- value fully recomputes its owning record's row the same way, keyed off
-- entity_id (which is the owning custom_records.id for a custom-object
-- field value). Guarded to only fire for is_searchable fields, so an
-- edit to a non-searchable field never touches the index. Known
-- limitation, named rather than silently absent: toggling a field
-- definition's is_searchable flag on/off does not retroactively reindex
-- already-written values - only a subsequent write to that record or
-- field picks up the new flag state.
CREATE TRIGGER trg_record_fts_ins_fieldval AFTER INSERT ON custom_field_values
WHEN (SELECT is_searchable FROM custom_field_definitions WHERE id = NEW.definition_id) = 1
BEGIN
    DELETE FROM record_search_fts WHERE record_id = NEW.entity_id;
    INSERT INTO record_search_fts (workspace_id, object_key, record_id, title, text)
    SELECT r.workspace_id, r.object_key, r.id, r.primary_name,
        r.primary_name || ' ' || r.display_number || ' ' || COALESCE((
            SELECT GROUP_CONCAT(v.value_text, ' ')
            FROM custom_field_values v
            JOIN custom_field_definitions d ON d.id = v.definition_id
            WHERE v.entity_id = r.id AND d.is_searchable = 1
        ), '')
    FROM custom_records r
    WHERE r.id = NEW.entity_id AND r.archived_at IS NULL;
END;

CREATE TRIGGER trg_record_fts_upd_fieldval AFTER UPDATE ON custom_field_values
WHEN (SELECT is_searchable FROM custom_field_definitions WHERE id = NEW.definition_id) = 1
   OR (SELECT is_searchable FROM custom_field_definitions WHERE id = OLD.definition_id) = 1
BEGIN
    DELETE FROM record_search_fts WHERE record_id = NEW.entity_id;
    INSERT INTO record_search_fts (workspace_id, object_key, record_id, title, text)
    SELECT r.workspace_id, r.object_key, r.id, r.primary_name,
        r.primary_name || ' ' || r.display_number || ' ' || COALESCE((
            SELECT GROUP_CONCAT(v.value_text, ' ')
            FROM custom_field_values v
            JOIN custom_field_definitions d ON d.id = v.definition_id
            WHERE v.entity_id = r.id AND d.is_searchable = 1
        ), '')
    FROM custom_records r
    WHERE r.id = NEW.entity_id AND r.archived_at IS NULL;
END;

CREATE TRIGGER trg_record_fts_del_fieldval AFTER DELETE ON custom_field_values
WHEN (SELECT is_searchable FROM custom_field_definitions WHERE id = OLD.definition_id) = 1
BEGIN
    DELETE FROM record_search_fts WHERE record_id = OLD.entity_id;
    INSERT INTO record_search_fts (workspace_id, object_key, record_id, title, text)
    SELECT r.workspace_id, r.object_key, r.id, r.primary_name,
        r.primary_name || ' ' || r.display_number || ' ' || COALESCE((
            SELECT GROUP_CONCAT(v.value_text, ' ')
            FROM custom_field_values v
            JOIN custom_field_definitions d ON d.id = v.definition_id
            WHERE v.entity_id = r.id AND d.is_searchable = 1
        ), '')
    FROM custom_records r
    WHERE r.id = OLD.entity_id AND r.archived_at IS NULL;
END;
