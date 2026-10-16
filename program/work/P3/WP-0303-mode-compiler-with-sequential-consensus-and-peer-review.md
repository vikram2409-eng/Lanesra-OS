---
id: WP-0303
title: Mode compiler with Sequential, Consensus and Peer review
phase: 3
epic: E3.1 Teams and compiler
status: proposed
owner: unassigned
depends_on: [WP-0302]
requirements: [REQ-ORCH-001, REQ-ORCH-002, REQ-ORCH-005]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [teams-and-modes]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0303 — Mode compiler with Sequential, Consensus and Peer review

## Goal

Compile modes to graphs, cached per version.

## Scope

- Grows out of graph_from_pipeline

## Acceptance criteria

- [ ] AC-1: Compiled graphs golden-tested

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
