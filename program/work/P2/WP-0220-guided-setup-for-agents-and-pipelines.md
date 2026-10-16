---
id: WP-0220
title: Guided Setup for agents and pipelines
phase: 2
epic: E2.4 Setup and screens
status: proposed
owner: unassigned
depends_on: [WP-0219, WP-0144]
requirements: [REQ-SETUP-006, REQ-SETUP-007, REQ-SETUP-008, REQ-SETUP-009, REQ-SETUP-010]
estimate: XL
editions: [desktop, workspace, demo]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0220 — Guided Setup for agents and pipelines

## Goal

Side-panel guide as a lens over the real object, with progress rail and three ways to start.

## Scope

- ai_setup_progress
- Describe it drafting via the admin assistant

## Acceptance criteria

- [ ] AC-1: Status computed from object state only

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
