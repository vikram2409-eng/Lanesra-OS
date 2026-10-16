---
id: WP-0313
title: Event Playbooks, triggers and busy-run policy
phase: 3
epic: E3.3 Graph and flow control
status: proposed
owner: unassigned
depends_on: [WP-0304]
requirements: [REQ-ORCH-013, REQ-ORCH-034]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [event-playbooks]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0313 — Event Playbooks, triggers and busy-run policy

## Goal

Record-event, schedule and webhook triggers; busy-run handling.

## Scope

- Duplicate event merging

## Acceptance criteria

- [ ] AC-1: Each busy policy tested

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
