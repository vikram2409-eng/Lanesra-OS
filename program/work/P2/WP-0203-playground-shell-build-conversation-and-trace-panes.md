---
id: WP-0203
title: Playground shell: Build, Conversation and Trace panes
phase: 2
epic: E2.1 Playground core
status: proposed
owner: unassigned
depends_on: [WP-0202, WP-0160, WP-0201]
requirements: [REQ-PG-001, REQ-PG-002, REQ-PG-003, REQ-UI-005, REQ-UI-009]
estimate: XL
editions: [desktop, workspace, demo]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0203 — Playground shell: Build, Conversation and Trace panes

## Goal

The three-pane Playground per mockup 5, opening any run read-only.

## Scope

- Panes, record picker, input form from schema, unsaved edits scoped to Playground runs

## Acceptance criteria

- [ ] AC-1: Matches mockup tokens in light and dark; keyboard navigable

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
