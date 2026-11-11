---
id: WP-0623
title: Work queue component
phase: 6
epic: E6.3 Components and blocks
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0622, WP-0560]
requirements: [REQ-EXP-009]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [page-components]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0623 — Work queue component

## Goal

Show human-seat work items on any page.

## Scope

- Work queue component over human-seat work items with claim and open
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: The component lists only items the viewer may claim
- [ ] AC-2: Claim and open behave as in the work queue screen
- [ ] AC-3: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-4: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-5: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/page_work_queue_component.rs`
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
