---
id: WP-0650
title: App menu editor and home page per role
phase: 6
epic: E6.6 Navigation and theme
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0615]
requirements: [REQ-EXP-018]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [app-navigation, apps-dashboards]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0650 — App menu editor and home page per role

## Goal

Let an admin decide what users see in the side menu of each app.

## Scope

- Menu editor per app: pages, objects, dashboards and links in groups; visibility by role; home page per role
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: A user sees the menu and home page for their role in the app
- [ ] AC-2: A menu entry to something the user cannot open is not shown
- [ ] AC-3: Menus export and import in a Solution
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/app_navigation.rs`; E2E
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
