---
id: WP-0208
title: Edit state and resume; breakpoints on live runs
phase: 2
epic: E2.2 Run control
status: proposed
owner: unassigned
depends_on: [WP-0207, WP-0201]
requirements: [REQ-ORCH-044, REQ-ORCH-045]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-playground]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0208 — Edit state and resume; breakpoints on live runs

## Goal

Correct a field at a checkpoint and resume as a branch; admin breakpoints.

## Scope

- Audited edits
- Breakpoints with 24h expiry

## Acceptance criteria

- [ ] AC-1: Original branch intact; breakpoint expiry tested

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
