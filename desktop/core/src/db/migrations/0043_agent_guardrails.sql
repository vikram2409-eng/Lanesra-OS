-- AI & Agentic Layer, Phase 7c: Guardrails. A per-agent free-text
-- operational-boundary statement, injected into its system prompt
-- alongside memory_md/skills (see chat_service::agent_system_prompt) -
-- advisory/prompted, not independently code-enforced, the same
-- real/advisory split this project already draws between Business Rules
-- (code-enforced) and an agent's own persona text (prompted). The one
-- genuinely code-enforced guard this phase adds - loop detection on 3
-- identical consecutive tool calls - needs no schema change; it's
-- entirely in chat_service::run_agent_once's in-memory round loop.
ALTER TABLE ai_agents ADD COLUMN guardrails_md TEXT NOT NULL DEFAULT '';
