---
id: WP-0405
title: Kill switch and four-eyes rule
phase: 4
epic: E4.2 Compliance and governance
status: proposed
owner: unassigned
depends_on: [WP-0304, WP-0151]
requirements: [REQ-GOV-009, REQ-GOV-010]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0405 — Kill switch and four-eyes rule

## Goal

Instant pause of a team or workspace; starter cannot approve own privileged steps.

## Scope


## Acceptance criteria

- [ ] AC-1: Resume clean

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
