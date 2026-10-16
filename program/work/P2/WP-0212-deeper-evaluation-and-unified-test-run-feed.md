---
id: WP-0212
title: Deeper evaluation and Unified Test Run feed
phase: 2
epic: E2.3 Evaluation and publish
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0211]
requirements: [REQ-EVAL-007, REQ-EVAL-008]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0212 — Deeper evaluation and Unified Test Run feed

## Goal

N repeats for variance, per-seat scores, tool-argument checks.

## Scope

- Feed results to test_eval_service

## Acceptance criteria

- [ ] AC-1: Variance computed correctly

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
