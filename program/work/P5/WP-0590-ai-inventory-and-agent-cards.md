---
id: WP-0590
title: AI inventory and agent cards
phase: 5
epic: E5.10 AI compliance
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0501, WP-0402]
requirements: [REQ-COMP-001]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [ai-compliance]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0590 — AI inventory and agent cards

## Goal

Answer "what AI do we run, and who owns it" from live data.

## Scope

- Inventory view and export; generated agent card per published version

## Acceptance criteria

- [ ] AC-1: The inventory lists every agent, Team, model, tool and external agent with an owner
- [ ] AC-2: An agent card regenerates on publish and is kept with the version

## Test plan

- `desktop/core/tests/ai_inventory.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
