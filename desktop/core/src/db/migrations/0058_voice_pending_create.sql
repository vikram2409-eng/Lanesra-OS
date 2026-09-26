-- Voice-First Mode, PR 2 (part 3): guided step-by-step record creation.
-- A session-scoped, opaque JSON blob holding an in-progress voice CREATE
-- (object_key + the fields collected so far + which field is currently
-- being asked about) - the exact same "one bounded JSON column on
-- voice_sessions" convention conversation_json already established, kept
-- as its own column rather than folded into conversation_json since it's
-- a genuinely different kind of state (a create in progress, not a
-- resolved-record reference history). NULL means no guided create is
-- currently in progress for this session.
ALTER TABLE voice_sessions ADD COLUMN pending_create_json TEXT;
