-- Enterprise Access Foundation, Phase 1 continued: Work Teams - first-class
-- security principals that can own records, distinct from an Access Group
-- (a later, Phase-2+ concept, not built yet) which never owns anything.
CREATE TABLE work_teams (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    team_type TEXT NOT NULL DEFAULT 'Operational' CHECK (team_type IN ('Operational', 'Queue', 'Project', 'CrossFunctional', 'ExternalPartner')),
    primary_org_unit_id TEXT NOT NULL REFERENCES org_units(id),
    owner_user_id TEXT REFERENCES users(id),
    can_own_records INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive')),
    effective_from TEXT,
    effective_to TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    UNIQUE (workspace_id, code)
);
CREATE INDEX idx_work_teams_workspace ON work_teams(workspace_id, status);

-- Membership has its own effective dates and is independent of record
-- ownership by design: ending a membership (or deleting the row once its
-- effective_to has passed) must never touch any record's record_owner_id -
-- a team keeps whatever it owns regardless of who is currently on it. See
-- team_membership_service::end_membership's own doc comment, and the
-- access_foundation_org_structure.rs test that pins this behavior down.
-- team_id cascades on delete: work_team_service::delete only guards against
-- *active* memberships (an ended one is exactly what should let a team be
-- retired), so a historical, already-ended membership row must not be able
-- to block the team's own deletion via this FK.
CREATE TABLE team_memberships (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    team_id TEXT NOT NULL REFERENCES work_teams(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id),
    role_in_team TEXT,
    effective_from TEXT NOT NULL,
    effective_to TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_team_memberships_team ON team_memberships(team_id, effective_to);
CREATE INDEX idx_team_memberships_user ON team_memberships(user_id);
