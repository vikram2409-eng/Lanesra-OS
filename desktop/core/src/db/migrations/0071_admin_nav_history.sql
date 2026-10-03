-- Admin Control Center Modernization (issue #197): "Global Admin Search
-- ... plus Recently Viewed/Pinned admin items." One row per
-- (workspace, user, admin_tab) - visiting a tab upserts `last_viewed_at`;
-- pinning just flips `is_pinned` on the same row rather than a second
-- table, since a pinned tab is still "recently viewed" by definition.
CREATE TABLE admin_nav_history (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  admin_tab TEXT NOT NULL,
  label TEXT NOT NULL,
  is_pinned INTEGER NOT NULL DEFAULT 0,
  last_viewed_at TEXT NOT NULL,
  UNIQUE (workspace_id, user_id, admin_tab)
);

CREATE INDEX idx_admin_nav_history_user ON admin_nav_history(workspace_id, user_id, last_viewed_at);
