---
id: WP-0603
title: Admin Home with attention counts
phase: 6
epic: E6.1 Admin console
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0602]
requirements: [REQ-ADM-005]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [admin-console, admin-status-strip]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0603 — Admin Home with attention counts

## Goal

Make the first screen tell an admin what needs them today.

## Scope

- Needs-attention list and a count per menu area from `admin_home_service`; setup progress; pinned and recent; recent changes
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Each count equals the number of items behind it and opens the matching filtered list
- [ ] AC-2: A clean workspace shows no attention panel
- [ ] AC-3: Setup progress hides once complete
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/admin_home_summary.rs`; vitest for the home screen
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
