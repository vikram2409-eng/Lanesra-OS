---
id: WP-0320
title: Composite memory recall, team and run scopes, auto-extraction
phase: 3
epic: E3.4 Identity, budget and memory
status: proposed
owner: unassigned
depends_on: [WP-0301]
requirements: [REQ-MEM-001, REQ-MEM-002, REQ-MEM-003, REQ-MEM-004, REQ-MEM-005]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0320 — Composite memory recall, team and run scopes, auto-extraction

## Goal

Ranked recall with explanations; new scopes; policy-gated extraction.

## Scope


## Acceptance criteria

- [ ] AC-1: Ranking golden tests

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
