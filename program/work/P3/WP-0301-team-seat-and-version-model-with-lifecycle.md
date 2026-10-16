---
id: WP-0301
title: Team, seat and version model with lifecycle
phase: 3
epic: E3.1 Teams and compiler
status: proposed
owner: unassigned
depends_on: [WP-0131]
requirements: [REQ-CORE-005, REQ-CORE-006, REQ-CORE-009]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0301 — Team, seat and version model with lifecycle

## Goal

ai_teams, ai_team_seats, ai_team_versions.

## Scope

- Draft → Published lifecycle, immutable versions

## Acceptance criteria

- [ ] AC-1: Seat swap keeps playbook intact

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
