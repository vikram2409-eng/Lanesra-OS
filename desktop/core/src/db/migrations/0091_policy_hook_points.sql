-- Migration 0091: policy hook points for guardrails (WP-0128)
-- Work package: WP-0128
-- Change: additive (one nullable-defaulted column on ai_guardrails)
-- Data touched: none; existing guardrails keep 'off' (not wired into the loop)
-- Re-run safe: no (ALTER TABLE ADD COLUMN)
--
-- WP-0128 wires guardrails into the agent run loop at three hook points
-- (ADR-0008 trace, REQ-LOOP-017): a step boundary, a model call and a tool
-- call. A guardrail's hook_point says where in the loop it runs:
--   'off'   - not wired into the loop (the WP-0133 default; the guardrail is
--             still a usable object, just never evaluated by the run loop);
--   'step'  - evaluated at each step boundary, against the step input;
--   'model' - evaluated against each model call's output text;
--   'tool'  - evaluated against each tool call's arguments.
-- At each point the policy engine runs every enabled guardrail attached
-- there; the first that fails blocks the loop and short-circuits the
-- remaining hook points in order (step, then model, then tool). Every
-- verdict is recorded on a trace span and emitted as a policy-decision run
-- event. A workspace that attaches no guardrail to any point sees no
-- behavior change - purely additive, the same opt-in convention the rest of
-- this codebase's governance surfaces follow.
ALTER TABLE ai_guardrails ADD COLUMN hook_point TEXT NOT NULL DEFAULT 'off';

-- Finding the guardrails wired at one point for a workspace is the hot path
-- the loop runs on every step, model call and tool call.
CREATE INDEX idx_ai_guardrails_hook_point
    ON ai_guardrails (workspace_id, hook_point) WHERE hook_point <> 'off';
