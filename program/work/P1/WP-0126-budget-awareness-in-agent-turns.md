---
id: WP-0126
title: Budget awareness in agent turns
phase: 1
epic: E1.3 Agent loop
status: in_review
owner: claude
depends_on: [WP-0121]
requirements: [REQ-LOOP-020]
estimate: S
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [agent-limits-and-partial-results]
branch: claude/enterprise-build-planning-jxy2kq
pr: 723
updated: 2026-10-24
---

# WP-0126 — Budget awareness in agent turns

## Goal

Tell the agent each turn how many steps, tokens and dollars remain so it wraps up gracefully. Dollars need model prices (WP-0146), so they are split into WP-0199; this package covers steps (tool-calling rounds) and tokens.

## Scope

- Budget line in the compiled turn context, worded via the prompt catalog

## Acceptance criteria

- [x] AC-1: Remaining rounds, and remaining tokens when the agent has a run token limit, are correct each round (tested)

## Test plan

- Extend `agent_loop_limits.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0008 trace and events; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-24 claude: Refined proposed → ready and claimed. Dependencies met (WP-0121 done); ADR-0007, ADR-0008 and ADR-0011 Accepted. Dollars need prices from the model catalog (WP-0146), so they move to WP-0199 (proposed).
- 2026-10-24 claude: Built. Each round of the agent loop sends the run's own system prompt plus catalog prompt `loop/budget-line`, which states the rounds left of the limit and, when the agent has a run token limit, the tokens left. The final-answer call keeps its own instruction. Two tests in `core/tests/agent_loop_limits.rs` check the exact values each round, with and without a token limit. The help article and the demo's step-limit simulator show it.
- 2026-10-24 claude: In review in #723.
