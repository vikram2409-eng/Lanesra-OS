-- Browser session storage for Team Workspace mode (multiple users on one
-- shared server instance). Not used by the desktop app, which tracks a
-- single in-memory session per process instead.

CREATE TABLE web_sessions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    user_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    expires_at TEXT NOT NULL
);
CREATE INDEX idx_web_sessions_expires ON web_sessions(expires_at);
