---
id: WP-0204
title: Trace pane, stream levels and resumable streams
phase: 2
epic: E2.1 Playground core
status: proposed
owner: unassigned
agent: muse
depends_on: [WP-0203]
requirements: [REQ-PG-004, REQ-PG-018]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0204 — Trace pane, stream levels and resumable streams

## Goal

Live trace timeline with payloads and context receipts.

## Scope

- Four stream levels, tool progress events, resume after tab reopen

## Acceptance criteria

- [ ] AC-1: No events lost across reconnect

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
