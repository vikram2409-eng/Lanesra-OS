---
id: WP-0327
title: Work steps canvas
phase: 3
epic: E3.5 Team experience
status: proposed
owner: unassigned
depends_on: [WP-0311, WP-0326]
requirements: [REQ-SETUP-013, REQ-UI-004]
estimate: XL
editions: [desktop, workspace, demo]
delivered: []
docs: [graph-mode-and-canvas]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0327 — Work steps canvas

## Goal

Mockup 4 with in-place validation.

## Scope

- Five-tab step panel

## Acceptance criteria

- [ ] AC-1: E2E build-and-run on canvas

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
