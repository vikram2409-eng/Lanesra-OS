---
id: WP-0206
title: Simulate writes with shadow transactions and diffs
phase: 2
epic: E2.2 Run control
status: proposed
owner: unassigned
depends_on: [WP-0141, WP-0203]
requirements: [REQ-TOOL-006, REQ-PG-007]
estimate: XL
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-playground]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0206 — Simulate writes with shadow transactions and diffs

## Goal

Write-tier and above tools run in a rollback transaction with a before/after diff per record.

## Scope

- Shadow transaction design for native, connector (mocked) and MCP (blocked or mocked) writes

## Acceptance criteria

- [ ] AC-1: Nothing persists after a simulated run (DB diff test)

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
