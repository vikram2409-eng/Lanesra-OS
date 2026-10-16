---
id: WP-0901
title: Open interop: A2A, AG-UI and YAML export
phase: v2
epic: Ev2 Open interop
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0419]
requirements: [REQ-API-010, REQ-API-011, REQ-API-012]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0901 — Open interop: A2A, AG-UI and YAML export

## Goal

Post-GA interop.

## Scope


## Acceptance criteria

- [ ] AC-1: Protocol conformance tests

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
