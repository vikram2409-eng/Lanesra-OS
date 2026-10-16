---
id: WP-0210
title: Recorded runs and deterministic replay
phase: 2
epic: E2.3 Evaluation and publish
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0005, WP-0111]
requirements: [REQ-EVAL-009]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0210 — Recorded runs and deterministic replay

## Goal

Save a run with model responses; suites replay at no model cost with identical results.

## Scope

- Builds on the scripted provider

## Acceptance criteria

- [ ] AC-1: Replay is byte-identical

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
