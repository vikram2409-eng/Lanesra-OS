---
id: WP-0216
title: Record lineage on activity timelines
phase: 2
epic: E2.2 Run control
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0111]
requirements: [REQ-GOV-027]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0216 — Record lineage on activity timelines

## Goal

Every agent change appears on the record's own timeline linked to run, step, version and approver.

## Scope

- Activity entries

## Acceptance criteria

- [ ] AC-1: Lineage present for native and Action Type writes

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
