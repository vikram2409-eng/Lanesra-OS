-- Per-app scoped automation: business rules, workflows, and dashboard
-- layouts can each optionally be tagged with the App Builder app
-- (migration 0025's app_definitions) they belong to, instead of always
-- being workspace-wide.
--
-- This is deliberately the smaller of two features considered for "each
-- app should have its own workflows/business rules/dashboards" (a real
-- user request) - true per-app multi-tenant isolation, with separate
-- users/roles/business profile per app, would be a much larger
-- architectural change on par with the Industry Data Model epic. Users,
-- roles, and the workspace business profile stay shared workspace-wide
-- here; access to an app is already handled by app_permissions'
-- Viewer/Editor grants (migration 0025).
--
-- app_id is nullable and defaults to NULL on every existing row: NULL
-- means "workspace-wide", exactly the behavior every business rule,
-- workflow, and dashboard layout has always had, so this ships with zero
-- migration risk to existing data. Setting app_id only changes which
-- app's Admin screens *show* the rule/workflow/dashboard by default (see
-- the frontend App scope selector/filter) - it does not change whether
-- the rule/workflow evaluates or the dashboard resolves; those keep
-- running exactly as they do today regardless of app_id.
--
-- ON DELETE SET NULL (not CASCADE): deleting the app a rule/workflow/
-- dashboard was tagged with falls it back to workspace-wide rather than
-- deleting automation that may still be doing real work - consistent with
-- app_definitions.dashboard_id's own ON DELETE SET NULL for the same
-- reason.
ALTER TABLE business_rules ADD COLUMN app_id TEXT REFERENCES app_definitions(id) ON DELETE SET NULL;
CREATE INDEX idx_business_rules_app ON business_rules(app_id);

ALTER TABLE workflow_definitions ADD COLUMN app_id TEXT REFERENCES app_definitions(id) ON DELETE SET NULL;
CREATE INDEX idx_workflow_definitions_app ON workflow_definitions(app_id);

ALTER TABLE dashboard_layouts ADD COLUMN app_id TEXT REFERENCES app_definitions(id) ON DELETE SET NULL;
CREATE INDEX idx_dashboard_layouts_app ON dashboard_layouts(app_id);
