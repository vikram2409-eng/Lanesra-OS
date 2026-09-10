-- AI & Agentic Layer, Phase 6: the AI Agent Foundry. Admin-defined named
-- AI Agents - a persona/system prompt layered over a curated set of
-- Actions (individual tool names, drawn from the same catalogs
-- chat_service's built-in records/admin assistants already use - see
-- ai_agent_service::tool_source), persistent Memory, a reusable Skills
-- library, and Agent-to-Agent delegation (hierarchy). Orchestration
-- (Pipelines/Triggers/Runs) lands in a later migration alongside its own
-- phase - this one is just the Agents/Skills/Memory/Actions/Delegation
-- half (see chat_service::run_agent_once/send_agent_message).

CREATE TABLE ai_agents (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT NOT NULL DEFAULT '🤖',
    system_prompt TEXT NOT NULL,
    -- A living document the agent reads every run and can revise itself
    -- via the always-available update_memory tool - never empty-checked,
    -- '' just means "nothing written yet".
    memory_md TEXT NOT NULL DEFAULT '',
    -- A JSON array of tool names (e.g. ["list_records","get_record"]),
    -- picked from chat_service's existing record_tools()/admin_tools()
    -- catalogs - not a new tool implementation. Whether this agent needs
    -- Administrator is computed from this set at runtime
    -- (ai_agent_service::agent_requires_admin), not stored separately.
    action_names_json TEXT NOT NULL DEFAULT '[]',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT
);
CREATE INDEX idx_ai_agents_workspace ON ai_agents (workspace_id, is_active);

-- Which other active Agents an Agent may call via the delegate_to_agent
-- tool - hierarchy, not a fixed org chart. Cycles aren't rejected at
-- write time (an indirect cycle across 3+ agents isn't worth a graph
-- walk here); chat_service's runtime delegation-depth guard is the
-- actual safety net, the same "a depth guard, not exhaustive static
-- analysis" choice workflow_service's own recursion guard already made.
CREATE TABLE ai_agent_delegates (
    agent_id TEXT NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    delegate_agent_id TEXT NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    PRIMARY KEY (agent_id, delegate_agent_id)
);

-- A reusable library of instructions, not tied to one Agent - the same
-- "short description up front, full content loaded only on demand" shape
-- this session's own Skill tool already uses. instructions_md is only
-- ever returned to the model via the use_skill tool result, never
-- injected into every turn's system prompt (only name+description are).
CREATE TABLE ai_skills (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    instructions_md TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT
);
CREATE INDEX idx_ai_skills_workspace ON ai_skills (workspace_id, is_active);

CREATE TABLE ai_agent_skills (
    agent_id TEXT NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    skill_id TEXT NOT NULL REFERENCES ai_skills(id) ON DELETE CASCADE,
    PRIMARY KEY (agent_id, skill_id)
);
