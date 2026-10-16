---
id: WP-0413
title: Delta checkpoints and identity-aware step cache
phase: 4
epic: E4.1 Scale
status: proposed
owner: unassigned
depends_on: [WP-0305, WP-0318]
requirements: [REQ-ORCH-031, REQ-MEM-011]
estimate: L
editions: [desktop, workspace]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0413 — Delta checkpoints and identity-aware step cache

## Goal

Smaller checkpoints; cache that never leaks across users.

## Scope


## Acceptance criteria

- [ ] AC-1: Cache key includes identity scope and data version

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
