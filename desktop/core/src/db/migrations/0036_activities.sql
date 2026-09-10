-- AI & Agentic Layer, phase 3 (Unified Activity Timeline): completes a
-- table that has existed since 0001_init.sql but was never wired up to
-- any Rust model/repo/service - `activities`
-- (related_type/related_id/activity_type/notes), already almost
-- exactly this feature's shape. Renamed here to the
-- entity_type/entity_id naming models::audit::AuditEvent already
-- established for "what this thing is attached to" (and channel/body
-- to match), and given the columns it was still missing: `direction`
-- (meaningful for email/message, not a call), `participants`, and
-- `source` ('manual' for everything logged this phase - see
-- activity_service's own doc comment for what a future automated
-- channel does with it).
--
-- entity_type/entity_id/body stayed nullable at the SQL level (SQLite
-- can't add a NOT NULL constraint to an already-nullable column without
-- a full table rebuild) - enforced instead at the application layer in
-- activity_service::validate, the only writer this table has ever had.
ALTER TABLE activities RENAME COLUMN related_type TO entity_type;
ALTER TABLE activities RENAME COLUMN related_id TO entity_id;
ALTER TABLE activities RENAME COLUMN activity_type TO channel;
ALTER TABLE activities RENAME COLUMN notes TO body;
ALTER TABLE activities ADD COLUMN direction TEXT;
ALTER TABLE activities ADD COLUMN participants TEXT;
ALTER TABLE activities ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';

DROP INDEX idx_activities_related;
CREATE INDEX idx_activities_entity ON activities (entity_type, entity_id, occurred_at DESC);
