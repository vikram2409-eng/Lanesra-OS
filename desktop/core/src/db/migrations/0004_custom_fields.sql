-- FR-CFG: lets an Administrator add custom fields to Companies and
-- Contacts without a code change - an attribute side-table, not a schema
-- change per field, so filtering/sorting on built-in columns is
-- unaffected. Bounded to these two entities in Phase 1; extending to
-- others later reuses this same design with a new entity_type value.

CREATE TABLE custom_field_definitions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL CHECK (entity_type IN ('Company', 'Contact')),
    key TEXT NOT NULL,
    label TEXT NOT NULL,
    field_type TEXT NOT NULL CHECK (field_type IN ('text', 'number', 'date', 'boolean', 'select')),
    options_json TEXT,
    required INTEGER NOT NULL DEFAULT 0,
    show_in_list INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    UNIQUE (workspace_id, entity_type, key)
);
CREATE INDEX idx_custom_field_definitions_lookup ON custom_field_definitions(workspace_id, entity_type, is_active);

-- Every value stored as text; numeric/date/boolean validity is enforced
-- in custom_field_service, not a SQL CHECK, given SQLite's dynamic
-- typing and because the valid range depends on the definition's
-- field_type, which lives in a different row entirely.
CREATE TABLE custom_field_values (
    id TEXT PRIMARY KEY,
    definition_id TEXT NOT NULL REFERENCES custom_field_definitions(id),
    entity_id TEXT NOT NULL,
    value_text TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (definition_id, entity_id)
);
CREATE INDEX idx_custom_field_values_entity ON custom_field_values(entity_id);
