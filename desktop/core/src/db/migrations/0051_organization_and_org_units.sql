-- Enterprise Access Foundation, Phase 1 (spec: "Access Foundation v1" -
-- Organization, Organization Units, Work Teams, ownership modes, owner/
-- org-unit system fields). This migration adds the Organization and
-- Organization Unit layer.
--
-- Design note: the spec treats "Organization" as a distinct
-- platform/security entity ("must not be confused with Company/Customer
-- business records"). This codebase already has exactly that entity - the
-- `workspaces` table already carries business_name/legal_name/
-- currency_code/locale/timezone (see 0001_init.sql), and a workspace is
-- already the one-tenant-per-workspace boundary threaded through every
-- other table via workspace_id. Rather than create a parallel
-- `organizations` table that would duplicate those columns and need to be
-- kept in sync with `workspaces` forever, this migration extends
-- `workspaces` with only the fields it's actually missing
-- (org_code/org_status/root_org_unit_id) and `organization_service`
-- presents that combined shape as "Organization" - same underlying row,
-- spec-matching vocabulary at the service/API layer. If genuine
-- multi-organization-per-workspace support is ever needed, that's a real
-- schema change at that point, not something worth speculatively building
-- for now.
--
-- Unrelated naming note: `reference_packages.rs` already seeds a *demo*
-- custom object literally called "Organization Unit" as industry-pack
-- sample data for certain verticals (a workspace-instantiable record type
-- an admin can create/edit like any other custom object). That is
-- unrelated to `org_units` below, which is a core, non-custom,
-- security-scoping table every workspace gets automatically. The two are
-- never meant to be the same thing.
ALTER TABLE workspaces ADD COLUMN org_code TEXT;
ALTER TABLE workspaces ADD COLUMN org_status TEXT NOT NULL DEFAULT 'Active' CHECK (org_status IN ('Active', 'Inactive'));
ALTER TABLE workspaces ADD COLUMN root_org_unit_id TEXT;

-- Organization Units: hierarchical business divisions/regions/departments/
-- branches. `path`/`depth` are a materialized path (e.g. "/root-id/
-- child-id/"), not a closure table - a move only ever has to rewrite the
-- moved subtree's own path prefixes in one bounded UPDATE, no second
-- physical structure to keep consistent on every write. `LIKE 'prefix%'`
-- over the indexed path column is how a branch (a unit plus every
-- descendant) is resolved - see org_unit_service::preview_move/move_unit.
CREATE TABLE org_units (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    name TEXT NOT NULL,
    unit_type TEXT NOT NULL DEFAULT 'Custom' CHECK (unit_type IN ('Division', 'Region', 'Department', 'Branch', 'BusinessLine', 'Custom')),
    parent_org_unit_id TEXT REFERENCES org_units(id),
    manager_user_id TEXT REFERENCES users(id),
    status TEXT NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive')),
    effective_from TEXT,
    effective_to TEXT,
    default_team_id TEXT,
    path TEXT NOT NULL,
    depth INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_org_units_workspace ON org_units(workspace_id, status);
CREATE INDEX idx_org_units_parent ON org_units(parent_org_unit_id);
CREATE INDEX idx_org_units_path ON org_units(path);

-- Backfill: every existing workspace gets a real org_code and one root
-- Organization Unit named after its own business_name, so the invariant
-- "every workspace has an Organization and a root Organization Unit" holds
-- immediately, not just lazily the first time someone opens the new admin
-- screens. IDs generated the same way 0013_business_rules.sql's own
-- backfill already does (lower(hex(randomblob(16))) - a plain opaque TEXT
-- primary key, same as every other id in this schema, not required to be
-- RFC 4122-shaped since nothing validates that format).
UPDATE workspaces SET org_code = 'ORG-' || upper(hex(randomblob(4))) WHERE org_code IS NULL;
CREATE UNIQUE INDEX idx_workspaces_org_code ON workspaces(org_code) WHERE org_code IS NOT NULL;

INSERT INTO org_units (id, workspace_id, name, unit_type, parent_org_unit_id, manager_user_id, status, effective_from, effective_to, default_team_id, path, depth, created_at, created_by, updated_at, updated_by)
SELECT lower(hex(randomblob(16))), id, business_name, 'Division', NULL, NULL, 'Active', NULL, NULL, NULL, '/', 0, created_at, NULL, created_at, NULL
FROM workspaces;

UPDATE workspaces SET root_org_unit_id = (
    SELECT ou.id FROM org_units ou WHERE ou.workspace_id = workspaces.id AND ou.parent_org_unit_id IS NULL LIMIT 1
) WHERE root_org_unit_id IS NULL;

-- Root unit's own path becomes "/<its-id>/" once the id is known (can't be
-- computed before the INSERT above assigns it).
UPDATE org_units SET path = '/' || id || '/' WHERE parent_org_unit_id IS NULL AND path = '/';
