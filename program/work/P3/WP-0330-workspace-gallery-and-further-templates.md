---
id: WP-0330
title: Workspace gallery and further templates
phase: 3
epic: E3.5 Team experience
status: proposed
owner: unassigned
depends_on: [WP-0221, WP-0301]
requirements: [REQ-API-013, REQ-PG-014]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [playground-templates]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0330 — Workspace gallery and further templates

## Goal

Reuse approved components; more industry templates.

## Scope


## Acceptance criteria

- [ ] AC-1: Each template passes its suite

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
