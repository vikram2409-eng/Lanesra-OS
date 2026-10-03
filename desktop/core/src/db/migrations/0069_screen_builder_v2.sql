-- UX/UI Modernization, Screen Builder 2.0 (issue #195, 5a): a new, richer
-- page-composition layer alongside the existing `screen_layouts` table
-- (Screen/App Builder Phase 1-3, migration 0023) - deliberately a second
-- table, not a migration of the old one in place. `screen_layouts` stores
-- a fixed tabs -> sections -> columns -> fields tree; this issue's spec
-- asks for an arbitrary tree of ~24 typed components (Layout/Record/Data/
-- Actions/Content/Navigation/Utility) each independently placed on a
-- 12-column responsive grid - a materially different shape, not an
-- evolution of the old one. Every existing saved screen_layouts row and
-- its two renderers (LayoutFormFields/LayoutDetailFields) are completely
-- untouched by this migration; `page_layouts` is the new page-composer's
-- own storage, wired into the actual record-detail render path only in
-- 5b.
--
-- Same governance shape as screen_layouts on purpose (draft/published
-- blobs, is_default, roles_json) - a page_layouts row's draft/published
-- JSON holds a `PageDefinition` tree (see page_layout.rs) instead of a
-- `LayoutTabs` tree, but the surrounding draft/publish/revert/default/
-- role-resolution lifecycle is identical, so page_layout_service mirrors
-- screen_layout_service function-for-function.
CREATE TABLE page_layouts (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  entity_type TEXT NOT NULL,
  name TEXT NOT NULL,
  is_default INTEGER NOT NULL DEFAULT 0,
  roles_json TEXT NOT NULL DEFAULT '[]',
  draft_json TEXT NOT NULL,
  published_json TEXT,
  created_at TEXT NOT NULL,
  created_by TEXT,
  updated_at TEXT NOT NULL,
  updated_by TEXT
);

CREATE INDEX idx_page_layouts_workspace_entity ON page_layouts(workspace_id, entity_type);
CREATE UNIQUE INDEX idx_page_layouts_one_default ON page_layouts(workspace_id, entity_type) WHERE is_default = 1;
