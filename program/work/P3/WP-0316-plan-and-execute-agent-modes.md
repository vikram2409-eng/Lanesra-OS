---
id: WP-0316
title: Plan and Execute agent modes
phase: 3
epic: E3.2 Modes
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0304]
requirements: [REQ-ORCH-024]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0316 — Plan and Execute agent modes

## Goal

Plan mode is read-only; Execute needs approval.

## Scope


## Acceptance criteria

- [ ] AC-1: Write tool refused in Plan mode

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
