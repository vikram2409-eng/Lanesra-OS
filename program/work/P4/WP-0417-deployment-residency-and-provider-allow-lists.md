---
id: WP-0417
title: Deployment residency and provider allow-lists
phase: 4
epic: E4.2 Compliance and governance
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0123]
requirements: [REQ-GOV-019]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0417 — Deployment residency and provider allow-lists

## Goal

Region pinning and per-workspace provider allow-lists; offline desktop verified.

## Scope


## Acceptance criteria

- [ ] AC-1: No outbound telemetry test

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
