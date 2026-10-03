-- UX/UI Modernization, Screen Builder 2.0 (issue #195, 5b): "admins can
-- save a customized page as an Organization Template." The 4 built-in
-- templates (Executive 360, Operations Workspace, Clean Detail, Data &
-- Insights) are pure client-side data (pageTemplates.ts), the same way
-- Workflow Studio's and Business Rule Board's starter templates are - no
-- table needed for those, since "apply" just seeds a PageDefinition into
-- a page's own draft and the result is an ordinary page row from then on.
-- This table exists only for the org-authored case: a named, reusable
-- snapshot of one page's current draft, saved once and offered alongside
-- the 4 built-ins in the same "start from a template" picker. A template
-- row is an immutable snapshot, not a live/governed object - no
-- draft/published/roles/is_default lifecycle, unlike page_layouts.
CREATE TABLE page_templates (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  entity_type TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  definition_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  created_by TEXT
);

CREATE INDEX idx_page_templates_workspace_entity ON page_templates(workspace_id, entity_type);
