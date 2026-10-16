---
id: WP-0215
title: Interactive output cards
phase: 2
epic: E2.1 Playground core
status: proposed
owner: unassigned
depends_on: [WP-0203]
requirements: [REQ-PG-017]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0215 — Interactive output cards

## Goal

Agents return schema-checked cards; actions go through the normal write path.

## Scope

- Five card types

## Acceptance criteria

- [ ] AC-1: Card actions respect permissions and approvals

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
