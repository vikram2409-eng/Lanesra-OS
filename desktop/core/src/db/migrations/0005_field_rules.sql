-- FR-RUL: admin-defined conditional rules over custom fields - "require
-- Industry when Status = Prospect". Scoped to custom fields only (target
-- must be a custom field), and trigger may be the entity's built-in
-- status field or another custom field - deliberately not built-in fields
-- in general, since those already have hardcoded server-side validation
-- elsewhere that a rule table would need to be taught to consult too.
--
-- Keys are stored as plain text, not foreign keys to
-- custom_field_definitions, matching how the definitions table's own
-- (workspace_id, entity_type, key) is already the real identity - this
-- keeps a rule's target/trigger meaningful even if referenced only by key.

CREATE TABLE field_rules (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL CHECK (entity_type IN ('Company', 'Contact')),
    trigger_field_source TEXT NOT NULL CHECK (trigger_field_source IN ('builtin', 'custom')),
    trigger_field_key TEXT NOT NULL,
    operator TEXT NOT NULL CHECK (operator IN ('equals', 'not_equals')),
    trigger_value TEXT NOT NULL,
    target_field_key TEXT NOT NULL,
    effect TEXT NOT NULL CHECK (effect IN ('require', 'hide')),
    is_active INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_field_rules_lookup ON field_rules(workspace_id, entity_type, is_active);
