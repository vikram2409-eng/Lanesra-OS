-- Admin flexibility: a simple report builder - an Administrator picks an
-- entity type, a field to group by (the entity's built-in status/stage, or
-- any active custom field on that entity), and an aggregate (count of
-- records, or sum of a numeric custom field) per group. Deliberately not a
-- full drag-and-drop builder - see "Report builder" in the product
-- backlog for the fuller version this was scoped down from.

CREATE TABLE custom_reports (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    name TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    group_by_source TEXT NOT NULL CHECK (group_by_source IN ('builtin', 'custom')),
    group_by_field TEXT NOT NULL,
    aggregate TEXT NOT NULL CHECK (aggregate IN ('count', 'sum')),
    sum_field_key TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_custom_reports_workspace ON custom_reports(workspace_id);
