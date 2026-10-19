-- Migration 0081: Agent Identity management (WP-0108)
-- Work package: WP-0108
-- Change: additive
-- Data touched: none
-- Re-run safe: yes
--
-- Agent Identities are Lanesra-issued identities for AI agents, per ADR-0016 §1.
-- Each belongs to exactly one workspace and never crosses it.
-- An owning human is recorded; management is admin-only and audited.
CREATE TABLE IF NOT EXISTS ai_agent_identities (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    -- Owning human's principal id (accountability)
    owner_principal_id TEXT NOT NULL,
    -- Access Role for this identity (Viewer, Editor, Administrator, etc.)
    access_role TEXT NOT NULL DEFAULT 'Viewer',
    -- Record scope as JSON
    record_scope TEXT NOT NULL DEFAULT '{"type": "all"}',
    -- Status: active, disabled
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by_principal_id TEXT,
    UNIQUE (workspace_id, name)
);

-- Agent credentials: short-lived tokens bound to one Agent Identity.
-- Secret material is stored encrypted; the plaintext token is shown only once at issuance.
CREATE TABLE IF NOT EXISTS ai_agent_credentials (
    id TEXT PRIMARY KEY,
    agent_identity_id TEXT NOT NULL REFERENCES ai_agent_identities(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    -- Human-readable label for the credential
    label TEXT NOT NULL,
    -- SHA-256 hash of the token (for lookup); the token itself is never stored
    token_hash TEXT NOT NULL UNIQUE,
    -- Encrypted token material (for rotation display); NULL after first view
    encrypted_token TEXT,
    -- Scope restriction for this credential (JSON, subset of identity's scope)
    scope TEXT NOT NULL DEFAULT '{"type": "all"}',
    -- Expiry: credentials are short-lived
    expires_at TEXT NOT NULL,
    -- Revocation: NULL = not revoked
    revoked_at TEXT,
    revoked_by_principal_id TEXT,
    -- Last use tracking
    last_used_at TEXT,
    created_at TEXT NOT NULL,
    created_by_principal_id TEXT,
    UNIQUE (agent_identity_id, label)
);

CREATE INDEX IF NOT EXISTS idx_agent_identities_workspace
    ON ai_agent_identities(workspace_id);
CREATE INDEX IF NOT EXISTS idx_agent_credentials_identity
    ON ai_agent_credentials(agent_identity_id);
CREATE INDEX IF NOT EXISTS idx_agent_credentials_hash
    ON ai_agent_credentials(token_hash);
