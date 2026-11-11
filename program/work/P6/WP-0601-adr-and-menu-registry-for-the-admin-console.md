---
id: WP-0601
title: ADR and menu registry for the Admin console
phase: 6
epic: E6.1 Admin console
status: proposed
owner: unassigned
planned_for: claude
depends_on: []
requirements: [REQ-ADM-001]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [admin-console]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0601 — ADR and menu registry for the Admin console

## Goal

Fix the ten-area menu as one piece of data that every edition reads, so the grouping is decided once and later phases add tools by adding a row.

## Scope

- ADR: the ten areas in three bands, the home of each of today's 40 tools and of every planned Phase 2 to 5 screen, naming rules, and the previous-location map
- A menu registry (area, tool, description, role, feature flag, previous names) used by the desktop UI and mirrored in `app.js`
- A check that fails CI when a tool is missing from the registry or listed twice
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: The ADR is Accepted and maps every current admin tool and every planned screen to exactly one area
- [ ] AC-2: A registry test fails when a tool has no home, two homes or no description
- [ ] AC-3: Desktop UI and `app.js` produce the same menu from the same registry data (parity test)
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/src/features/settings/__tests__/adminMenuRegistry.test.ts`; `program/tools/` parity check for `app.js`
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Today: `ADMIN_CATEGORIES` in `desktop/src/features/settings/Settings.tsx` (15 cards, 40 tools; Dependency Explorer listed twice).
- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
