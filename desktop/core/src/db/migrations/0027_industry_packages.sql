-- Industry Data Model foundations (roadmap "Industry Data Model" / dev
-- spec "Top 10 Industry Data Models & Packaged Business Apps", section
-- 13.2 "Package registry tables"): the plumbing an installable industry
-- app package needs, built ahead of any real package content - every
-- table name below matches the spec's own registry table names exactly,
-- so the two stay traceable to each other.
--
-- Nothing here is industry-specific; this is purely "can a declarative
-- metadata package be imported, installed transactionally, tracked, and
-- deactivated" - the same three foundations called out on the roadmap
-- card (package manifest format, an InstalledApp ownership/versioning
-- registry, transactional install/rollback machinery). Reference apps
-- (Field Service, Property Management, ...) are manifests fed through
-- this machinery, not new schema.

-- The local package catalog: packages that have been imported (bundled
-- with the app, or uploaded as a .lanesra-app file per spec 13.6) and
-- validated, available to install. A workspace can hold more than one
-- version of the same package_id at once (e.g. to review an upgrade
-- before committing to it) - installed_apps.installed_version is what's
-- actually live. manifest_json is the full parsed
-- IndustryPackageManifest, kept verbatim so install can be re-run or
-- re-inspected without needing the original file again; checksum guards
-- against a tampered or corrupted re-import silently differing from what
-- was already validated.
CREATE TABLE app_packages (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    package_id TEXT NOT NULL,
    name TEXT NOT NULL,
    industry TEXT NOT NULL,
    version TEXT NOT NULL,
    min_lanesra_version TEXT NOT NULL,
    manifest_json TEXT NOT NULL,
    checksum TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'import',
    imported_at TEXT NOT NULL,
    imported_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_app_packages_workspace ON app_packages(workspace_id);
CREATE UNIQUE INDEX idx_app_packages_unique_version ON app_packages(workspace_id, package_id, version);

-- A package version's declared dependency on another package - spec
-- 13.2's own shape (package_id/dependency_package_id/version_constraint/
-- required flag). Kept as its own table (rather than only living inside
-- manifest_json) so dependency resolution can query it directly instead
-- of re-parsing JSON on every install attempt.
CREATE TABLE app_dependencies (
    id TEXT PRIMARY KEY,
    app_package_id TEXT NOT NULL REFERENCES app_packages(id) ON DELETE CASCADE,
    dependency_package_id TEXT NOT NULL,
    version_constraint TEXT NOT NULL,
    is_required INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX idx_app_dependencies_package ON app_dependencies(app_package_id);

-- One package actually installed into this workspace - at most one
-- active install per package_id (re-installing the same package_id is an
-- update, not a second row). app_definition_id optionally points at the
-- App Builder app (see migration 0025) this install created, so App
-- Switcher/nav-filtering/permissions machinery App Builder already ships
-- is reused as-is for "the app-level UX an installed industry package
-- needs" (spec 13.4) rather than building a second, parallel nav system.
-- status is 'active' | 'deactivated' - deactivating (the spec's default
-- removal behavior) never deletes the business records or metadata this
-- install created, only hides the app from navigation; see
-- package_artifacts for what a future destructive uninstall would need
-- to consider (out of scope for this foundation phase).
CREATE TABLE installed_apps (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    package_id TEXT NOT NULL,
    name TEXT NOT NULL,
    icon TEXT NOT NULL DEFAULT '⬡',
    industry TEXT NOT NULL,
    description TEXT,
    installed_version TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'deactivated')),
    app_definition_id TEXT REFERENCES app_definitions(id) ON DELETE SET NULL,
    -- Recommended role -> permission-level templates from the package's
    -- manifest (JSON array of {role, level}) - spec: "permissions:
    -- Recommended role templates, always reviewed by administrator
    -- before activation." Deliberately informational only: install never
    -- creates a real app_permissions grant from this - an administrator
    -- applies (or ignores) each recommendation manually from the
    -- existing Admin -> Apps permissions panel, the same review step the
    -- spec requires.
    recommended_permissions_json TEXT NOT NULL DEFAULT '[]',
    installed_at TEXT NOT NULL,
    installed_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    deactivated_at TEXT,
    deactivated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_installed_apps_workspace ON installed_apps(workspace_id);
CREATE UNIQUE INDEX idx_installed_apps_unique_package ON installed_apps(workspace_id, package_id);

-- The ownership ledger spec 13.2 calls "package_artifacts": every record
-- an install actually created, so a future update or uninstall knows
-- what it owns versus what an administrator added by hand afterward
-- (spec 1.1/13.3: "Local administrator customizations take precedence
-- over non-breaking package defaults; package updates must not silently
-- overwrite them"). artifact_type is one of the subsystems an install can
-- touch ('custom_object', 'custom_field', 'relationship_definition',
-- 'business_rule', 'workflow_definition', 'screen_layout',
-- 'dashboard_layout', 'custom_report', 'numbering_override',
-- 'custom_record'); metadata_id is that item's own primary key (or, for
-- numbering_override - which has no id of its own, see migration 0026 -
-- its entity_type, the table's actual key). origin_version is the
-- package version that created this exact artifact, so a later update
-- can tell "created by v1.0, never touched since" apart from "created by
-- v1.0, then edited after v1.1 changed it." is_locally_customized starts
-- false and is meant to flip true once an update flow exists that can
-- detect drift from the artifact's original snapshot - that diffing
-- logic is intentionally not built yet (see
-- industry_package_service's own module comment for what's deferred).
CREATE TABLE package_artifacts (
    id TEXT PRIMARY KEY,
    installed_app_id TEXT NOT NULL REFERENCES installed_apps(id) ON DELETE CASCADE,
    artifact_type TEXT NOT NULL,
    metadata_id TEXT NOT NULL,
    origin_version TEXT NOT NULL,
    is_locally_customized INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_package_artifacts_app ON package_artifacts(installed_app_id);

-- One attempt to install/update/deactivate/reactivate a package - spec
-- 13.2's "app_install_runs", kept even for failed attempts (unlike every
-- other table in this schema, a failed run's row is the point: it's the
-- audit trail of what was tried and why it didn't apply, backing spec
-- 13.5's "store reference on app_install_runs" for the pre-install
-- safety checkpoint). backup_snapshot_path is the on-disk `.lanesra`
-- backup taken immediately before this run's transaction opened (see
-- backup_service::create_backup) - belt-and-suspenders alongside the
-- transaction's own atomic rollback, since it also lets an administrator
-- restore to this exact pre-install point even after a run that
-- committed successfully but produced something they later want fully
-- undone.
CREATE TABLE app_install_runs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    package_id TEXT NOT NULL,
    package_version TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('install', 'update', 'deactivate', 'reactivate')),
    status TEXT NOT NULL CHECK (status IN ('running', 'succeeded', 'failed')),
    started_at TEXT NOT NULL,
    completed_at TEXT,
    backup_snapshot_path TEXT,
    error_message TEXT,
    actor_user_id TEXT REFERENCES users(id)
);
CREATE INDEX idx_app_install_runs_workspace ON app_install_runs(workspace_id);
