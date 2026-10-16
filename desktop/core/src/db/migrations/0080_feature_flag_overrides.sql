-- Migration 0080: Feature flag overrides per workspace (ADR-0012)
-- Work package: WP-0010
-- Change: additive
-- Data touched: none
-- Re-run safe: yes
--
-- Flags themselves are defined in code (lanesra_core::services::
-- feature_flag_service::REGISTRY) with a default per edition; this table
-- holds only an Administrator's per-workspace override of that default.
-- No row means "use the edition default".
CREATE TABLE IF NOT EXISTS feature_flag_overrides (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    flag_key TEXT NOT NULL,
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    updated_by_principal_id TEXT,
    UNIQUE (workspace_id, flag_key)
);
