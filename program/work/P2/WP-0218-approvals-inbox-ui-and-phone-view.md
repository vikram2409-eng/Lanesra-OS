---
id: WP-0218
title: Approvals inbox UI and phone view
phase: 2
epic: E2.4 Setup and screens
status: proposed
owner: unassigned
agent: muse
depends_on: [WP-0151, WP-0153]
requirements: [REQ-UI-007]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0218 — Approvals inbox UI and phone view

## Goal

Mockup 7 including the one-decision-per-screen phone layout.

## Scope

- Four responses, diff, standing approval

## Acceptance criteria

- [ ] AC-1: E2E at phone viewport

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
