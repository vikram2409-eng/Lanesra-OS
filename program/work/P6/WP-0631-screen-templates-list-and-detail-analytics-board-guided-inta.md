---
id: WP-0631
title: Screen templates: List and Detail, Analytics Board, Guided Intake
phase: 6
epic: E6.4 Layouts and templates
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0630, WP-0620, WP-0621]
requirements: [REQ-EXP-013]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [page-templates]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0631 — Screen templates: List and Detail, Analytics Board, Guided Intake

## Goal

Three complete screens an admin can publish in minutes, with no AI feature needed.

## Scope

- Three object-aware templates with components placed and bound; thumbnails; the four existing record-page templates kept
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Each template produces a valid, publishable page for an object with few fields and no relationships
- [ ] AC-2: A page made from a template is independent of it
- [ ] AC-3: Each template passes the pre-publish checks at all three widths
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/src/features/settings/__tests__/pageTemplates.test.ts` extended
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
