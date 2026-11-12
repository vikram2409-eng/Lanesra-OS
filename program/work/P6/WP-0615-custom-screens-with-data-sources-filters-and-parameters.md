---
id: WP-0615
title: Custom screens with data sources, filters and parameters
phase: 6
epic: E6.2 One page model
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0611]
requirements: [REQ-EXP-005]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [experience-studio-pages]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0615 — Custom screens with data sources, filters and parameters

## Goal

Let an admin build a screen that is not tied to one record: a home page, a list page, a console.

## Scope

- Standalone page types (list, home, console); page-level data sources (object, saved view, report, Glossary metric), filters and parameters; role assignment
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: An admin builds and publishes a standalone page and a user of the assigned role opens it
- [ ] AC-2: A data source returns only records the viewing user may read (two-user test)
- [ ] AC-3: A page whose source is deleted shows a clear error to the admin and an empty state to users, never a crash
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/custom_pages.rs`; E2E
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
