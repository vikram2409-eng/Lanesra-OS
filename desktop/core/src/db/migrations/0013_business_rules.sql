-- Admin extensibility Phase C (spec §22/ADM-BR): replaces the single-
-- condition/single-effect field_rules engine (migrations 0005/0007) with a
-- richer IF (AND/OR) / THEN rule builder - multiple conditions per rule,
-- more operators, and actions beyond require/hide (lock, set default/set
-- value, block the whole save with a custom message, show a non-blocking
-- message). Existing field_rules rows are carried forward as one rule with
-- one condition and one action each, so nothing an admin already
-- configured is lost.
--
-- Like every entity_type column in this product since migration 0007,
-- business_rules.entity_type is plain TEXT, validated at the service layer
-- against "any built-in type or active custom object key" - the same
-- entity_registry vocabulary relationship_definitions now shares.

CREATE TABLE business_rules (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    match_type TEXT NOT NULL DEFAULT 'all' CHECK (match_type IN ('all', 'any')),
    priority INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    effective_start_date TEXT,
    effective_end_date TEXT,
    is_protected INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_business_rules_lookup ON business_rules(workspace_id, entity_type, is_active, priority);

CREATE TABLE business_rule_conditions (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL REFERENCES business_rules(id) ON DELETE CASCADE,
    field_source TEXT NOT NULL CHECK (field_source IN ('builtin', 'custom')),
    field_key TEXT NOT NULL,
    operator TEXT NOT NULL,
    value TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_business_rule_conditions_rule ON business_rule_conditions(rule_id);

CREATE TABLE business_rule_actions (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL REFERENCES business_rules(id) ON DELETE CASCADE,
    action_type TEXT NOT NULL,
    target_field_key TEXT,
    action_value TEXT,
    message TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_business_rule_actions_rule ON business_rule_actions(rule_id);

-- Carry forward every existing field_rules row as an equivalent one-
-- condition/one-action business rule, preserving is_active/sort_order and
-- audit columns, then retire the old table - the same "rebuild and copy"
-- approach migration 0007 used, just across three new tables instead of
-- one rebuilt one.
INSERT INTO business_rules (id, workspace_id, entity_type, name, description, match_type, priority, is_active, created_at, created_by, updated_at, updated_by)
    SELECT id, workspace_id, entity_type,
           'When ' || trigger_field_key || ' ' || operator || ' "' || trigger_value || '"',
           NULL, 'all', sort_order, is_active, created_at, created_by, updated_at, updated_by
    FROM field_rules;

INSERT INTO business_rule_conditions (id, rule_id, field_source, field_key, operator, value, sort_order)
    SELECT lower(hex(randomblob(16))), id, trigger_field_source, trigger_field_key, operator, trigger_value, 0
    FROM field_rules;

INSERT INTO business_rule_actions (id, rule_id, action_type, target_field_key, action_value, message, sort_order)
    SELECT lower(hex(randomblob(16))), id, effect, target_field_key, NULL, NULL, 0
    FROM field_rules;

DROP TABLE field_rules;
