-- Solution Packages & Admin IA design spec, Phase 3: component-tagging -
-- migration 0029's own comment named this as the thing "still needs" to
-- happen before "what have I customized beyond what I installed" can
-- answer for hand-built work, not just installed packages. This is that.
--
-- package_artifacts (migration 0027) already answers "what did this one
-- install create", but it's scoped to installed_apps and only ever
-- populated by run_install - a Custom Object, Business Rule, Screen
-- Layout etc. an admin builds by hand through the ordinary Admin screens
-- has never had any owner record at all. solution_components is the
-- workspace-wide answer both cases share: every component, package-
-- installed or hand-built, tagged with the publisher that owns it.
--
-- Populated two ways (see solution_component_service):
--   - Every one of the 7 component-creating service functions
--     (custom_object/custom_field/relationship/business_rule/workflow/
--     screen_layout/dashboard_layout/custom_report create) tags itself
--     'local' unconditionally the moment it's created - the ordinary
--     admin-UI path never has any other context to tag it with.
--   - industry_package_service::run_install re-tags every artifact it
--     buffers (the same (artifact_type, metadata_id) pairs it writes to
--     package_artifacts) to the installing package's *resolved* publisher
--     right after those service calls return - overwriting the 'local'
--     tag those calls just wrote, since install reuses the exact same
--     service functions the admin UI does rather than a separate code
--     path. Net effect: every component ends up tagged with its real
--     owner regardless of which path created it, with no signature
--     changes needed on any of the 10 call sites this touches.
--
-- Deliberately narrower than package_artifacts: numbering_override and
-- custom_record are tracked there (for install/uninstall bookkeeping) but
-- not here - a numbering scheme tweak or a seeded sample record isn't a
-- "component" in the sense the Solution Packages spec means (a piece of
-- customizable, redistributable metadata), so tagging them would just add
-- noise to a workspace's Components view.
CREATE TABLE solution_components (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    artifact_type TEXT NOT NULL,
    metadata_id TEXT NOT NULL,
    publisher_id TEXT NOT NULL REFERENCES publishers(id) ON DELETE CASCADE,
    -- Set only when this component's current owner is a package install
    -- (retagged by run_install); NULL for anything still owned by 'local'
    -- hand-built work. Lets the Components view show "installed by
    -- Field Service v1.0" without a second join back through
    -- package_artifacts.
    installed_app_id TEXT REFERENCES installed_apps(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id)
);
-- One tag per component - re-tagging (the install retag step, or a
-- future re-run) updates this row in place rather than accumulating
-- history; solution_components answers "who owns this right now", not
-- an audit trail of ownership changes.
CREATE UNIQUE INDEX idx_solution_components_unique ON solution_components(workspace_id, artifact_type, metadata_id);
CREATE INDEX idx_solution_components_publisher ON solution_components(publisher_id);
CREATE INDEX idx_solution_components_installed_app ON solution_components(installed_app_id);
