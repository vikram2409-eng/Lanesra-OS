---
id: WP-0205
title: Step-through
phase: 2
epic: E2.2 Run control
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0204]
requirements: [REQ-PG-006]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0205 — Step-through

## Goal

Pause after each step, inspect and edit output, continue.

## Scope

- Engine pause hook and UI

## Acceptance criteria

- [ ] AC-1: Edited output flows downstream

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
