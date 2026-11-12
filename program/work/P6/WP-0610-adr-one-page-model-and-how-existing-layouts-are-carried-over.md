---
id: WP-0610
title: ADR: one page model and how existing layouts are carried over
phase: 6
epic: E6.2 One page model
status: proposed
owner: unassigned
planned_for: claude
depends_on: []
requirements: [REQ-EXP-001]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [experience-studio-pages]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0610 — ADR: one page model and how existing layouts are carried over

## Goal

Decide the page model before three editors are merged, so nothing users built is lost.

## Scope

- ADR: page types, definition format (extends `PageDefinition`), data sources and parameters, block references, how `screen_layouts`, `page_layouts` and `dashboard_layouts` are carried over, aliases and their removal, the shared format with `app.js` (ADR-0015)
- Component registry rules: one allowlist on the server, one library on the client
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: The ADR is Accepted and covers every field of the three existing layout tables with its new home
- [ ] AC-2: The ADR states the alias and deprecation rule for the Screen layouts and Dashboards tools and APIs
- [ ] AC-3: A JSON Schema for the page definition is committed and shared with the demo
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `program/tools/programctl.py validate`; schema lint in CI
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Today: `page_layout_service.rs` ("a second system, not an evolution of the first"), `screen_layout_service`, `DashboardLayoutsAdmin.tsx`.
- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
