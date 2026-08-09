-- FR-WFL: admin-defined workflow automation, Phase 1 - "when an
-- Opportunity's stage (or an Invoice's status) transitions to X, create a
-- follow-up Task". Scoped to these two entities because they're the two
-- places a status/stage transition already carries real sales-process
-- meaning worth automating on (a deal is won, an invoice becomes overdue);
-- see "Workflow automation" in the product backlog for the fuller
-- brainstorm this Phase 1 slice was cut from.
--
-- The created Task is always linked back to the triggering record via
-- related_type/related_id (tasks.related_type/related_id, see 0001_init),
-- so it shows up on that Opportunity's or Invoice's own Tasks list with no
-- extra plumbing. assignee_user_id is nullable - null means "assign to the
-- record's owner" (Opportunity.owner_user_id, or the Invoice's Company's
-- owner_user_id, since invoices have no owner of their own - the same
-- attribution report_service::sales_by_owner already uses).

CREATE TABLE workflow_rules (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL CHECK (entity_type IN ('Opportunity', 'Invoice')),
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
CREATE INDEX idx_workflow_rules_lookup ON workflow_rules(workspace_id, entity_type, is_active);
