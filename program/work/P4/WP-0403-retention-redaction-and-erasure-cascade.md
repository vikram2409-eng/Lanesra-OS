---
id: WP-0403
title: Retention, redaction and erasure cascade
phase: 4
epic: E4.2 Compliance and governance
status: proposed
owner: unassigned
depends_on: [WP-0402, WP-0320]
requirements: [REQ-GOV-017, REQ-MEM-006]
estimate: L
editions: [desktop, workspace]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0403 — Retention, redaction and erasure cascade

## Goal

7-year tool audit, 90-day outputs, run deletion cascades to memories.

## Scope


## Acceptance criteria

- [ ] AC-1: Erasure entry recorded

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
