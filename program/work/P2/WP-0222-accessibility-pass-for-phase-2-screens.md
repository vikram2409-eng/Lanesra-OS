---
id: WP-0222
title: Accessibility pass for Phase 2 screens
phase: 2
epic: E2.4 Setup and screens
status: proposed
owner: unassigned
depends_on: [WP-0203, WP-0217, WP-0218]
requirements: [REQ-PG-015]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-playground]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0222 — Accessibility pass for Phase 2 screens

## Goal

Keyboard, screen reader and WCAG AA for every Phase 2 screen.

## Scope

- Audit and fixes

## Acceptance criteria

- [ ] AC-1: Axe clean; manual screen-reader script recorded

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
