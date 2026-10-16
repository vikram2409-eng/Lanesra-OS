---
id: WP-0306
title: Convert Pipelines to Teams with compatibility aliases
phase: 3
epic: E3.1 Teams and compiler
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0304]
requirements: [REQ-ORCH-020, REQ-ORCH-021, REQ-API-009]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0306 — Convert Pipelines to Teams with compatibility aliases

## Goal

Automatic conversion; aliases keep APIs working.

## Scope

- team_commands.rs replaces ai_agent_pipeline_commands.rs

## Acceptance criteria

- [ ] AC-1: Upgrade test converts fixture pipelines

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
