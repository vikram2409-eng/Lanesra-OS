---
id: WP-0312
title: Safe expression language
phase: 3
epic: E3.3 Graph and flow control
status: proposed
owner: unassigned
depends_on: [WP-0302]
requirements: [REQ-ORCH-026]
estimate: L
editions: [desktop, workspace]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0312 — Safe expression language

## Goal

One expression language shared with Business Rules with hard limits.

## Scope

- Fuzz-tested evaluator

## Acceptance criteria

- [ ] AC-1: Limits enforced; fuzz test runs nightly

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
