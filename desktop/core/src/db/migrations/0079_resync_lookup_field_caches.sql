-- Re-sync every Lookup field's cached value with the relationship link it
-- mirrors. Before this release a link made by anything other than a lookup
-- field save (a workflow's create_record action, the Related-list UI, an
-- import, the API) left the field's custom_field_values cache empty, and
-- the next ordinary save of that record wrote the empty value back and
-- deleted the link. relationship_service now keeps the cache in step on
-- every link and unlink; this repairs caches already out of step.
--
-- relationship_instances is the data of record. A field on the
-- definition's source type caches the target id; the reverse field a
-- one_to_one definition creates on the target type caches the source id
-- (the side rule relationship_service::set_lookup_value uses).

-- 1. Source-side lookup fields: cache = linked target id.
INSERT INTO custom_field_values (id, definition_id, entity_id, value_text, created_at, updated_at)
SELECT lower(hex(randomblob(16))), f.id, i.source_id, i.target_id,
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM custom_field_definitions f
JOIN relationship_definitions r ON r.id = f.lookup_relationship_definition_id
JOIN relationship_instances i ON i.relationship_definition_id = r.id
WHERE f.field_type = 'lookup' AND f.entity_type = r.source_entity_type
ON CONFLICT (definition_id, entity_id) DO UPDATE SET value_text = excluded.value_text, updated_at = excluded.updated_at;

-- 2. Target-side (reverse, one_to_one) lookup fields: cache = linked source id.
INSERT INTO custom_field_values (id, definition_id, entity_id, value_text, created_at, updated_at)
SELECT lower(hex(randomblob(16))), f.id, i.target_id, i.source_id,
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM custom_field_definitions f
JOIN relationship_definitions r ON r.id = f.lookup_relationship_definition_id
JOIN relationship_instances i ON i.relationship_definition_id = r.id
WHERE f.field_type = 'lookup' AND f.entity_type = r.target_entity_type AND f.entity_type <> r.source_entity_type
ON CONFLICT (definition_id, entity_id) DO UPDATE SET value_text = excluded.value_text, updated_at = excluded.updated_at;

-- 3. Drop cached values that no longer point at an existing link.
DELETE FROM custom_field_values
WHERE definition_id IN (SELECT id FROM custom_field_definitions WHERE field_type = 'lookup')
  AND NOT EXISTS (
    SELECT 1
    FROM custom_field_definitions f
    JOIN relationship_definitions r ON r.id = f.lookup_relationship_definition_id
    JOIN relationship_instances i ON i.relationship_definition_id = r.id
    WHERE f.id = custom_field_values.definition_id
      AND ((f.entity_type = r.source_entity_type AND i.source_id = custom_field_values.entity_id AND i.target_id = custom_field_values.value_text)
        OR (f.entity_type = r.target_entity_type AND i.target_id = custom_field_values.entity_id AND i.source_id = custom_field_values.value_text))
  );
