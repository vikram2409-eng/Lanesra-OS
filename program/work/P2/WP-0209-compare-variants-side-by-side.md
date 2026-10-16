---
id: WP-0209
title: Compare variants side by side
phase: 2
epic: E2.1 Playground core
status: proposed
owner: unassigned
depends_on: [WP-0203]
requirements: [REQ-PG-008]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0209 — Compare variants side by side

## Goal

Run the same input on two variants with output, cost and latency diff.

## Scope

- Variant definition and dual run

## Acceptance criteria

- [ ] AC-1: Diff correctness tests

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
