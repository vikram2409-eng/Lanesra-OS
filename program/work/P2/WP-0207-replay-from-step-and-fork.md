---
id: WP-0207
title: Replay from step and fork
phase: 2
epic: E2.2 Run control
status: proposed
owner: unassigned
depends_on: [WP-0111, WP-0203]
requirements: [REQ-ORCH-018, REQ-ORCH-019, REQ-PG-009]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0207 — Replay from step and fork

## Goal

Re-run a step and its downstream; fork a run into a Playground session; outputs immutable.

## Scope

- Lineage links parent/child runs

## Acceptance criteria

- [ ] AC-1: Upstream outputs reused byte-identical

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
