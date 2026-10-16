---
id: WP-0214
title: Explain this, tool cards and inline citations
phase: 2
epic: E2.1 Playground core
status: proposed
owner: unassigned
depends_on: [WP-0204]
requirements: [REQ-PG-012, REQ-TOOL-003, REQ-MEM-007]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-playground]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0214 — Explain this, tool cards and inline citations

## Goal

Plain-language explanations and tool cards in the Playground.

## Scope

- Explanations built from trace data, not a fresh model guess, where possible

## Acceptance criteria

- [ ] AC-1: Explanations cite trace items

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
