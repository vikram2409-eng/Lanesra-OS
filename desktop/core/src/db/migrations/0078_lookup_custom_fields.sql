-- Lookup Custom Field type: a Custom Relationship (many_to_one or
-- one_to_one, non-polymorphic) now auto-creates a pickable "lookup"
-- field on the appropriate object(s) instead of only ever surfacing as
-- a post-creation "Related list" - see relationship_service's own
-- auto-creation/cascade-deactivation logic and
-- custom_field_service::create_lookup_field.
--
-- Both new columns are NULL for every field created before this migration
-- and for every ordinary admin-created field afterward (text/number/
-- date/boolean/select) - only meaningful when field_type = 'lookup',
-- and only ever set by custom_field_service::create_lookup_field, never
-- through the ordinary admin create/edit form.
--
-- The field_type CHECK constraint migration 0007 left in place
-- (field_type IN ('text', 'number', 'date', 'boolean', 'select')) can't be
-- widened with an ALTER - SQLite doesn't support altering a CHECK in
-- place - so this rebuilds the table the same way 0007 rebuilt it for
-- entity_type: dropping the CHECK entirely rather than re-enumerating it,
-- since CUSTOM_FIELD_TYPES is already the real source of truth for valid
-- field types (0007's own comment makes the identical argument for
-- entity_type).

PRAGMA foreign_keys=off;

CREATE TABLE custom_field_definitions_new (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL,
    key TEXT NOT NULL,
    label TEXT NOT NULL,
    field_type TEXT NOT NULL,
    options_json TEXT,
    required INTEGER NOT NULL DEFAULT 0,
    show_in_list INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    min_value TEXT,
    max_value TEXT,
    max_length INTEGER,
    regex_pattern TEXT,
    is_searchable INTEGER NOT NULL DEFAULT 0,
    is_filterable INTEGER NOT NULL DEFAULT 0,
    is_reportable INTEGER NOT NULL DEFAULT 1,
    default_value TEXT,
    is_unique INTEGER NOT NULL DEFAULT 0,
    help_text TEXT,
    placeholder TEXT,
    is_hidden_by_default INTEGER NOT NULL DEFAULT 0,
    lookup_target_entity_type TEXT,
    lookup_relationship_definition_id TEXT REFERENCES relationship_definitions(id),
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    UNIQUE (workspace_id, entity_type, key)
);
INSERT INTO custom_field_definitions_new
    (id, workspace_id, entity_type, key, label, field_type, options_json, required, show_in_list, sort_order, is_active,
     min_value, max_value, max_length, regex_pattern, is_searchable, is_filterable, is_reportable,
     default_value, is_unique, help_text, placeholder, is_hidden_by_default, created_at, created_by, updated_at, updated_by)
    SELECT id, workspace_id, entity_type, key, label, field_type, options_json, required, show_in_list, sort_order, is_active,
           min_value, max_value, max_length, regex_pattern, is_searchable, is_filterable, is_reportable,
           default_value, is_unique, help_text, placeholder, is_hidden_by_default, created_at, created_by, updated_at, updated_by
    FROM custom_field_definitions;
DROP TABLE custom_field_definitions;
-- Unlike migration 0007's rebuild, triggers already exist by this point
-- (migration 0042's record_search_fts triggers) whose bodies reference
-- "custom_field_definitions" by name. SQLite's default ALTER TABLE RENAME
-- re-validates every trigger that mentions the destination name as part
-- of the rename itself, and fails because the table doesn't exist again
-- yet at that instant (it's what's being renamed into place) -
-- legacy_alter_table makes the rename a pure textual operation with no
-- such re-validation, matching every other table rename in this codebase
-- that predates those triggers.
PRAGMA legacy_alter_table=ON;
ALTER TABLE custom_field_definitions_new RENAME TO custom_field_definitions;
PRAGMA legacy_alter_table=OFF;
CREATE INDEX idx_custom_field_definitions_lookup ON custom_field_definitions(workspace_id, entity_type, is_active);
CREATE INDEX idx_custom_field_definitions_lookup_rel ON custom_field_definitions(lookup_relationship_definition_id);

PRAGMA foreign_keys=on;
