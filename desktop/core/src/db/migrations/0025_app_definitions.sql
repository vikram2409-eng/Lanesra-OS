-- App Builder Phase 1: group a set of already-existing objects, their
-- screens and a dashboard into one named, publishable application - the
-- packaging layer the platform story (see /platform on the public site)
-- has been proposed on top of Custom Objects/Relationships/Screen
-- Builder/Dashboards for a while. Every primitive an app assembles
-- (screen layouts, dashboards, custom objects) already exists and is
-- unchanged by this migration; app_definitions only stores which of them
-- belong together under one name/icon and who can see that grouping.
--
-- object_keys_json is a JSON array of entity_type strings (built-in, e.g.
-- "Task", or a custom object's key) - opaque to this layer, the same
-- "a frontend registry resolves it" choice screen_layouts' field keys and
-- dashboard_layouts' widget config already make. dashboard_id optionally
-- points at an existing dashboard_layouts row to use as this app's own
-- Dashboard section; NULL means the app has no dashboard of its own.
--
-- Unlike screen/dashboard layouts, an app's content isn't a draft/publish
-- fork - editing its object list or dashboard takes effect immediately,
-- the same as editing a Custom Object definition does. is_published is a
-- simple visibility gate: an unpublished app exists (so an admin can keep
-- building it) but never appears in anyone's app switcher, published or
-- not.
CREATE TABLE app_definitions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    icon TEXT NOT NULL DEFAULT '⬡',
    description TEXT,
    object_keys_json TEXT NOT NULL DEFAULT '[]',
    dashboard_id TEXT REFERENCES dashboard_layouts(id) ON DELETE SET NULL,
    is_published INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT
);
CREATE INDEX idx_app_definitions_workspace ON app_definitions(workspace_id);

-- Per-app access grants - the "new app-level permissions" App Builder was
-- explicitly scoped to have, rather than reusing the plain role-checkbox
-- pattern Screen/App Builder and Dashboards already use. A grant is either
-- to a role (principal_type='role', principal_id one of user_repo::ROLES)
-- or to one specific user (principal_type='user', principal_id a users.id)
-- - the per-user case is the actual new capability nothing else in the
-- product has: an individual can be given (or denied, by simply having no
-- grant) access to a specific app independent of their workspace role.
-- level is 'viewer' or 'editor'; see app_service::effective_access for how
-- multiple applicable grants resolve to one effective level, and its own
-- doc comment for what "editor" currently does and doesn't enforce yet.
CREATE TABLE app_permissions (
    id TEXT PRIMARY KEY,
    app_id TEXT NOT NULL REFERENCES app_definitions(id) ON DELETE CASCADE,
    principal_type TEXT NOT NULL CHECK (principal_type IN ('role', 'user')),
    principal_id TEXT NOT NULL,
    level TEXT NOT NULL CHECK (level IN ('viewer', 'editor')),
    created_at TEXT NOT NULL,
    created_by TEXT
);
CREATE INDEX idx_app_permissions_app ON app_permissions(app_id);
-- A given principal gets at most one grant per app - re-granting the same
-- role/user updates the level in place rather than accumulating rows a
-- resolver would have to pick the "best" of.
CREATE UNIQUE INDEX idx_app_permissions_unique_principal
    ON app_permissions(app_id, principal_type, principal_id);
