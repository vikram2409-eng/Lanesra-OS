-- Admin Automation & Customization addendum, Phase 4 (spec §4): four more
-- settings on a custom field definition, all optional/off so every
-- existing field keeps its current behavior unchanged after this
-- migration.
--
-- default_value: applied by custom_field_service::set_entity_values when
-- a save omits a value (or passes an empty one) for this field - the same
-- "only if currently empty" semantics business rules' own set_default
-- action already uses, just sourced from the field definition instead of
-- a rule, and layered underneath it (a business rule's set_default/
-- set_value can still override a field's own default).
--
-- is_unique: enforced at the same save-time validation point as
-- required/min/max/regex - rejects a save whose value already exists on
-- a different record of the same entity_type for this definition.
--
-- help_text / placeholder: presentation-only, rendered by the form but
-- never consulted server-side.
ALTER TABLE custom_field_definitions ADD COLUMN default_value TEXT;
ALTER TABLE custom_field_definitions ADD COLUMN is_unique INTEGER NOT NULL DEFAULT 0;
ALTER TABLE custom_field_definitions ADD COLUMN help_text TEXT;
ALTER TABLE custom_field_definitions ADD COLUMN placeholder TEXT;
