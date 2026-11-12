---
id: WP-0625
title: Six starter blocks
phase: 6
epic: E6.3 Components and blocks
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0624, WP-0620]
requirements: [REQ-EXP-011]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [reusable-blocks]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0625 — Six starter blocks

## Goal

Give admins useful blocks on day one.

## Scope

- Record summary, my open work, approval strip, KPI row, activity timeline, agent chat; AI blocks appear once WP-0622 has shipped
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Each starter block inserts and binds to the chosen object without further setup
- [ ] AC-2: Starter blocks are copies an admin can edit; updating the product never overwrites an edited one
- [ ] AC-3: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-4: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-5: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest for each block definition
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- The approval strip and agent chat blocks are added after WP-0622; record that in the Progress log.
- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
