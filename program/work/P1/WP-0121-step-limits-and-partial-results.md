---
id: WP-0121
title: Step limits and partial results
phase: 1
epic: E1.3 Agent loop
status: ready
owner: unassigned
depends_on: [WP-0110, WP-0005]
requirements: [REQ-LOOP-001, REQ-LOOP-008, REQ-LOOP-009, REQ-LOOP-010]
threats: [T-06]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-limits-and-partial-results]
branch: 
pr: 
updated: 2026-10-19
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

- [ ] AC-1: Each limit tested with the scripted provider: correct final status, final-answer request has no tools
- [ ] AC-2: No limit path panics or exits the process

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
