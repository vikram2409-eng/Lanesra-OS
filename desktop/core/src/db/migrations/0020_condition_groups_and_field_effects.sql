-- Admin Automation & Customization, second addendum round: nested OR-group
-- conditions, a wider field-effect/action palette, and a per-field
-- "hidden by default" flag.
--
-- group_id: NULL means the condition stands on its own and participates
-- directly in the rule/workflow's top-level match_type (AND when 'all', OR
-- when 'any'). Conditions sharing the same non-NULL group_id are first
-- OR'd together into one sub-unit, and that sub-unit then participates in
-- the top-level match_type alongside the ungrouped conditions - one level
-- of nesting, matching the rule-builder's "+ Add condition" (top-level) vs.
-- "+ OR group" (nested OR cluster) affordances. See
-- domain::conditions::conditions_match for the matcher. Every existing
-- condition gets group_id = NULL, so no rule's behavior changes until an
-- admin explicitly creates a group.
ALTER TABLE business_rule_conditions ADD COLUMN group_id TEXT;
ALTER TABLE workflow_conditions ADD COLUMN group_id TEXT;

-- is_hidden_by_default: a custom field flagged this way is left off every
-- create/edit form unless a business rule's "show" action targets it and
-- its condition currently matches - the counterpart to a business rule's
-- "hide" action, for fields that should start invisible rather than start
-- visible and only sometimes get hidden. Off by default so every existing
-- field keeps rendering exactly as it already does.
ALTER TABLE custom_field_definitions ADD COLUMN is_hidden_by_default INTEGER NOT NULL DEFAULT 0;
