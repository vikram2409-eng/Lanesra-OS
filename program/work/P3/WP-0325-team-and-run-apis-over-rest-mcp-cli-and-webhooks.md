---
id: WP-0325
title: Team and run APIs over REST, MCP, CLI and webhooks
phase: 3
epic: E3.5 Team experience
status: proposed
owner: unassigned
depends_on: [WP-0304]
requirements: [REQ-API-002, REQ-API-006, REQ-API-007]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0325 — Team and run APIs over REST, MCP, CLI and webhooks

## Goal

Every endpoint on all three surfaces.

## Scope


## Acceptance criteria

- [ ] AC-1: OpenAPI + MCP tool list + CLI parity test

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
