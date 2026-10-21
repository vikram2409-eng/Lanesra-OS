-- Migration 0081: Agent Identities and their credentials (ADR-0006, ADR-0016 §1)
-- Work package: WP-0108
-- Change: additive
-- Data touched: none
-- Re-run safe: yes
--
-- An Agent Identity is the principal an unattended run acts as. It belongs to
-- exactly one workspace and never signs in. Its rights come from Access
-- Control v1: Access Roles (agent_identity_access_roles) plus an explicit
-- Administrator flag, the same two things a user has. An accountable human
-- owns it. Credentials are Lanesra-issued, short-lived tokens; only
-- sha256(token) is stored, the token itself is shown once and never again.
CREATE TABLE IF NOT EXISTS agent_identities (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    is_administrator INTEGER NOT NULL DEFAULT 0 CHECK (is_administrator IN (0, 1)),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    UNIQUE (workspace_id, name)
);

CREATE TABLE IF NOT EXISTS agent_identity_access_roles (
    agent_identity_id TEXT NOT NULL REFERENCES agent_identities(id) ON DELETE CASCADE,
    access_role_id TEXT NOT NULL REFERENCES access_roles(id) ON DELETE CASCADE,
    PRIMARY KEY (agent_identity_id, access_role_id)
);

CREATE TABLE IF NOT EXISTS agent_identity_credentials (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    agent_identity_id TEXT NOT NULL REFERENCES agent_identities(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    revoked_at TEXT,
    revoked_by TEXT,
    last_used_at TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT
);

CREATE INDEX IF NOT EXISTS idx_agent_identities_workspace
    ON agent_identities(workspace_id);
CREATE INDEX IF NOT EXISTS idx_agent_identity_credentials_identity
    ON agent_identity_credentials(agent_identity_id);
