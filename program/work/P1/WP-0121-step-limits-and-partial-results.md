---
id: WP-0121
title: Step limits and partial results
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
depends_on: [WP-0110, WP-0005]
requirements: [REQ-LOOP-001, REQ-LOOP-008, REQ-LOOP-009, REQ-LOOP-010]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0121 — Step limits and partial results

## Goal

Every limit ends gracefully: round limit (default 8, max 25) and token limit ask once for a final answer with tools disabled and mark `completed_partial`; wall-clock limit cancels and retries once on fallback.

## Scope

- Make MAX_ROUNDS configurable per agent within bounds
- Token accounting per step
- Cancellation of provider calls on timeout
- Fallback retry once

## Acceptance criteria

- [ ] AC-1: Each limit tested with the scripted provider: correct final status, final-answer request has no tools
- [ ] AC-2: No limit path panics or exits the process

## Test plan

- `desktop/core/tests/agent_loop_limits.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
