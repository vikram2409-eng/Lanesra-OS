-- Consistency pass: every admin-configurable record in the product should
-- carry who created/last modified it, the same as the 9 built-in business
-- entities already do. Auditing this migration's siblings turned up two
-- shapes of gap:
--
-- 1. custom_records, custom_field_definitions, custom_reports,
--    relationship_definitions, relationship_instances, business_rules and
--    workflow_definitions already have created_by/updated_by columns
--    (added alongside those tables) and their repositories already write
--    them - the actor was simply never read back out into the Rust model
--    returned to callers. That's a Rust-only fix (see the accompanying
--    model/repository changes); no schema change needed for those seven.
-- 2. status_transitions and numbering_configs never had the columns at
--    all - this migration adds them to close that second gap.
--
-- Both columns are nullable: existing rows (and any future write from a
-- code path with no authenticated actor, e.g. a system/scheduled job)
-- simply carry NULL, exactly like every other created_by/updated_by
-- column in this schema - "unknown actor" is a valid, expected state, not
-- an error.
ALTER TABLE status_transitions ADD COLUMN created_by TEXT REFERENCES users(id);
ALTER TABLE status_transitions ADD COLUMN updated_by TEXT REFERENCES users(id);

ALTER TABLE numbering_configs ADD COLUMN created_by TEXT REFERENCES users(id);
ALTER TABLE numbering_configs ADD COLUMN updated_by TEXT REFERENCES users(id);
