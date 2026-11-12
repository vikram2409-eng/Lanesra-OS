---
id: WP-0613
title: Carry Screen layouts (forms) into the page model
phase: 6
epic: E6.2 One page model
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0611]
requirements: [REQ-EXP-003]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [experience-studio-pages, screen-app-builder]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0613 — Carry Screen layouts (forms) into the page model

## Goal

One builder for forms and pages without changing any form users see today.

## Scope

- Form page type with tabs, sections, field spans and claimed related lists; automatic carry-over; Screen layouts tool, API and Solution export as aliases with a deprecation notice
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Every existing layout renders the same form after upgrade (snapshot parity per object and role)
- [ ] AC-2: The existing Screen layouts test suite passes unchanged against the aliases
- [ ] AC-3: A Solution exported before the change imports correctly after it
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- Upgrade test with layouts from the last release; parity tests; existing suite
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
