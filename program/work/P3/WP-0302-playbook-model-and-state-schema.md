---
id: WP-0302
title: Playbook model and state schema
phase: 3
epic: E3.1 Teams and compiler
status: proposed
owner: unassigned
depends_on: [WP-0301]
requirements: [REQ-CORE-007]
estimate: M
editions: [desktop, workspace]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0302 — Playbook model and state schema

## Goal

Playbook as an execution_graph source with inputs schema and version.

## Scope

- ai_state_schemas

## Acceptance criteria

- [ ] AC-1: Round-trip tests

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
