-- Admin Automation & Customization addendum, Phase 1 (spec §2.2/§3.2):
-- "field-to-field comparison" - a condition's value can be resolved from
-- another field on the same record instead of only ever a fixed literal.
-- Both nullable: NULL on both means "compare to the literal `value` column"
-- (every existing row, and the common case going forward); non-NULL on
-- both means "look up this field in the trigger context and compare to
-- that instead" (domain::conditions::field_ref_is_valid validates it the
-- same way a condition's primary field_source/field_key already is).
--
-- No CHECK constraint on compare_field_source, matching every other
-- field_source-shaped column in this schema (migrations 0013/0014/0016) -
-- validated at the service layer instead.

ALTER TABLE business_rule_conditions ADD COLUMN compare_field_source TEXT;
ALTER TABLE business_rule_conditions ADD COLUMN compare_field_key TEXT;
ALTER TABLE workflow_conditions ADD COLUMN compare_field_source TEXT;
ALTER TABLE workflow_conditions ADD COLUMN compare_field_key TEXT;
