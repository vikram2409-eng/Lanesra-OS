---
id: WP-0718
title: Visual lane builder
phase: 7
epic: E7.5 Authoring
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0712, WP-0714]
requirements: [REQ-LANE-024]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-14
---

# WP-0718 — Visual lane builder

## Goal

Let admins compose lanes without editing data.

## Scope

- Stages and steps on a canvas, a step panel per kind, branches drawn as connections, live publish checks; produces the same definition format; reuses the graph canvas

## Acceptance criteria

- [ ] AC-1: A lane built in the builder equals the same lane written as data
- [ ] AC-2: Every publish check is shown while editing
- [ ] AC-3: The builder is keyboard operable and passes axe checks
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest; Playwright E2E
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
