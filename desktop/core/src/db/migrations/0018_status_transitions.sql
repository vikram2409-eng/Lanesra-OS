-- Admin Automation & Customization addendum, Phase 2 (spec §2.5): a
-- dedicated Status Transition editor rather than forcing administrators to
-- model every allowed From -> To state change as a generic field rule.
--
-- `from_status` is nullable: NULL means "from any status" (a wildcard
-- entry, e.g. "any status -> Cancelled" without listing every from-value
-- individually). `to_status` is always a specific value - a rule always
-- names a destination.
--
-- No rows for an (workspace_id, entity_type) pair means that entity type's
-- transitions stay fully unrestricted (today's behavior, preserved for
-- backward compatibility) - the moment an Administrator adds the first
-- active rule for an entity type, that entity's transitions become an
-- allow-list.
CREATE TABLE status_transitions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL,
    from_status TEXT,
    to_status TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_status_transitions_workspace_entity ON status_transitions(workspace_id, entity_type);
