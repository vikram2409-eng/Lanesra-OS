-- Solution Packages & Admin IA design spec, Phase 4: named, scoped
-- Solutions - the Dynamics-365-style "build a solution in test, export it,
-- import it in prod" workflow the user explicitly asked for, layered on
-- top of everything Phases 1-3 already shipped.
--
-- What's genuinely new here, and what isn't:
--   - "Environments" needed no new table at all: `workspaces` (migration
--     0001) already models exactly one standalone instance per running
--     app/server (see `workspace_repo::get` - "SELECT * FROM workspaces
--     LIMIT 1", a deliberate singleton). Two Lanesra OS instances - two
--     desktop installs, or two Team Workspace deployments - already ARE
--     two separate environments in the D365 sense, the same way two D365
--     orgs are. Nothing to build there; it's already true of the
--     architecture.
--   - The real gap was that `export_local_workspace` (Phase 3) is
--     all-or-nothing: it exports *everything* the `local` publisher owns,
--     with no way to name a deliberate subset. `solutions` +
--     `solution_members` is that missing curation layer - a Solution is a
--     named, versioned, admin-picked list of components (drawn from
--     `solution_components`, migration 0030) that can be exported on its
--     own via `industry_package_service::export_solution`, producing the
--     exact same `.lanesra`-shaped manifest the existing
--     import/validate/install pipeline already knows how to consume in a
--     second workspace - "prod". No new import/install machinery needed;
--     a Solution's export is just a differently-scoped input to the
--     pipeline Phase 0-3 already built and tested.
CREATE TABLE solutions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    -- Admin-controlled, not a timestamp (contrast
    -- `export_local_workspace`'s synthetic `1.0.<unix timestamp>`): a
    -- Solution's version is a deliberate release number the admin bumps
    -- each time they export a new snapshot to hand to prod, exactly like
    -- a D365 solution's version. Defaults to 1.0.0.0 on creation.
    version TEXT NOT NULL DEFAULT '1.0.0.0',
    -- Informational branding only - which registered publisher (Phase 2)
    -- this Solution is presented as belonging to. Always exported under
    -- the `local` namespace regardless (see export_solution's own
    -- comment), so this never gates anything at export/import time; it
    -- just labels the solution in the UI, same as a D365 solution's
    -- publisher does.
    publisher_id TEXT REFERENCES publishers(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE UNIQUE INDEX idx_solutions_workspace_name ON solutions(workspace_id, name);

-- Which components (artifact_type/metadata_id, the same identity pair
-- `solution_components` uses) an admin has explicitly curated into a
-- Solution. Deliberately not a foreign key onto solution_components
-- itself: a component can outlive or be deleted independently of any
-- Solution it was ever added to, and `export_solution` already has to
-- tolerate a membership row whose component no longer resolves (the same
-- forgiving pattern `export_local_workspace` uses for a dangling
-- relationship reference) rather than depend on referential integrity
-- across the polymorphic artifact_type/metadata_id pair, which SQLite
-- foreign keys can't express anyway.
CREATE TABLE solution_members (
    id TEXT PRIMARY KEY,
    solution_id TEXT NOT NULL REFERENCES solutions(id) ON DELETE CASCADE,
    artifact_type TEXT NOT NULL,
    metadata_id TEXT NOT NULL,
    added_at TEXT NOT NULL,
    added_by TEXT REFERENCES users(id)
);
CREATE UNIQUE INDEX idx_solution_members_unique ON solution_members(solution_id, artifact_type, metadata_id);
CREATE INDEX idx_solution_members_solution ON solution_members(solution_id);
