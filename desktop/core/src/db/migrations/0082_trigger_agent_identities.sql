-- Migration 0082: unattended triggers run as an Agent Identity (ADR-0016 §1)
-- Work package: WP-0109
-- Change: data (two additive columns, then one Agent Identity per trigger)
-- Data touched: ai_agent_triggers.agent_identity_id set; agent_identities and audit_events rows added
-- Re-run safe: no (ALTER TABLE; the data step only touches triggers that have no identity yet)
--
-- Every schedule and webhook Trigger gets its own Agent Identity, created with
-- the narrowest rights that keep it working exactly as before: not an
-- Administrator and no Access Roles. Before this migration a drained run had
-- no actor at all (so no Administrator rights), and since WP-0107 a webhook
-- run never borrows its API client owner's Administrator rights, so nothing
-- that worked stops working and nothing gains a right. Each assignment is
-- written to the audit log so the admin can see what changed.
ALTER TABLE ai_agent_triggers ADD COLUMN agent_identity_id TEXT
    REFERENCES agent_identities(id) ON DELETE SET NULL;

ALTER TABLE ai_agent_pending_runs ADD COLUMN agent_identity_id TEXT;

-- The accountable owner: whoever created the trigger, or else the workspace's
-- first Administrator.
INSERT INTO agent_identities
    (id, workspace_id, name, description, owner_user_id, is_administrator, status,
     created_at, created_by, updated_at, updated_by)
SELECT
    lower(hex(randomblob(16))),
    t.workspace_id,
    -- The full trigger id keeps the name unique (agent_identities has
    -- UNIQUE (workspace_id, name)).
    'Trigger ' || t.trigger_type || ' ' || t.id,
    'Created on upgrade for trigger ' || t.id,
    COALESCE(
        (SELECT u.id FROM users u WHERE u.id = t.created_by AND u.workspace_id = t.workspace_id),
        (SELECT u.id FROM users u
           JOIN user_roles ur ON ur.user_id = u.id
           JOIN roles r ON r.id = ur.role_id AND r.name = 'Administrator'
          WHERE u.workspace_id = t.workspace_id
          ORDER BY u.created_at LIMIT 1)
    ),
    0,
    'active',
    strftime('%Y-%m-%dT%H:%M:%S+00:00', 'now'),
    NULL,
    strftime('%Y-%m-%dT%H:%M:%S+00:00', 'now'),
    NULL
FROM ai_agent_triggers t
WHERE t.agent_identity_id IS NULL
  AND COALESCE(
        (SELECT u.id FROM users u WHERE u.id = t.created_by AND u.workspace_id = t.workspace_id),
        (SELECT u.id FROM users u
           JOIN user_roles ur ON ur.user_id = u.id
           JOIN roles r ON r.id = ur.role_id AND r.name = 'Administrator'
          WHERE u.workspace_id = t.workspace_id
          ORDER BY u.created_at LIMIT 1)
      ) IS NOT NULL
  AND NOT EXISTS (
        SELECT 1 FROM agent_identities ai
         WHERE ai.description = 'Created on upgrade for trigger ' || t.id
      );

UPDATE ai_agent_triggers
SET agent_identity_id = (
    SELECT ai.id FROM agent_identities ai
     WHERE ai.description = 'Created on upgrade for trigger ' || ai_agent_triggers.id
)
WHERE agent_identity_id IS NULL;

INSERT INTO audit_events
    (id, workspace_id, occurred_at, user_id, event_type, entity_type, entity_id, summary, details_json)
SELECT
    lower(hex(randomblob(16))),
    t.workspace_id,
    strftime('%Y-%m-%dT%H:%M:%S+00:00', 'now'),
    NULL,
    'agent_identity_assigned_on_upgrade',
    'AiAgentTrigger',
    t.id,
    'This ' || t.trigger_type || ' trigger now runs as Agent Identity ''' || ai.name
      || ''' (not an Administrator, no Access Roles). Grant it only what it needs.',
    NULL
FROM ai_agent_triggers t
JOIN agent_identities ai ON ai.id = t.agent_identity_id
WHERE ai.description = 'Created on upgrade for trigger ' || t.id
  AND NOT EXISTS (
        SELECT 1 FROM audit_events e
         WHERE e.event_type = 'agent_identity_assigned_on_upgrade' AND e.entity_id = t.id
      );
