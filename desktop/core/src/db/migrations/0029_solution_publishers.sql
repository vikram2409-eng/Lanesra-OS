-- Solution Packages & Admin IA design spec, Phase 2: a real Publisher
-- entity and the publisher/namespace scope the user asked to "build it
-- now" (not stub) rather than treat every package as implicitly owned by
-- Lanesra. Every app_packages.package_id has always been a namespaced
-- string by convention (e.g. "lanesra.field_service") - this table is
-- what finally makes that namespace a real, registered thing rather than
-- just a string prefix nobody validates.
--
-- Every workspace gets two publishers seeded automatically (see
-- publisher_service::ensure_defaults, called both at first-run setup and
-- lazily wherever publishers are read, so a workspace created before this
-- migration self-heals the first time it touches this feature - no data
-- migration needed):
--   'lanesra' (is_official) - owns every bundled reference package
--     (Field Service, Property Management, ...), so those keep importing
--     out of the box with no admin setup step.
--   'local'   (is_local) - the implicit home for whatever an admin builds
--     by hand (Custom Objects, Business Rules, ...) rather than installs
--     from a package. Nothing writes package_artifacts rows against it
--     yet in this phase - see industry_package_service's own module
--     comment for what "component-tagging" (attributing hand-built
--     things to this publisher) still needs, deferred to a later phase.
-- An admin can register further publishers of their own (Admin ->
-- Deployment Management -> Publishers) once they want to package and
-- namespace their own customizations - export itself is still future
-- scope, but the registry needs to exist before that can.
CREATE TABLE publishers (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    -- The namespace prefix every package_id under this publisher must
    -- start with (e.g. key 'acme' -> package_ids like 'acme.inspection').
    -- Lowercase ascii letters/digits/underscore, validated by
    -- publisher_service::validate_key, not just this column's type.
    key TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    is_official INTEGER NOT NULL DEFAULT 0,
    is_local INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE UNIQUE INDEX idx_publishers_workspace_key ON publishers(workspace_id, key);

-- Which publisher declared a package, and whether it's Packaged (what the
-- `is_managed` column still spells the original Managed/Unmanaged way -
-- Deployment Management's user-facing terms are Packaged/Custom, but the
-- column itself predates that rename and isn't renamed retroactively) -
-- every package imported through Admin -> App Catalog is Packaged by
-- definition in this phase (a versioned release from a registered
-- publisher, installed through the transactional pipeline); Custom
-- packages (an admin's own local customizations, wrapped as a package)
-- are still future scope - is_managed exists now so that distinction has
-- a column ready rather than needing a third migration once it ships.
-- publisher_id is nullable (ON DELETE SET NULL, not CASCADE, matching
-- installed_apps.app_definition_id's own reasoning) so a hypothetically
-- deleted publisher doesn't cascade-delete the packages it published.
ALTER TABLE app_packages ADD COLUMN publisher_id TEXT REFERENCES publishers(id) ON DELETE SET NULL;
ALTER TABLE app_packages ADD COLUMN is_managed INTEGER NOT NULL DEFAULT 1;
CREATE INDEX idx_app_packages_publisher ON app_packages(publisher_id);
