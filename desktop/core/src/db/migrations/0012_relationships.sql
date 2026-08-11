-- Admin extensibility Phase B (spec §20.3/§24.1): admin-defined
-- relationships between two entity types - built-in to built-in, built-in
-- to custom object, or custom object to custom object. `source_entity_type`
-- is the "many"/owning side (the side that holds the link), matching how
-- Contact 1:N-relates to Company today: Contact is `source`, Company is
-- `target`. one_to_many and many_to_one are therefore the same physical
-- shape read from either direction - the UI's forward_label/reverse_label
-- pair is what makes the direction legible, not a fourth relationship_type
-- value.
--
-- Like field_rules/workflow_rules before it, entity_type columns here are
-- plain TEXT rather than a CHECK/FK enumeration - the same "any built-in
-- name or any active custom object key for this workspace" vocabulary
-- entity_registry validates at the service layer.

CREATE TABLE relationship_definitions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    key TEXT NOT NULL,
    source_entity_type TEXT NOT NULL,
    target_entity_type TEXT NOT NULL,
    relationship_type TEXT NOT NULL CHECK (relationship_type IN ('many_to_one', 'one_to_one', 'many_to_many')),
    forward_label TEXT NOT NULL,
    reverse_label TEXT NOT NULL,
    is_required INTEGER NOT NULL DEFAULT 0,
    show_related_list INTEGER NOT NULL DEFAULT 1,
    delete_behavior TEXT NOT NULL DEFAULT 'restrict' CHECK (delete_behavior IN ('restrict', 'archive')),
    is_protected INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    UNIQUE (workspace_id, key)
);
CREATE INDEX idx_relationship_definitions_workspace ON relationship_definitions(workspace_id, is_active);

-- One row per actual link between two records. Cardinality
-- (many_to_one/one_to_one uniqueness) is enforced in relationship_service,
-- not here - SQLite partial unique indexes cannot condition on a sibling
-- table's column (relationship_definitions.relationship_type) without a
-- trigger, and the service layer already owns equivalent validation for
-- every other subsystem in this product (business rules, workflow rules,
-- custom field shape).
CREATE TABLE relationship_instances (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    relationship_definition_id TEXT NOT NULL REFERENCES relationship_definitions(id),
    source_entity_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    target_entity_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    UNIQUE (relationship_definition_id, source_id, target_id)
);
CREATE INDEX idx_relationship_instances_source ON relationship_instances(relationship_definition_id, source_entity_type, source_id);
CREATE INDEX idx_relationship_instances_target ON relationship_instances(relationship_definition_id, target_entity_type, target_id);
