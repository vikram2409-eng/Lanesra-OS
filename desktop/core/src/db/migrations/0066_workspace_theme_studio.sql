-- UX/UI Modernization, Phase A (issue #191): Design Tokens & Theme Studio.
--
-- Nothing today lets an admin change --accent/brand colors, typography,
-- radius or density - OrganizationAdmin's branding section only handles a
-- logo upload. This is the first real token-storage/versioning surface:
-- a workspace keeps a history of theme versions, exactly one of which may
-- be 'published' (the one the running app actually renders), the rest
-- 'draft' or 'archived'. A Published version is an immutable snapshot of
-- exactly what rendered - the same Draft/Published/immutable-snapshot
-- discipline AI Agent Platform v2's agent versioning already established
-- - so "roll back" publishes a *new* version carrying an old version's
-- tokens rather than ever mutating history.
--
-- The 4 curated presets (Orbit/Slate/Ember/Aurora) are NOT rows here -
-- they're immutable Rust constants in theme_service.rs. A row only exists
-- once a workspace actually applies or customizes one; `preset_key`
-- records which preset a row started from (NULL for a fully custom
-- theme), purely informational.
CREATE TABLE workspace_themes (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    -- 'draft' | 'published' | 'archived'. A publish archives whatever was
    -- previously published for this workspace - never more than one
    -- published row at a time (enforced by the partial unique index
    -- below), mirroring agent_version_service's own publish-deprecates-
    -- prior-published rule.
    status TEXT NOT NULL,
    version INTEGER NOT NULL,
    preset_key TEXT,
    -- The full token document (color/typography/shape/density groups) -
    -- see theme_service::ThemeTokens for the typed shape this
    -- deserializes into. Kept as opaque JSON at the storage layer, same
    -- convention ai_agent_policies.blocked_tool_names_json already uses.
    tokens_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT,
    published_at TEXT,
    published_by TEXT
);
CREATE INDEX idx_workspace_themes_workspace ON workspace_themes (workspace_id, version DESC);
CREATE UNIQUE INDEX idx_workspace_themes_one_published ON workspace_themes (workspace_id) WHERE status = 'published';
