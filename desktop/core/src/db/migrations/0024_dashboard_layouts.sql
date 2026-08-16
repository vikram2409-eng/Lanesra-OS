-- Dashboard customization Phase 1: multiple named dashboard layouts per
-- workspace (not per-entity - a dashboard is workspace-level, unlike a
-- Screen layout), each an ordered list of widgets, assigned to zero or
-- more roles. Mirrors screen_layouts (migration 0023) in every structural
-- way: a layout is assigned to roles, exactly one per workspace is the
-- default (enforced by the partial unique index below) and is the
-- fallback for any role no other layout claims, and every edit lands on
-- draft_json first - only Publish copies it to published_json, which is
-- what the live Dashboard actually renders. NULL published_json (every
-- layout starts this way) means "no effect yet", the same as an
-- unpublished Screen layout.
--
-- draft_json/published_json hold the widget list as one JSON blob
-- ({"widgets":[{"id","kind","config":{...}}]}) rather than a normalized
-- widgets table - same rationale as screen_layouts' tabs_json: this layer
-- never queries into a widget's config, only stores/serves it whole. A
-- widget's `kind` determines how its `config` is shaped (Phase 1 ships
-- one kind, "kpi", whose config is just `{"kpi_key":"..."}` - the same
-- opaque KPI-key string workspaces.dashboard_kpi_prefs already stored,
-- now scoped per-layout instead of per-workspace); this layer doesn't
-- validate config shape, matching how it already treats field/relationship
-- keys in screen_layouts as opaque strings a frontend registry resolves.
CREATE TABLE dashboard_layouts (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
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
CREATE INDEX idx_dashboard_layouts_workspace ON dashboard_layouts(workspace_id);
CREATE UNIQUE INDEX idx_dashboard_layouts_one_default
    ON dashboard_layouts(workspace_id)
    WHERE is_default = 1;
