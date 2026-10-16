---
id: WP-0213
title: Publish gate, Ready to publish page and impact analysis
phase: 2
epic: E2.3 Evaluation and publish
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0212]
requirements: [REQ-PG-011, REQ-EVAL-004, REQ-EVAL-010, REQ-UI-008]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0213 — Publish gate, Ready to publish page and impact analysis

## Goal

Publish runs the suite and blocks below threshold; impact list from the System Graph.

## Scope

- Mockup 8 without shadow (Phase 4)

## Acceptance criteria

- [ ] AC-1: Publish blocked at 89%, allowed at 90%

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
