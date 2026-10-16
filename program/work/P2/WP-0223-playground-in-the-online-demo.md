---
id: WP-0223
title: Playground in the online demo
phase: 2
epic: E2.5 Templates and demo
status: proposed
owner: unassigned
agent: muse
depends_on: [WP-0013, WP-0203]
requirements: [REQ-PG-016]
estimate: XL
branch: 
pr: 
updated: 2026-10-16
---

# WP-0223 — Playground in the online demo

## Goal

Simulated runs on sample data with write tools always simulated, per the demo ADR.

## Scope

- Shared Team format

## Acceptance criteria

- [ ] AC-1: Demo E2E covers a template run

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
