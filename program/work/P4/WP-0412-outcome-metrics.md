---
id: WP-0412
title: Outcome metrics
phase: 4
epic: E4.3 Advanced capabilities
status: proposed
owner: unassigned
depends_on: [WP-0404]
requirements: [REQ-OBS-006]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [observability-dashboards]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0412 — Outcome metrics

## Goal

Business results per run and per dollar.

## Scope


## Acceptance criteria

- [ ] AC-1: Glossary metric linkage tested

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
