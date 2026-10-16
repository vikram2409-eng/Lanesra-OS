---
id: WP-0202
title: Run REST endpoints: get, trace, cancel, pause, resume
phase: 2
epic: E2.2 Run control
status: proposed
owner: unassigned
depends_on: [WP-0112]
requirements: [REQ-API-003]
estimate: M
editions: [desktop, workspace]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0202 — Run REST endpoints: get, trace, cancel, pause, resume

## Goal

Public run API used by the Playground, CLI and MCP.

## Scope

- Endpoints, MCP tools, CLI commands

## Acceptance criteria

- [ ] AC-1: OpenAPI documented; authz tested

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
