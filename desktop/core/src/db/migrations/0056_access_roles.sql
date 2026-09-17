-- Access Control v1 (spec: "Access Control v1" - Access Roles, capabilities,
-- Record Scopes, Access Inspector), the second of six Enterprise Access
-- Foundation phases. Phase 1 (0051-0055) shipped Organization, Organization
-- Units, Work Teams and record ownership fields but deliberately left
-- `ownership_service::require_assign_capability` as an Administrator-only
-- placeholder. This migration adds the real, additive capability+scope
-- engine that replaces it - alongside, not instead of, the legacy `roles`/
-- `user_roles` tables, which keep gating every other admin-only action
-- exactly as they do today. See access_service.rs for the evaluator this
-- schema exists to serve.
--
-- Design notes:
-- - A role's grants are per object_key, with an optional '*' row as the
--   default for any object_key that doesn't have its own row - a custom
--   object created after the role was defined still resolves to something
--   sensible with zero re-seeding, the same allowlist-with-fallback idea
--   `bulk_action_service.rs` already uses for its own per-object dispatch.
-- - Record Scope is a single ordered enum (OWNER < TEAM < ORG_UNIT_AND_BELOW
--   < ORGANIZATION), each level a strict superset of the one before it - a
--   grant names the broadest scope it allows, evaluation checks the record
--   against that one level directly (see access_service::scope_allows_record).
-- - Multiple Access Roles compose by union: a user's effective scope for a
--   capability is the broadest scope granted by ANY of their roles - the
--   same "fold to the strongest level" rule app_service.rs's own App
--   Builder permission grants already use.
CREATE TABLE access_roles (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_system INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    UNIQUE (workspace_id, name)
);
CREATE INDEX idx_access_roles_workspace ON access_roles(workspace_id);

CREATE TABLE access_role_grants (
    id TEXT PRIMARY KEY,
    access_role_id TEXT NOT NULL REFERENCES access_roles(id),
    object_key TEXT NOT NULL,
    can_create INTEGER NOT NULL DEFAULT 0,
    can_read INTEGER NOT NULL DEFAULT 0,
    can_update INTEGER NOT NULL DEFAULT 0,
    can_delete INTEGER NOT NULL DEFAULT 0,
    can_assign INTEGER NOT NULL DEFAULT 0,
    record_scope TEXT NOT NULL DEFAULT 'OWNER' CHECK (record_scope IN ('OWNER', 'TEAM', 'ORG_UNIT_AND_BELOW', 'ORGANIZATION')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (access_role_id, object_key)
);
CREATE INDEX idx_access_role_grants_role ON access_role_grants(access_role_id);

CREATE TABLE user_access_roles (
    user_id TEXT NOT NULL REFERENCES users(id),
    access_role_id TEXT NOT NULL REFERENCES access_roles(id),
    created_at TEXT NOT NULL,
    PRIMARY KEY (user_id, access_role_id)
);
CREATE INDEX idx_user_access_roles_user ON user_access_roles(user_id);

-- Bootstrap: every workspace gets two system Access Roles immediately, so
-- the engine is usable the moment this migration runs rather than leaving
-- every workspace with zero grants until an admin visits the new screen.
INSERT INTO access_roles (id, workspace_id, name, description, is_system, created_at, created_by, updated_at, updated_by)
SELECT lower(hex(randomblob(16))), id, 'Full Access', 'Every capability, organization-wide - the default for anyone who already holds the legacy Administrator role.', 1, created_at, NULL, created_at, NULL
FROM workspaces;

INSERT INTO access_roles (id, workspace_id, name, description, is_system, created_at, created_by, updated_at, updated_by)
SELECT lower(hex(randomblob(16))), id, 'Standard User', 'Ordinary create/read/update/delete, same as every workspace already behaves today - the one thing this role does not grant by default is Assign (reassigning a record''s owner), which stays scoped to whoever an admin explicitly grants it to.', 1, created_at, NULL, created_at, NULL
FROM workspaces;

INSERT INTO access_role_grants (id, access_role_id, object_key, can_create, can_read, can_update, can_delete, can_assign, record_scope, created_at, updated_at)
SELECT lower(hex(randomblob(16))), ar.id, '*', 1, 1, 1, 1, 1, 'ORGANIZATION', ar.created_at, ar.created_at
FROM access_roles ar WHERE ar.name = 'Full Access';

-- Deliberately Organization scope, not Owner: ordinary CRUD on business
-- objects has never been owner-restricted in this product (any
-- authenticated user could already read/update/delete any record, gated
-- only by App Builder's own optional per-app grants) - only Assign was ever
-- actually enforced (Administrator-only). Seeding Standard User at Owner
-- scope for create/read/update/delete would silently lock every existing
-- non-Administrator user out of records they don't personally own the
-- moment this migration runs. Assign is the one real, new restriction this
-- phase adds - an admin can tighten create/read/update/delete further by
-- editing this role or assigning a custom one, but the shipped default must
-- not regress today's behavior.
INSERT INTO access_role_grants (id, access_role_id, object_key, can_create, can_read, can_update, can_delete, can_assign, record_scope, created_at, updated_at)
SELECT lower(hex(randomblob(16))), ar.id, '*', 1, 1, 1, 1, 0, 'ORGANIZATION', ar.created_at, ar.created_at
FROM access_roles ar WHERE ar.name = 'Standard User';

-- Backfill: every existing user is assigned exactly one of the two system
-- roles, keyed off whether they hold the legacy Administrator role today -
-- so upgrading never leaves a user with zero Access Roles at all.
INSERT INTO user_access_roles (user_id, access_role_id, created_at)
SELECT u.id, ar.id, datetime('now')
FROM users u
JOIN access_roles ar ON ar.workspace_id = u.workspace_id AND ar.name = 'Full Access'
WHERE EXISTS (
    SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
    WHERE ur.user_id = u.id AND r.name = 'Administrator'
);

INSERT INTO user_access_roles (user_id, access_role_id, created_at)
SELECT u.id, ar.id, datetime('now')
FROM users u
JOIN access_roles ar ON ar.workspace_id = u.workspace_id AND ar.name = 'Standard User'
WHERE NOT EXISTS (
    SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
    WHERE ur.user_id = u.id AND r.name = 'Administrator'
);
