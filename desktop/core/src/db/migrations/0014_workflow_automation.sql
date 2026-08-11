-- Admin extensibility Phase D (spec §23/ADM-WF): replaces the original
-- single-trigger/single-action workflow_rules (status transition ->
-- create task only) with a richer Trigger -> Conditions -> Actions
-- engine - more trigger types, AND/OR conditions (reusing the same
-- domain::conditions matcher business_rule_conditions uses), and actions
-- beyond task creation. Existing workflow_rules rows are carried forward
-- automatically as an equivalent status_changed workflow with a
-- create_task action.
--
-- Also broadens task_links.related_type (migration 0001's CHECK
-- constraint only ever listed the seven original built-in entity types)
-- so a workflow's create_task action - and manual task creation - can
-- link a Task to a custom object record. Same "rebuild without the CHECK,
-- application layer is the real source of truth" pattern migration 0007
-- already used for custom_field_definitions/field_rules/workflow_rules.

PRAGMA foreign_keys=off;

CREATE TABLE task_links_new (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    related_type TEXT,
    related_id TEXT
);
INSERT INTO task_links_new (id, task_id, related_type, related_id)
    SELECT id, task_id, related_type, related_id FROM task_links;
DROP TABLE task_links;
ALTER TABLE task_links_new RENAME TO task_links;
CREATE INDEX idx_task_links_task ON task_links(task_id);
CREATE INDEX idx_task_links_related ON task_links(related_type, related_id);

PRAGMA foreign_keys=on;

CREATE TABLE workflow_definitions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    trigger_type TEXT NOT NULL CHECK (trigger_type IN ('record_created', 'record_updated', 'status_changed', 'field_changed', 'date_reached', 'due_overdue', 'scheduled')),
    trigger_status TEXT,
    trigger_field_key TEXT,
    trigger_offset_days INTEGER NOT NULL DEFAULT 0,
    match_type TEXT NOT NULL DEFAULT 'all' CHECK (match_type IN ('all', 'any')),
    priority INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    is_protected INTEGER NOT NULL DEFAULT 0,
    last_scheduled_run_at TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_workflow_definitions_lookup ON workflow_definitions(workspace_id, entity_type, is_active);

CREATE TABLE workflow_conditions (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL REFERENCES workflow_definitions(id) ON DELETE CASCADE,
    field_source TEXT NOT NULL CHECK (field_source IN ('builtin', 'custom')),
    field_key TEXT NOT NULL,
    operator TEXT NOT NULL,
    value TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_workflow_conditions_workflow ON workflow_conditions(workflow_id);

-- One row per action; `params_json` is a typed, serde-tagged payload
-- (see WorkflowActionParams) - action shapes differ too much (a task's
-- title/description/due date vs. a field's target/value vs. a
-- relationship+object for create_related_record) for a flat column set
-- without a wall of always-mostly-null columns, the same reasoning
-- custom_field_definitions' options_json already established for
-- single/multi-select choices.
CREATE TABLE workflow_actions (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL REFERENCES workflow_definitions(id) ON DELETE CASCADE,
    action_type TEXT NOT NULL,
    params_json TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_workflow_actions_workflow ON workflow_actions(workflow_id);

-- Execution log (ADM-WF-08) - also doubles as the fire-once dedup source
-- for date_reached/due_overdue triggers: before firing a date-based
-- workflow for a given record, the engine checks whether a run already
-- exists for that (workflow_id, entity_id) pair rather than maintaining a
-- second "already fired" table.
CREATE TABLE workflow_runs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    workflow_id TEXT NOT NULL REFERENCES workflow_definitions(id),
    entity_type TEXT NOT NULL,
    entity_id TEXT,
    trigger_type TEXT NOT NULL,
    triggered_at TEXT NOT NULL,
    outcome TEXT NOT NULL CHECK (outcome IN ('success', 'error', 'skipped')),
    actions_summary TEXT,
    error_message TEXT
);
CREATE INDEX idx_workflow_runs_workflow ON workflow_runs(workflow_id, entity_id);
CREATE INDEX idx_workflow_runs_workspace ON workflow_runs(workspace_id, triggered_at);

-- In-app notification center (spec §15/Platform Extensibility - "provide
-- an in-app notification center for locally generated workflow, approval
-- and reminder notifications"). recipient_user_id is nullable to support
-- a broadcast-to-all-Administrators notification (add_notification action
-- with audience = all_admins) without one row per admin.
CREATE TABLE notifications (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    recipient_user_id TEXT REFERENCES users(id),
    message TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    created_at TEXT NOT NULL,
    read_at TEXT
);
CREATE INDEX idx_notifications_recipient ON notifications(workspace_id, recipient_user_id, read_at);

INSERT INTO workflow_definitions (id, workspace_id, entity_type, name, description, trigger_type, trigger_status, trigger_field_key, trigger_offset_days, match_type, priority, is_active, created_at, created_by, updated_at, updated_by)
    SELECT id, workspace_id, entity_type,
           entity_type || ' -> ' || trigger_status,
           NULL, 'status_changed', trigger_status, NULL, 0, 'all', 0, is_active, created_at, created_by, updated_at, updated_by
    FROM workflow_rules;

INSERT INTO workflow_actions (id, workflow_id, action_type, params_json, sort_order)
    SELECT lower(hex(randomblob(16))), id, 'create_task',
           json_object('title', task_title, 'description', task_description, 'due_in_days', due_in_days, 'assignee_user_id', assignee_user_id),
           0
    FROM workflow_rules;

DROP TABLE workflow_rules;
