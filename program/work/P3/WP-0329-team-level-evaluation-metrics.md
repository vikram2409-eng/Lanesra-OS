---
id: WP-0329
title: Team-level evaluation metrics
phase: 3
epic: E3.5 Team experience
status: proposed
owner: unassigned
depends_on: [WP-0212, WP-0304]
requirements: [REQ-EVAL-003]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [evaluation-harness]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0329 — Team-level evaluation metrics

## Goal

Task success, steps, re-plans, delegations, cost per success.

## Scope


## Acceptance criteria

- [ ] AC-1: Metrics computed from traces

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
