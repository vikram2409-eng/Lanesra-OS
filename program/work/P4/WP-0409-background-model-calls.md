---
id: WP-0409
title: Background model calls
phase: 4
epic: E4.3 Advanced capabilities
status: proposed
owner: unassigned
depends_on: [WP-0304]
requirements: [REQ-ORCH-025]
estimate: M
editions: [desktop, workspace]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0409 — Background model calls

## Goal

waiting_model status that survives restart.

## Scope


## Acceptance criteria

- [ ] AC-1: Restart mid-call resumes

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
