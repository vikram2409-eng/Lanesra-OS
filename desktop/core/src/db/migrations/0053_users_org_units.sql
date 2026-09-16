-- Enterprise Access Foundation, Phase 1 continued: every user gets a
-- Primary Organization Unit (nullable at the schema level - "required for
-- internal users" is service-layer validation, since SQLite can't add a
-- NOT NULL column without a uniform default to an already-populated
-- table), plus optional Additional Organization Unit memberships in a
-- separate join table.
ALTER TABLE users ADD COLUMN primary_org_unit_id TEXT REFERENCES org_units(id);

UPDATE users SET primary_org_unit_id = (
    SELECT w.root_org_unit_id FROM workspaces w WHERE w.id = users.workspace_id
) WHERE primary_org_unit_id IS NULL;

CREATE TABLE user_org_units (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    user_id TEXT NOT NULL REFERENCES users(id),
    org_unit_id TEXT NOT NULL REFERENCES org_units(id),
    created_at TEXT NOT NULL,
    UNIQUE (user_id, org_unit_id)
);
CREATE INDEX idx_user_org_units_user ON user_org_units(user_id);
