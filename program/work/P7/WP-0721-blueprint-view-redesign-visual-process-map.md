---
id: WP-0721
title: "Blueprint view redesign: a visual, modern process map of states and steps"
phase: 7
epic: E7.5 Authoring
status: done
owner: claude
planned_for: claude
depends_on: [WP-0712]
requirements: [REQ-LANE-020]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [process-lanes]
branch: wp/WP-0721-blueprint-redesign
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/805
updated: 2026-11-27
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

- [x] AC-1: The Blueprint renders as a process map - stages shown as distinct,
  ordered states with their steps grouped inside each - visually modern and
  appealing (Pega-Blueprint-level), from the same lane data WP-0712 reads.
- [x] AC-2: Each step shows its kind and role distinctly (icon + colour), and
  every branch/skip path and declared way out of a hold is drawn as a
  connector, not only named in text.
- [x] AC-3: The redesign is accessible - readable by a screen reader, WCAG AA
  colour contrast, full keyboard navigation - and renders correctly in light
  and dark theme and at phone width (no horizontal page scroll).
- [x] AC-4: Desktop and Team Workspace show the same redesigned Blueprint from
  the same code; the WP-0712 export still works.
- [x] AC-5: The online demo (`app.js`) shows the same redesigned Blueprint on
  the warehouse sample lane; demo E2E covers it.
- [x] AC-6: Website help (`/help/process-lanes`) and `llms.txt`,
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
- 2026-11-20 claude: Built the redesign (owner reassigned it to claude). The
  shared `LaneBlueprintView` (desktop + Team Workspace) and the demo's
  `renderLaneBlueprint`/`laneBlueprintStepHtml` now draw a visual process map:
  a stage **rail of states** across the top (numbered, linking to each state),
  then each stage as a **state column** of step cards. Each step card has a
  coloured kind strip + an icon per kind (form/record/check/approval/agent,
  `KindIcon` / `laneKindIcon`, same SVG paths both sides) and a role chip; each
  branch / way-out is **drawn as a connector** (arrow glyph + a colour-coded
  target pill) while keeping the exact `on → target` text WP-0712's tests
  assert. New `.bp-*` CSS in `desktop/src/styles.css` and `styles.css` (token
  driven, so light + dark both theme; `@media(max-width:760px)` collapses the
  map to one column and the print block flattens it top-to-bottom for the
  export). AC-1/AC-2 (map, kind icon+colour, drawn connectors); AC-3 (axe test
  green; WCAG tokens; keyboardable rail links; verified no horizontal page
  overflow at 360/390/768px); AC-4 (one React component for desktop +
  workspace; PrintOverlay export unchanged); AC-5 (demo render + strengthened
  `warehouse-lane-blueprint.spec.ts`); AC-6 (help article, llms.txt,
  llms-full.txt, Releases v0.99.14, Roadmap). Local: Vitest 222 passed (incl.
  the new axe test; snapshot regenerated); `tsc` + `vite build` clean; demo
  blueprint renders 5 stages / 13 steps / 4 connectors via the environment's
  Chromium (the pinned e2e browser is CI-only). Opening the PR; status to
  in_review once it is linked.
- 2026-11-27 claude: Finalized. PR #805 merged to main (2026-11-20); the merged code carries every edition, so status in_review → done and delivered [desktop, workspace, demo] (all six ACs were already ticked from the build). Verified on main a0e6f6c: desktop + Team Workspace share `desktop/src/features/lanes/LaneBlueprint.tsx` (with its snapshot + axe test) and `.bp-*` tokens in `desktop/src/styles.css` (AC-1/2/3/4); the online demo renders the same process map via `renderLaneBlueprint`/`laneBlueprintStepHtml`/`laneKindIcon` + `.bp-*` in `app.js`, covered by `desktop/e2e/tests/warehouse-lane-blueprint.spec.ts` (AC-5); docs updated in `llms.txt`/`llms-full.txt` + help/process-lanes + Releases v0.99.14 + Roadmap (AC-6). This was a merged-but-left-in_review record, not new work; no product code changed. STATUS.md regenerated; programctl validate 0 errors.
