---
id: WP-0125
title: Context window handling and compaction policies
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
depends_on: [WP-0121]
requirements: [REQ-LOOP-013, REQ-LOOP-018, REQ-LOOP-019]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0125 — Context window handling and compaction policies

## Goal

When history grows, shrink it by the agent's compaction policy (drop oldest, recent window, prune tool calls, shorten tool results, head-and-tail) and never fail on overflow.

## Scope

- Token estimator per provider
- Five strategies, default shorten-then-window
- Tokens saved recorded on the span

## Acceptance criteria

- [ ] AC-1: Each strategy has a golden test on a fixed transcript
- [ ] AC-2: Head-and-tail keeps the original brief
- [ ] AC-3: Overflow beyond all strategies summarises, never errors

## Test plan

- `desktop/core/tests/context_compaction.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
