-- Next-Gen program, Ontology Layer epic, issue #343: Action Types - a
-- named, parameterized, governed write operation scoped to one Object
-- Type. No invocation-log table: every invoke() call writes one
-- audit_repo::record row (audit_events), the same place every other
-- mutation in this codebase is already audited, rather than a second log.

CREATE TABLE action_type_definitions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    target_object_type TEXT NOT NULL,
    required_capability TEXT NOT NULL,
    parameters_json TEXT NOT NULL DEFAULT '[]',
    effect TEXT NOT NULL,
    effect_config_json TEXT NOT NULL DEFAULT '{}',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    UNIQUE (workspace_id, target_object_type, name)
);
CREATE INDEX idx_action_type_definitions_workspace_object ON action_type_definitions (workspace_id, target_object_type);
