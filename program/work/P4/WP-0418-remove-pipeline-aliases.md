---
id: WP-0418
title: Remove Pipeline aliases
phase: 4
epic: E4.1 Scale
status: proposed
owner: unassigned
depends_on: [WP-0306]
requirements: [REQ-ORCH-022]
estimate: S
branch: 
pr: 
updated: 2026-10-16
---

# WP-0418 — Remove Pipeline aliases

## Goal

Two releases after Teams ship, after a deprecation notice.

## Scope


## Acceptance criteria

- [ ] AC-1: Pipeline tables read-only

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
