---
id: WP-0721
title: "Blueprint view redesign: a visual, modern process map of states and steps"
phase: 7
epic: E7.5 Authoring
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0712]
requirements: [REQ-LANE-020]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch:
pr:
updated: 2026-11-18
---

# WP-0721 — Blueprint view redesign: a visual, modern process map of states and steps

## Goal

Make the lane Blueprint read at a glance as a real process map, not a list.
Today's Blueprint (WP-0712) shows a lane's stages and steps correctly but
plainly; this redesign gives it a visually appealing, modern treatment on the
level of a Pega Blueprint - the **process states (stages)** laid out as a clear
left-to-right flow, the **process steps** grouped under each state, with step
kind, role and branches legible at a glance. Same data, same export; a better
picture of it.

## Scope

- Redesign the Blueprint rendering into a visual process map: stages as
  distinct, ordered states across the flow; each stage's steps grouped inside
  it; the path between stages (including branch/skip gates and the way out of a
  hold) drawn, not just listed.
- A modern visual language: clear stage headers/state chips, per-step cards
  that show kind (form / record / check / approval / agent) and role with an
  icon and colour, branch targets drawn as connectors, and the run-data and
  mapped-record facts WP-0712 already surfaces kept but restyled.
- Reference target: Pega Blueprint's stage-and-step process map - clean,
  spacious, obviously a "state machine for work."
- Light and dark theming; works down to phone width (no horizontal page
  scroll); the existing export (WP-0712) keeps working.

## Out of scope

- The Visual lane builder (WP-0718) - this is the read/preview Blueprint, not
  the editor. Any shared visual components are welcome but the builder's own
  redesign, if wanted, is separate.
- The lane runner screen (WP-0707, WP-0720) - a different screen.
- Any change to the lane definition format, compile or runtime.

## Acceptance criteria

- [ ] AC-1: The Blueprint renders as a process map - stages shown as distinct,
  ordered states with their steps grouped inside each - visually modern and
  appealing (Pega-Blueprint-level), from the same lane data WP-0712 reads.
- [ ] AC-2: Each step shows its kind and role distinctly (icon + colour), and
  every branch/skip path and declared way out of a hold is drawn as a
  connector, not only named in text.
- [ ] AC-3: The redesign is accessible - readable by a screen reader, WCAG AA
  colour contrast, full keyboard navigation - and renders correctly in light
  and dark theme and at phone width (no horizontal page scroll).
- [ ] AC-4: Desktop and Team Workspace show the same redesigned Blueprint from
  the same code; the WP-0712 export still works.
- [ ] AC-5: The online demo (`app.js`) shows the same redesigned Blueprint on
  the warehouse sample lane; demo E2E covers it.
- [ ] AC-6: Website help (`/help/process-lanes`) and `llms.txt`,
  `llms-full.txt`, the Releases page and the Roadmap page are updated.

## Test plan

- Frontend: component/snapshot tests for the process-map render (stages,
  grouped steps, branch connectors) and the Receive Shipment lane snapshot;
  axe accessibility checks.
- Playwright E2E for the redesigned Blueprint page in both modes; demo E2E.
- No Rust/engine change expected; full regression suite green, test count not
  lower (`testing/REGRESSION.md`).

## Notes

- Sequencing: per the owner's 2026-11-16 direction, this Blueprint redesign
  comes **before WP-0720** (lane runner, second cut) on the UI track. It does
  not block the engine track (WP-0708, WP-0709), which proceeds in parallel.
- Builds on WP-0712 (Blueprint view and export) - reuses its data projection
  and export; this WP is the visual layer on top. Reference: Pega Blueprint.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. -->
- 2026-11-18 claude: Filed to capture the owner's Blueprint-redesign ask
  (visually appealing, modern, Pega-Blueprint-style process map of states and
  steps) as a tracked WP rather than an informal order. UI work, muse's lane;
  sequenced before WP-0720. Proposed, awaiting muse.
