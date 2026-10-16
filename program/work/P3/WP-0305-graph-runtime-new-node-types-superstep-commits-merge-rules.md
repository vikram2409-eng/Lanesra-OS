---
id: WP-0305
title: Graph runtime: new node types, superstep commits, merge rules
phase: 3
epic: E3.3 Graph and flow control
status: proposed
owner: unassigned
depends_on: [WP-0303]
requirements: [REQ-ORCH-004, REQ-ORCH-015, REQ-ORCH-017, REQ-ORCH-027]
estimate: XL
editions: [desktop, workspace]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0305 — Graph runtime: new node types, superstep commits, merge rules

## Goal

plan, selector, handoff, judge, mission nodes; staged parallel writes with per-field merge rules.

## Scope

- Logically-parallel marking on desktop

## Acceptance criteria

- [ ] AC-1: Conflict detection tests

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
