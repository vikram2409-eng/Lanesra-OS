---
id: WP-0651
title: Themes per app, light and dark pairs, component styles
phase: 6
epic: E6.6 Navigation and theme
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0611]
requirements: [REQ-EXP-019]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [theme-studio]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0651 — Themes per app, light and dark pairs, component styles

## Goal

Brand each app and support dark mode properly.

## Scope

- Theme assignment per app; paired light and dark values per token; density and corner tokens; contrast gate on both modes
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Publish is blocked when either mode fails the contrast check
- [ ] AC-2: An app without its own theme uses the workspace theme
- [ ] AC-3: Existing published themes carry over unchanged
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/theme_service.rs` extended; vitest; upgrade test
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
