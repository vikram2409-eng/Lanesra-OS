-- Admin flexibility: custom fields, business rules and workflow automation
-- now apply to every major entity (Company, Contact, Opportunity, Quote,
-- Order, Invoice, Contract, Task, Product for custom fields/rules; the
-- same set minus Product, which has no status field, for workflow
-- automation) - not just the Phase 1 subsets from migrations 0004-0006.
--
-- The entity_type CHECK constraints those migrations wrote enumerated a
-- fixed, now-too-narrow allowlist. SQLite cannot ALTER a CHECK constraint
-- in place, so each table is rebuilt without one - the application layer
-- (CUSTOM_FIELD_ENTITY_TYPES / WORKFLOW_ENTITY_TYPES, and each service's
-- require_admin + validate_shape) is already the real source of truth for
-- which entity types are valid, exactly like trigger_field_key/
-- target_field_key were never foreign-keyed either.

PRAGMA foreign_keys=off;

CREATE TABLE custom_field_definitions_new (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL,
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
INSERT INTO custom_field_definitions_new
    (id, workspace_id, entity_type, key, label, field_type, options_json, required, show_in_list, sort_order, is_active, created_at, created_by, updated_at, updated_by)
    SELECT id, workspace_id, entity_type, key, label, field_type, options_json, required, show_in_list, sort_order, is_active, created_at, created_by, updated_at, updated_by
    FROM custom_field_definitions;
DROP TABLE custom_field_definitions;
ALTER TABLE custom_field_definitions_new RENAME TO custom_field_definitions;
CREATE INDEX idx_custom_field_definitions_lookup ON custom_field_definitions(workspace_id, entity_type, is_active);

CREATE TABLE field_rules_new (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL,
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
INSERT INTO field_rules_new
    (id, workspace_id, entity_type, trigger_field_source, trigger_field_key, operator, trigger_value, target_field_key, effect, is_active, sort_order, created_at, created_by, updated_at, updated_by)
    SELECT id, workspace_id, entity_type, trigger_field_source, trigger_field_key, operator, trigger_value, target_field_key, effect, is_active, sort_order, created_at, created_by, updated_at, updated_by
    FROM field_rules;
DROP TABLE field_rules;
ALTER TABLE field_rules_new RENAME TO field_rules;
CREATE INDEX idx_field_rules_lookup ON field_rules(workspace_id, entity_type, is_active);

CREATE TABLE workflow_rules_new (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL,
    trigger_status TEXT NOT NULL,
    task_title TEXT NOT NULL,
    task_description TEXT,
    due_in_days INTEGER NOT NULL DEFAULT 0,
    assignee_user_id TEXT REFERENCES users(id),
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
INSERT INTO workflow_rules_new
    (id, workspace_id, entity_type, trigger_status, task_title, task_description, due_in_days, assignee_user_id, is_active, created_at, created_by, updated_at, updated_by)
    SELECT id, workspace_id, entity_type, trigger_status, task_title, task_description, due_in_days, assignee_user_id, is_active, created_at, created_by, updated_at, updated_by
    FROM workflow_rules;
DROP TABLE workflow_rules;
ALTER TABLE workflow_rules_new RENAME TO workflow_rules;
CREATE INDEX idx_workflow_rules_lookup ON workflow_rules(workspace_id, entity_type, is_active);

PRAGMA foreign_keys=on;
