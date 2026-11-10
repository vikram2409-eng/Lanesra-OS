-- Migration 0093: agent persona and reasoning mode (WP-0120)
-- Work package: WP-0120
-- Change: additive (four defaulted columns on ai_agents and ai_agent_versions)
-- Data touched: none; existing agents keep empty persona and 'direct' mode
-- Re-run safe: no (ALTER TABLE ADD COLUMN)
--
-- WP-0120 gives an agent a persona (role, goal, background) and a reasoning
-- mode, compiled with its instructions, guardrails, memory digest and
-- ontology context into the system prompt (REQ-CORE-001, REQ-CORE-002).
-- Persona is set by its own admin action (ai_agent_service::set_persona),
-- the same separate-from-the-create/update-form shape memory_md and
-- guardrails_md already use, and snapshotted into a published version the
-- same way. reasoning_mode is one of 'direct' (today's behaviour, the
-- default), 'plan_first' (a plan is produced and shown in the trace before
-- any tool use) or 'reflect' (one self-critique model call before the final
-- answer) - REQ-LOOP-015. Limits are already modelled per agent by the
-- Gateway routing policy (WP-0121, model_routing), so no new limit column is
-- added here. An agent that sets none of these behaves exactly as before.
ALTER TABLE ai_agents ADD COLUMN role TEXT NOT NULL DEFAULT '';
ALTER TABLE ai_agents ADD COLUMN goal TEXT NOT NULL DEFAULT '';
ALTER TABLE ai_agents ADD COLUMN background TEXT NOT NULL DEFAULT '';
ALTER TABLE ai_agents ADD COLUMN reasoning_mode TEXT NOT NULL DEFAULT 'direct';

ALTER TABLE ai_agent_versions ADD COLUMN role TEXT NOT NULL DEFAULT '';
ALTER TABLE ai_agent_versions ADD COLUMN goal TEXT NOT NULL DEFAULT '';
ALTER TABLE ai_agent_versions ADD COLUMN background TEXT NOT NULL DEFAULT '';
ALTER TABLE ai_agent_versions ADD COLUMN reasoning_mode TEXT NOT NULL DEFAULT 'direct';
