-- Admin extensibility (spec §20.2): lets an Administrator define a whole
-- new business object at runtime, not just fields on an existing one. A
-- custom object's records reuse the same custom_field_values/field_rules/
-- workflow_rules machinery every built-in entity already uses - entity_type
-- there is just a string, and a custom object's `key` (a lowercase slug,
-- guaranteed distinct from the PascalCase built-in type names like
-- "Company") IS that entity_type value. Nothing about those three
-- subsystems needs a schema change to support this; only the service-layer
-- validation that previously checked entity_type against a fixed Rust list
-- also needs to accept "any active custom object key for this workspace" -
-- see custom_object_service.

CREATE TABLE custom_object_definitions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    key TEXT NOT NULL,
    singular_label TEXT NOT NULL,
    plural_label TEXT NOT NULL,
    icon TEXT NOT NULL DEFAULT '◆',
    prefix TEXT NOT NULL,
    digits INTEGER NOT NULL DEFAULT 6,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    UNIQUE (workspace_id, key)
);
CREATE INDEX idx_custom_object_definitions_workspace ON custom_object_definitions(workspace_id, is_active);

-- One row per record of a custom object - the generic equivalent of the
-- `companies`/`contacts`/etc tables for object types that don't get their
-- own dedicated table. `object_key` matches custom_object_definitions.key
-- (joined by key, not id, since custom-field/rule/workflow code already
-- keys everything off the entity_type string rather than a definition id).
-- Status is a fixed three-value set for MVP (ADM-CO custom status value
-- configuration is a later refinement, not required for the object to be
-- usable end to end today).
CREATE TABLE custom_records (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    object_key TEXT NOT NULL,
    display_number TEXT NOT NULL,
    primary_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive', 'Archived')),
    owner_user_id TEXT REFERENCES users(id),
    notes TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    archived_at TEXT,
    UNIQUE (workspace_id, object_key, display_number)
);
CREATE INDEX idx_custom_records_lookup ON custom_records(workspace_id, object_key, status);
CREATE INDEX idx_custom_records_name ON custom_records(primary_name);
