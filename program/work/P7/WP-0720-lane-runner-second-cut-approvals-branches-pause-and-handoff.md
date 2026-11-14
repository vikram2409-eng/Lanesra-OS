---
id: WP-0720
title: Lane runner, second cut: approvals, branches, pause and handoff
phase: 7
epic: E7.3 Runner and visibility
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0707, WP-0706]
requirements: [REQ-LANE-014, REQ-LANE-012, REQ-LANE-009]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-14
---

# WP-0720 — Lane runner, second cut: approvals, branches, pause and handoff

## Goal

Finish the runner for everything that is not the straight path, so a discrepancy, a hold and a handoff can be worked on the same screen in the product as in the demo.

## Scope

- Approval steps on the runner: who it waits on, the decision and its effect on the route
- Branch stages shown distinctly in the stage bar; the declared way out of a hold or rejection
- Pause, resume, handoff, reopen within the current stage, cancel with the list of posted writes
- The online demo runner aligned with this screen (same parts, same wording); this ends the demo freeze of ADR-0025

## Acceptance criteria

- [ ] AC-1: The Receive Shipment discrepancy case runs end to end on the product runner: review, approve or hold, and the declared way out
- [ ] AC-2: Pause, resume and handoff work from the runner and are audited; cancel lists the writes already posted
- [ ] AC-3: A branch stage is visibly distinct in the stage bar and readable by a screen reader
- [ ] AC-4: The demo runner and the product runner show the same parts and wording for the same lane (parity walkthrough)
- [ ] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-6: Website help (`/help/process-lanes`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest; Playwright E2E in both modes with the scripted provider; axe checks
- Online demo: `node --check app.js` and the demo E2E for the aligned runner
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Split from WP-0707 by ADR-0025 decision 1. ADR-0023, ADR-0024; design in SOLUTION-DESIGN.md section 17. Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
