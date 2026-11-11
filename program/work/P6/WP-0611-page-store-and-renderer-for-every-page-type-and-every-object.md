---
id: WP-0611
title: Page store and renderer for every page type and every object
phase: 6
epic: E6.2 One page model
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0610]
requirements: [REQ-EXP-001, REQ-EXP-002]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [experience-studio-pages, screen-app-builder]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0611 — Page store and renderer for every page type and every object

## Goal

Make a published page work everywhere. Today a published page takes over only Company and Product on desktop.

## Scope

- Page type on the page model; one renderer used by every built-in and custom object and by standalone pages
- Pages tool in Experience Studio replacing the Page Builder entry; role assignment, default, draft, publish, revert
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: A published record page renders for every built-in and custom object in desktop and Team Workspace
- [ ] AC-2: An object with no published page renders exactly as before (parity tests per object)
- [ ] AC-3: The upgrade test carries existing `page_layouts` rows forward unchanged
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/page_model.rs`; `desktop/src/components/__tests__/PageRenderer.test.tsx`; upgrade test; E2E
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR from WP-0610; migration numbering ADR-0003.
- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
