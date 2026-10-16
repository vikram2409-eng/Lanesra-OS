---
id: WP-0317
title: Durability per Team and Team variants
phase: 3
epic: E3.1 Teams and compiler
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0304]
requirements: [REQ-ORCH-032, REQ-ORCH-035]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0317 — Durability per Team and Team variants

## Goal

Choose save cadence; named settings-only variants.

## Scope

- ai_team_variants

## Acceptance criteria

- [ ] AC-1: Writing teams forced to save-every-step

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
