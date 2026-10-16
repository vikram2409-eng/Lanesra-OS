---
id: WP-0221
title: Three industry templates with evaluation suites
phase: 2
epic: E2.5 Templates and demo
status: proposed
owner: unassigned
depends_on: [WP-0213, WP-0224]
requirements: [REQ-PG-013]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [playground-templates]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0221 — Three industry templates with evaluation suites

## Goal

Overdue Invoice Recovery, Lead Qualification, Claims First-Notice Intake, each with a 10-case suite.

## Scope

- Templates expressed in today's pipeline topologies (Teams arrive in Phase 3)

## Acceptance criteria

- [ ] AC-1: Each suite passes its gate - this is the Phase 2 gate

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- No paid API keys (ADR-0004): real-model evidence comes from a local Ollama model via WP-0224; CI replays the recorded runs.
- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
