---
id: WP-0121
title: Step limits and partial results
phase: 1
epic: E1.3 Agent loop
status: in_progress
owner: claude
depends_on: [WP-0110, WP-0005]
requirements: [REQ-LOOP-001, REQ-LOOP-008, REQ-LOOP-009, REQ-LOOP-010]
threats: [T-06]
estimate: L
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [agent-limits-and-partial-results]
branch: claude/enterprise-build-planning-jxy2kq
pr: 
updated: 2026-10-23
---

# WP-0121 — Step limits and partial results

## Goal

Every limit ends gracefully: round limit (default 8, max 25) and token limit ask once for a final answer with tools disabled and mark `completed_partial`; wall-clock limit cancels and retries once on fallback.

## Scope

- Make MAX_ROUNDS configurable per agent within bounds
- Token accounting per step
- Cancellation of provider calls on timeout
- Fallback retry once
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - At the round limit the loop returns an error and discards `produced` (`chat_service.rs:1973`); keep the partial output as `completed_partial`
  - Make model-routing `temperature` and `max_tokens` take effect (T-03)

## Acceptance criteria

- [x] AC-1: Each limit tested with the scripted provider: correct final status, final-answer request has no tools
- [x] AC-2: No limit path panics or exits the process

## Test plan

- `desktop/core/tests/agent_loop_limits.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0008 trace and events; ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0011 prompt catalog; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-19 muse: Refined proposed → ready. Added threats: [T-06] (threat model names WP-0121 as a cost-exhaustion mitigation). AC-1/AC-2 already testable; scope already carries the gap-audit findings (completed_partial, temperature/max_tokens). Verified: REQ-LOOP-001/008/009/010 exist; ADR-0007/0008/0009/0010/0011/0012 Accepted; depends_on WP-0005 done, WP-0110 still proposed (listed per Definition of Ready). No code changed.
- 2026-10-23 claude: Claimed and built. Limits live on the agent's routing (migration 0084 adds `max_rounds`, `run_token_limit`, `step_timeout_seconds` to `ai_agent_routing`); `AgentRunLimits` resolves them with defaults of 8 rounds, no token cap and 300 s a call, bounded 1 to 25 rounds, at least 1,000 tokens and 5 to 1,800 s, and the routing form refuses values outside those bounds. Round and token limits: the agent loop (`run_agent_once`) and the Assistant chat (`send_message`) no longer return an error and discard their work; they make one final call with tools off, whose instruction is the new catalog prompt `loop/final-answer` added to the system prompt (so no message the person did not write enters their history), and keep its answer. `AgentRunOutcome.stop_reason` says which limit was reached; a graph agent step then records `completed_partial`, and the run ends `completed_partial` with `limit_rounds` or `limit_tokens`. Tools off keeps the tool definitions (a history holding tool calls needs them) and sends each provider's own no-tools choice (Anthropic `tool_choice: none`, OpenAI `tool_choice: "none"`, Gemini `functionCallingConfig.mode: NONE`). Wall-clock: the gateway wraps each tier's attempts in the agent's time limit; a timeout cancels the call and moves to the next tier once, and a second one ends the dispatch with `provider.timeout`; an agent with no routing is cut off at the default. Gap audit: routing `temperature` and `max_tokens` now reach every adapter through `CallOptions` (Anthropic's temperature capped at 1). Tests: 12 in the new `core/tests/agent_loop_limits.rs` (scripted provider, paused clock for the time limits) and 1 adapter wire-format test. Desktop routing form and the demo's routing panel gain the three limits; the demo's Gateway tab gains a step-limit simulator. Help article `agent-limits-and-partial-results` updated.
