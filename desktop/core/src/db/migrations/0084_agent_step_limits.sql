-- Migration 0084: per-agent step limits (WP-0121)
-- Work package: WP-0121
-- Change: schema (additive nullable columns)
-- Data touched: none; existing routing rows keep NULL, which means the defaults
-- Re-run safe: no (ALTER TABLE)
--
-- An agent's routing gains three limits. max_rounds caps the tool-calling
-- rounds in one run (default 8, at most 25). run_token_limit caps the tokens
-- one run may use, input and output together (no cap when NULL). Reaching
-- either asks the model once for a final answer with tools switched off and
-- keeps it as a partial result. step_timeout_seconds caps how long one model
-- call may take (default 300); a call that runs over is cancelled and tried
-- once on the next routing tier.
ALTER TABLE ai_agent_routing ADD COLUMN max_rounds INTEGER;
ALTER TABLE ai_agent_routing ADD COLUMN run_token_limit INTEGER;
ALTER TABLE ai_agent_routing ADD COLUMN step_timeout_seconds INTEGER;
