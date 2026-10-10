-- Next-Gen AI Foundry & Low-Code Platform Enhancements, Domain A
-- (Intelligence Foundation), FND-02: the Semantic Metadata Layer.
--
-- Explicit business semantics above raw schema, so humans and AI use
-- consistent meaning instead of inferring it from a field's label. Two
-- new System Graph node types (`business_glossary_term`,
-- `metric_definition`, see migration 0073) reuse this layer's own
-- traceability: a glossary term's or metric's `system_edges` are derived
-- from the rows below, not a second lineage mechanism.
--
-- Wave 0 scope, per FND-02's own "Recommended delivery" note: this is the
-- *definition/metadata* layer only - `metric_definitions` records what a
-- metric means (its formula/source/grain/filters as a declarative
-- description), not a working computation engine. A real formula
-- evaluator is Domain D's Derived Metrics work (a later, separate phase)
-- consuming this metadata, not duplicating it.
CREATE TABLE business_glossary_terms (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    definition TEXT NOT NULL,
    owner_user_id TEXT REFERENCES users(id),
    -- JSON array of alternate names an agent or search might encounter.
    synonyms_json TEXT NOT NULL DEFAULT '[]',
    -- 'standard' | 'sensitive' | 'restricted' - the same vocabulary
    -- models::ai_memory::CLASSIFICATIONS already established for memory
    -- items; models::semantic re-declares the same 3 values locally
    -- rather than importing across modules for a 3-item constant.
    data_classification TEXT NOT NULL DEFAULT 'standard',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    UNIQUE (workspace_id, name)
);

-- An object/field's mapping to a glossary term's business meaning and/or
-- a fixed semantic role (Customer, Amount, Effective Date, ...) a
-- generated report or agent can key off instead of guessing from a
-- label. `field_key` NULL means the mapping describes the whole object,
-- not one field. At least one of `glossary_term_id`/`semantic_role` is
-- required - enforced in semantic_mapping_service, not a CHECK
-- constraint, since SQLite's column-null-based uniqueness makes a clean
-- single CHECK awkward here and the service already validates every
-- other cross-field rule this codebase has (e.g.
-- relationship_service's `target_is_polymorphic` handling).
CREATE TABLE semantic_mappings (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL,
    field_key TEXT,
    glossary_term_id TEXT REFERENCES business_glossary_terms(id) ON DELETE CASCADE,
    semantic_role TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT
);
CREATE INDEX idx_semantic_mappings_entity ON semantic_mappings (workspace_id, entity_type, field_key);
CREATE INDEX idx_semantic_mappings_term ON semantic_mappings (glossary_term_id);

CREATE TABLE metric_definitions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    source_entity_type TEXT NOT NULL,
    source_field_key TEXT,
    -- 'sum' | 'avg' | 'count' | 'count_distinct' | 'min' | 'max'.
    aggregation TEXT NOT NULL,
    -- Free-text grouping description (e.g. "monthly", "by region") - no
    -- real dimension/time-bucketing engine exists yet (Domain D's own
    -- later work); this column records the stated intent for traceability
    -- and documentation, not an executable grouping spec.
    grain TEXT,
    filters_json TEXT NOT NULL DEFAULT '{}',
    time_logic TEXT,
    owner_user_id TEXT REFERENCES users(id),
    glossary_term_id TEXT REFERENCES business_glossary_terms(id),
    version INTEGER NOT NULL DEFAULT 1,
    effective_start_date TEXT,
    effective_end_date TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    UNIQUE (workspace_id, key)
);

-- Same "append a JSON snapshot, prune to a cap" shape
-- business_rule_repo::insert_version/list_version_rows already
-- established (migration 0013's business_rule_versions) - reused instead
-- of a new temporal-versioning engine; `metric_definitions.version`
-- increments on every update and each snapshot captures it alongside the
-- formula/effective-date fields active at that time.
CREATE TABLE metric_versions (
    id TEXT PRIMARY KEY,
    metric_definition_id TEXT NOT NULL REFERENCES metric_definitions(id) ON DELETE CASCADE,
    snapshot_json TEXT NOT NULL,
    saved_at TEXT NOT NULL
);
CREATE INDEX idx_metric_versions_metric ON metric_versions (metric_definition_id);
