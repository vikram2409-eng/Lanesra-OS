---
id: WP-0407
title: Missions
phase: 4
epic: E4.3 Advanced capabilities
status: proposed
owner: unassigned
depends_on: [WP-0305]
requirements: [REQ-ORCH-043]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0407 — Missions

## Goal

Long-running runs with todo list and sub-runs.

## Scope

- ai_missions

## Acceptance criteria

- [ ] AC-1: Stops on budget

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
