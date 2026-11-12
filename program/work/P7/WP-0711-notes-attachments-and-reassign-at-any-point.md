---
id: WP-0711
title: Notes, attachments and reassign at any point
phase: 7
epic: E7.3 Runner and visibility
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0707]
requirements: [REQ-LANE-018]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-12
---

# WP-0711 — Notes, attachments and reassign at any point

## Goal

Real work needs a few actions outside the scripted steps.

## Scope

- Add note, attach file and reassign the current step from the runner; each audited; attachments follow content labels and access

## Acceptance criteria

- [ ] AC-1: Each action is available at every step and appears in the audit trail
- [ ] AC-2: Reassign respects the step's role
- [ ] AC-3: An attachment is visible only to those who can see the run
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_anytime_actions.rs`; vitest
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
