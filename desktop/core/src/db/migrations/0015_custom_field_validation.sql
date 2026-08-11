-- Admin extensibility Phase E (spec ADM-CF-04/ADM-CF-05): adds optional
-- validation (min/max for numbers, max length and regex for text) and the
-- searchable/filterable/reportable capability flags to custom field
-- definitions. All nullable/default-off so every existing field keeps its
-- current unrestricted behavior after this migration - min_value/max_value
-- are stored as TEXT (like every other value in this table) since SQLite
-- has no fixed-precision numeric column type; custom_field_service parses
-- them as f64 at validation time, the same convention field values
-- themselves already use.
--
-- is_reportable defaults to 1 (on) so every field already usable in the
-- report builder stays usable after upgrading; is_searchable/is_filterable
-- default to 0 since neither capability has a consuming feature yet (see
-- this migration's PR description) - they exist so a field's definition
-- doesn't need another migration once one does.

ALTER TABLE custom_field_definitions ADD COLUMN min_value TEXT;
ALTER TABLE custom_field_definitions ADD COLUMN max_value TEXT;
ALTER TABLE custom_field_definitions ADD COLUMN max_length INTEGER;
ALTER TABLE custom_field_definitions ADD COLUMN regex_pattern TEXT;
ALTER TABLE custom_field_definitions ADD COLUMN is_searchable INTEGER NOT NULL DEFAULT 0;
ALTER TABLE custom_field_definitions ADD COLUMN is_filterable INTEGER NOT NULL DEFAULT 0;
ALTER TABLE custom_field_definitions ADD COLUMN is_reportable INTEGER NOT NULL DEFAULT 1;
