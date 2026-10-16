---
id: WP-0321
title: Team drive
phase: 3
epic: E3.4 Identity, budget and memory
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0301]
requirements: [REQ-MEM-009]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0321 — Team drive

## Goal

Versioned, labelled team files.

## Scope

- ai_team_files

## Acceptance criteria

- [ ] AC-1: Search time limit enforced

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
