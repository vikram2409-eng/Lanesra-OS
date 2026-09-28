-- AI Agent Platform v2, Phase 1: agent versioning, a logical Model
-- Reference indirection, and a durable Approval Service - the foundation
-- every later phase (Toolboxes/Policy Engine, the Execution Graph,
-- Solution packaging of agents) references.
--
-- `ai_agents` today (migration 0038) is a single mutable row with no
-- version lifecycle - editing it in place is the only way to change a
-- persona/instructions/tools/model routing. This migration adds
-- `ai_agent_versions` alongside it as an immutable-once-published
-- snapshot per version (Draft -> Test -> Published -> Deprecated ->
-- Disabled, enforced by the not-yet-written agent_version_service, not
-- by a DB trigger - the same "service owns the state machine, the table
-- just stores it" shape ai_agent_pipeline's own topology validation
-- already uses). `ai_agents` gains a nullable `current_version_id`
-- pointer and becomes the stable identity row going forward; its own
-- persona/instructions/etc. columns are left in place unchanged in this
-- pass (existing reads/writes in ai_agent_service/chat_service keep
-- working exactly as before) - the version row is an additive, parallel
-- history rather than a breaking cutover. Every existing agent gets a
-- single "v1" Published version backfilled from its current fields
-- below, so nothing is ever agent-versioned-but-versionless.
CREATE TABLE ai_agent_versions (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    -- 'draft' | 'test' | 'published' | 'deprecated' | 'disabled'.
    status TEXT NOT NULL DEFAULT 'draft',
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT NOT NULL,
    system_prompt TEXT NOT NULL,
    memory_md TEXT NOT NULL DEFAULT '',
    guardrails_md TEXT NOT NULL DEFAULT '',
    action_names_json TEXT NOT NULL DEFAULT '[]',
    delegate_agent_ids_json TEXT NOT NULL DEFAULT '[]',
    skill_ids_json TEXT NOT NULL DEFAULT '[]',
    -- A snapshot of AiAgentModelRouting at the time this version was
    -- authored (NULL means "no per-agent routing policy", same meaning
    -- as ai_agent_routing having no row today). Routing itself is still
    -- resolved from the live ai_agent_routing table at dispatch time in
    -- this phase - this column is version history, not (yet) the
    -- authoritative source `ai_gateway_service` reads from.
    model_routing_json TEXT,
    -- A JSON Schema an agent version may declare for Structured Output -
    -- validated against the model's final answer in chat_service's
    -- existing response-parsing path, with one repair retry on failure.
    -- NULL means free-form text output, exactly today's behavior.
    output_schema_json TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    published_at TEXT,
    UNIQUE (agent_id, version_number)
);
CREATE INDEX idx_ai_agent_versions_agent ON ai_agent_versions (agent_id, version_number);
CREATE INDEX idx_ai_agent_versions_status ON ai_agent_versions (agent_id, status);

ALTER TABLE ai_agents ADD COLUMN current_version_id TEXT REFERENCES ai_agent_versions(id) ON DELETE SET NULL;

-- Backfill: every existing agent gets exactly one Published v1 version,
-- snapshotting its current row, with ai_agents.current_version_id
-- pointed at it. new_uuid()/now_iso() have no SQL-level equivalent, so
-- the id is deterministically derived from agent_id (stable, unique per
-- agent, never collides with a uuid4 generated afterward) and the
-- timestamp is copied from the agent's own created_at rather than
-- "now" - a backfilled v1 was conceptually created alongside the agent.
INSERT INTO ai_agent_versions (
    id, agent_id, version_number, status, name, description, icon, system_prompt,
    memory_md, guardrails_md, action_names_json, delegate_agent_ids_json, skill_ids_json,
    created_at, created_by, published_at
)
SELECT
    'v1-' || a.id, a.id, 1, 'published', a.name, a.description, a.icon, a.system_prompt,
    a.memory_md, a.guardrails_md, a.action_names_json,
    COALESCE((SELECT '[' || GROUP_CONCAT('"' || REPLACE(d.delegate_agent_id, '"', '\"') || '"') || ']'
              FROM ai_agent_delegates d WHERE d.agent_id = a.id), '[]'),
    COALESCE((SELECT '[' || GROUP_CONCAT('"' || REPLACE(s.skill_id, '"', '\"') || '"') || ']'
              FROM ai_agent_skills s WHERE s.agent_id = a.id), '[]'),
    a.created_at, a.created_by, a.created_at
FROM ai_agents a;

UPDATE ai_agents SET current_version_id = 'v1-' || id;

-- A logical, per-workspace Model Reference - a name (e.g. "primary",
-- "fast", "cheap") a Solution Package or an agent version can portably
-- declare, bound to a concrete ai_providers row per workspace. Today
-- ai_agent_routing points directly at ai_providers.id, which is fine for
-- a single workspace but not something a packaged Solution can export
-- and have resolve sensibly in a different workspace with different
-- provider rows - this indirection is what later phases (Solution
-- packaging of agents, in particular) resolve through instead.
CREATE TABLE ai_agent_model_refs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    provider_id TEXT NOT NULL REFERENCES ai_providers(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    UNIQUE (workspace_id, name)
);

-- A durable, generalized pending-action row - the foundation Phase 2's
-- Tool-Call Firewall and later phases gate sensitive actions through,
-- and the eventual replacement for ai_agent_runs' own ad hoc
-- awaiting_approval/paused_at_step_order pair (PipelineStep.
-- requires_approval today only pauses a run in place; it isn't wired to
-- this table yet - that rewiring is Phase 2/3 work, not this migration).
-- `subject_type`/`subject_id` name what's pending (e.g.
-- ('agent_run_step', <run_id>) or ('agent_version_publish', <version_id>))
-- rather than a column-per-subject-kind, since the set of approvable
-- things is expected to grow with every later phase.
CREATE TABLE ai_approvals (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    subject_type TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    -- A snapshot of exactly what's being proposed - the step's proposed
    -- output, a version's proposed diff, etc. - so approving/rejecting
    -- later never depends on subject_id's row still looking the same.
    proposal_json TEXT NOT NULL,
    -- 'pending' | 'approved' | 'rejected'.
    status TEXT NOT NULL DEFAULT 'pending',
    requested_by TEXT,
    resolved_by TEXT,
    resolution_notes TEXT,
    created_at TEXT NOT NULL,
    resolved_at TEXT
);
CREATE INDEX idx_ai_approvals_workspace_status ON ai_approvals (workspace_id, status);
CREATE INDEX idx_ai_approvals_subject ON ai_approvals (subject_type, subject_id);
