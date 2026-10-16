---
id: WP-0211
title: Save as test case, golden cases and graders
phase: 2
epic: E2.3 Evaluation and publish
status: proposed
owner: unassigned
depends_on: [WP-0210]
requirements: [REQ-PG-010, REQ-EVAL-001, REQ-EVAL-002]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [evaluation-harness]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0211 — Save as test case, golden cases and graders

## Goal

One-click golden case from a run; graders incl. tool-path and cost/latency ceilings.

## Scope

- Extend ai_eval_service

## Acceptance criteria

- [ ] AC-1: Each grader has pass/fail tests

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
