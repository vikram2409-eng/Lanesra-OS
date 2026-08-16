-- Screen/App Builder Phase 1: an object (a built-in entity_type name or a
-- custom object's key) can have several named layouts, not just one -
-- each with its own tabs of admin-drag-ordered field sections. A layout
-- is assigned to zero or more roles (screen_layout_service resolves the
-- signed-in user's roles against every layout's roles_json to pick the
-- one they see); exactly one layout per (workspace_id, entity_type) is
-- the default, enforced by the partial unique index below, and is the
-- fallback for any role no other layout claims.
--
-- draft_json/published_json each hold the layout's tabs structure as one
-- JSON blob ({"tabs":[{"id","title","sections":[{"id","title","fields":
-- [...]}]}]}) rather than a normalized tabs/sections/fields schema - this
-- layer never queries the structure, only reads/writes it whole and
-- serves it to the frontend to render, the same choice migration 0022
-- made for rule version snapshots. A field is referenced only by its key
-- string - this table has no idea what a field is or whether it still
-- exists, matching how the online demo's equivalent layout gracefully
-- treats an unrecognized/stale key (filtered at render time, never a
-- storage-layer concern). published_json is NULL until first published.
CREATE TABLE screen_layouts (
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
CREATE INDEX idx_screen_layouts_workspace_entity ON screen_layouts(workspace_id, entity_type);
CREATE UNIQUE INDEX idx_screen_layouts_one_default
    ON screen_layouts(workspace_id, entity_type)
    WHERE is_default = 1;
