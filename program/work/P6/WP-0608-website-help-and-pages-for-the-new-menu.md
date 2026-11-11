---
id: WP-0608
title: Website help and pages for the new menu
phase: 6
epic: E6.1 Admin console
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0602]
requirements: [REQ-ADM-009]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [admin-console, workspace-essentials]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0608 — Website help and pages for the new menu

## Goal

Keep every instruction on the website true after the menu changes. The help articles in `app.js` name an "Admin →" path 138 times today.

## Scope

- Rewrite every help navigation path to the new menu; a new "Finding your way around Admin" article with the previous-location map
- Update `/platform`, `/roadmap`, `/releases`, `llms.txt`, `llms-full.txt`, `README.md` and `desktop/README.md`
- A check that fails CI when a help article names a menu path that is not in the registry
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: No help article, llms file or website page names a path that is not in the menu registry (automated check)
- [ ] AC-2: The new article lists every previous location and its new one
- [ ] AC-3: In-product Help links open the updated articles
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `program/tools/` check for help paths against the registry; `node --check app.js`
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
