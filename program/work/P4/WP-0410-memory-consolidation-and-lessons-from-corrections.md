---
id: WP-0410
title: Memory consolidation and lessons from corrections
phase: 4
epic: E4.3 Advanced capabilities
status: proposed
owner: unassigned
depends_on: [WP-0320, WP-0151]
requirements: [REQ-MEM-008, REQ-MEM-010]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0410 — Memory consolidation and lessons from corrections

## Goal

Owner-approved topic notes and lessons.

## Scope


## Acceptance criteria

- [ ] AC-1: Restricted content never extracted

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
