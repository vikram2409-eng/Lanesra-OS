---
id: WP-0612
title: Finish the 24 existing components
phase: 6
epic: E6.2 One page model
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0610]
requirements: [REQ-EXP-006]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [page-components]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0612 — Finish the 24 existing components

## Goal

Remove every "not wired to live data yet" placeholder before adding new components.

## Scope

- KPI field-sum and field-count sources; chart over related data and saved reports; button, command bar, quick action (Action Types) and agent action through the tool firewall
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Each of the 24 components renders real data or performs its action; none shows a placeholder
- [ ] AC-2: An action a Business Rule blocks is blocked from the page too
- [ ] AC-3: An agent action is refused, approved or run exactly as the same call from the assistant would be
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/page_components_live.rs`; vitest per component
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
