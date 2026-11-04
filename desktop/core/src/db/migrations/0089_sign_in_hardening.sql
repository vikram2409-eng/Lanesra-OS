-- Migration 0089: Team Workspace first-run and sign-in hardening (WP-0106)
-- Work package: WP-0106
-- Change: additive (two tables)
-- Data touched: none; both tables are created empty and filled lazily
-- Re-run safe: no (CREATE TABLE)
--
-- Two one-row-per-workspace style tables, lazily created on first read via
-- the repos' ensure_default (like integration_settings/ai_settings), so an
-- existing workspace picks these up with no data migration.

-- Per-workspace security configuration: sign-in lockout thresholds, web
-- session lifetime and idle timeout, and the public base URL the server is
-- reached at (used by the SSO work packages for redirect/assertion URLs,
-- links in notifications and CORS defaults). An admin edits these; the
-- public URL can also come from the LANESRA_PUBLIC_URL environment
-- variable, which wins over the stored value when set.
CREATE TABLE security_settings (
    workspace_id TEXT PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    lockout_max_failures INTEGER NOT NULL DEFAULT 10,
    lockout_minutes INTEGER NOT NULL DEFAULT 15,
    session_lifetime_hours INTEGER NOT NULL DEFAULT 336,   -- 24 * 14
    session_idle_minutes INTEGER NOT NULL DEFAULT 0,       -- 0 = no idle timeout
    public_base_url TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);

-- Brute-force state, one row per (workspace, identifier). The identifier is
-- a lowercased username ("user:<name>") or a client address ("addr:<ip>"),
-- so sign-in is throttled per account and per source address. A failure
-- increments failed_count and, past the threshold, sets locked_until_ms;
-- a success clears the row. Times are epoch-millisecond integers to match
-- the ai_tool_call_window convention and keep the throttle unit-testable
-- with an injected clock.
CREATE TABLE sign_in_lockouts (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    identifier TEXT NOT NULL,
    failed_count INTEGER NOT NULL DEFAULT 0,
    first_failed_at_ms INTEGER NOT NULL,
    locked_until_ms INTEGER,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (workspace_id, identifier)
);
