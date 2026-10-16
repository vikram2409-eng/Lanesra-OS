---
id: WP-0318
title: Agent Identities and identity-scoped runs
phase: 3
epic: E3.4 Identity, budget and memory
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0104, WP-0304]
requirements: [REQ-GOV-002, REQ-GOV-003, REQ-GOV-005]
estimate: XL
branch: 
pr: 
updated: 2026-10-16
---

# WP-0318 — Agent Identities and identity-scoped runs

## Goal

Service principals for unattended runs; interactive runs never exceed the user; Team Owner and Auditor roles.

## Scope

- ai_agent_identities

## Acceptance criteria

- [ ] AC-1: Access matrix tests; agent never sees more than its user

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
