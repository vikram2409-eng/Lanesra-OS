---
id: WP-0126
title: Budget awareness in agent turns
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
depends_on: [WP-0121]
requirements: [REQ-LOOP-020]
estimate: S
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-limits-and-partial-results]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0126 — Budget awareness in agent turns

## Goal

Tell the agent each turn how many steps, tokens and dollars remain so it wraps up gracefully.

## Scope

- Budget line in the compiled turn context, worded via the prompt catalog

## Acceptance criteria

- [ ] AC-1: Remaining values are correct each round (tested)

## Test plan

- Extend `agent_loop_limits.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0008 trace and events; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
