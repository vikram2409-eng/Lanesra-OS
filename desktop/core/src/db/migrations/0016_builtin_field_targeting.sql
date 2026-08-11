-- ADM-BR/ADM-WF "any field" targeting: business rule conditions and
-- workflow conditions already had a `field_source` ('builtin'|'custom')
-- column from migrations 0013/0014, so no schema change is needed there -
-- only the service-layer allowlist of which builtin field_key values are
-- valid per entity_type is being relaxed (see domain::builtin_fields).
--
-- Actions never had that distinction: `target_field_key` on
-- business_rule_actions was always assumed to be a custom field. This adds
-- the same field_source concept there, defaulting every existing row to
-- 'custom' (the only kind that existed before this migration), so a
-- require/hide/lock/set_default/set_value action can now target a built-in
-- field too. Plain TEXT with no CHECK, validated at the service layer -
-- same convention entity_type columns use throughout this schema (see
-- migration 0013's own header comment).
--
-- Workflow's equivalent `update_field` action has no table column to add a
-- source to - its target is inside `workflow_actions.params_json`, an
-- opaque JSON blob the service layer already parses per action_type - so it
-- gets a `"field_source"` JSON key instead, defaulting to "custom" when
-- absent for the same backward-compatibility reason.
--
-- workflow_definitions.trigger_field_key had the same "always assumed
-- custom" gap for the field_changed trigger specifically (date_reached/
-- due_overdue already only ever name a curated built-in date field, so
-- they don't need this). Same fix, same default.

ALTER TABLE business_rule_actions ADD COLUMN target_field_source TEXT NOT NULL DEFAULT 'custom';
ALTER TABLE workflow_definitions ADD COLUMN trigger_field_source TEXT NOT NULL DEFAULT 'custom';
