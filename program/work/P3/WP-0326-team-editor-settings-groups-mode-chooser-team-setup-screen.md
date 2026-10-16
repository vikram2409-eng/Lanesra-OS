---
id: WP-0326
title: Team editor: settings groups, mode chooser, team setup screen
phase: 3
epic: E3.5 Team experience
status: proposed
owner: unassigned
depends_on: [WP-0301, WP-0219]
requirements: [REQ-SETUP-011, REQ-SETUP-012, REQ-SETUP-015, REQ-UI-003]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [teams-and-modes, guided-setup]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0326 — Team editor: settings groups, mode chooser, team setup screen

## Goal

Mockup 3.

## Scope


## Acceptance criteria

- [ ] AC-1: E2E and axe

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
