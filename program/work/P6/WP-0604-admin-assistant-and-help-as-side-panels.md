---
id: WP-0604
title: Admin Assistant and Help as side panels
phase: 6
epic: E6.1 Admin console
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0602]
requirements: [REQ-ADM-006]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [admin-console]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0604 — Admin Assistant and Help as side panels

## Goal

Keep help and the assistant beside the work instead of in separate areas.

## Scope

- Docked panels on every admin screen; Help opens on the article registered for the screen; the Assistant knows which screen is open
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Help opens on the right article for at least every tool in the registry
- [ ] AC-2: The Assistant panel keeps its conversation while the admin moves between tools
- [ ] AC-3: Both panels are keyboard reachable and pass axe checks
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest and E2E for both panels
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
