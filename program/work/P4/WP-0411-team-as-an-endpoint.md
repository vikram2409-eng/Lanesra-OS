---
id: WP-0411
title: Team as an endpoint
phase: 4
epic: E4.3 Advanced capabilities
status: proposed
owner: unassigned
depends_on: [WP-0325]
requirements: [REQ-API-008]
estimate: M
editions: [desktop, workspace]
delivered: []
editions_reason: a server API; the online demo has no server
docs: [team-endpoints]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0411 — Team as an endpoint

## Goal

OpenAI-compatible chat endpoint and MCP tool per published Team.

## Scope


## Acceptance criteria

- [ ] AC-1: Scoped keys enforced

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
