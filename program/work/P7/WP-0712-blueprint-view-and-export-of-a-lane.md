---
id: WP-0712
title: Blueprint view and export of a lane
phase: 7
epic: E7.5 Authoring
status: done
owner: muse
planned_for: muse
depends_on: [WP-0703]
requirements: [REQ-LANE-020]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [process-lanes]
branch: wp/WP-0712-blueprint-view-and-export-r2
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/761
updated: 2026-11-14
---

# WP-0712 — Blueprint view and export of a lane

## Goal

Give business owners a page they can read and sign off.

## Scope

- Read-only page: purpose, stage chevrons with their steps (kind, role, writes), run data fields and mapped record fields; export to PDF

## Acceptance criteria

- [x] AC-1: The view renders any valid lane definition, including branches
- [x] AC-2: The export matches the view
- [x] AC-3: The view is readable by a Viewer and exposes no record data
- [x] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [x] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [x] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest for the view; a snapshot of the Receive Shipment blueprint
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-11-12 muse: Claimed — the owner assigned the next two packages to muse ("assign next two work packages to yourself and start building it"). Status proposed → in_progress on branch wp/WP-0712-blueprint-view-and-export. ACs reviewed and stand as written. Next: read ADR-0023, ADR-0024, SOLUTION-DESIGN.md §17, then build the blueprint view + PDF export. Note: this branch stacks on the WP-0703 branch (PR #756, CI red on fmt + one compile error + two clippy errors, fixes in progress) — rebase onto main after #756 merges.
- 2026-11-12 muse: Built — AC-1..AC-6 ticked, all from tests run locally (a local Rust toolchain, 1.97.0, landed in this VM during the build, so Rust tests ran here instead of CI-only). Core: new `services/process_lane_blueprint.rs` (`get_lane_blueprint` / `list_lane_blueprints`, authenticated-member reads, no admin gate, definition only) + Tauri commands + server dispatch arms on the same service; 6 tests in `core/tests/process_lane_blueprint.rs` pass, 1 HTTP test in `server/tests/process_lane_blueprint_http.rs` passes. Frontend: `features/lanes/LaneBlueprint.tsx` (`LaneBlueprintView` + `LaneBlueprintPage`, Process Lanes nav section, same component on screen and in PrintOverlay for AC-2); 7 Vitest tests pass, snapshot of the Receive Shipment blueprint committed, full suite 75/75, tsc clean. Demo: `renderLaneBlueprint` + route + dashboard card in app.js reading the shared example (falls back to fetch when the WP-0702 seed is absent), print button + `@media print` rules; local Node harness 34/34 on the real renderer, 5 program-tools tests pass (suite 82/82), Playwright spec committed (browser run is CI-only). Docs: `/help/process-lanes` blueprint section + wps WP-0712, Releases v0.99.2, Roadmap foundation sentence, llms.txt + llms-full.txt, index.html cache-buster v0.99.2; sitemap already listed the article. `programctl validate` 0 errors / 0 warnings. Open items: (1) AC-3 scope note — the shared example's own authored text names PO-2026-001 as its worked example (step descriptions, close notification); the blueprint renders the definition faithfully and never reads a record, but a literal-free example would need a WP-0701 example revision. (2) Found while testing: WP-0703's `core/tests/process_lane_definition.rs` `example_path()` uses `../../../` which overshoots the repo root (should be `../../`); its tests never ran anywhere yet because CI failed at compile first — flagged to the parent, NOT fixed here (WP-0703's file). (3) `cargo clippy -p lanesra-core --all-targets` still fails on the two known WP-0703 errors (unreachable pattern in `models/process_lane.rs:516`, unnecessary unwrap in `services/process_lane_service.rs:718`); no clippy errors in this WP's files. (4) E2E browser run + full CI suite are CI-only; status stays in_progress with pr: blank (validate refuses in_review without a link) — parent sets in_review when the PR exists. Next: parent opens the PR, watches CI, merges when green, then rebase is NOT needed by this WP (branch stacks on #756; rebase onto main after #756 merges is the parent's step).
- 2026-11-12 muse: Rebasing onto main — original branch stacked on the WP-0703 branch; after PR #756 (WP-0703) and PR #758 (WP-0702, merge 29681b1) and PR #760 (WP-0701/WP-0703 closeout, merge c4fc2bf) landed, a fresh branch wp/WP-0712-blueprint-view-and-export-r2 was cut from main c4fc2bf and the branch's 24 files re-applied onto it. The merge vs main conflicted in 5 files (app.js, dispatch.rs, index.html, REGRESSION.md, styles.css); all resolved by keeping both sides, except the /help/process-lanes article: the branch carried a second model-focused article (wps WP-0703, WP-0712) duplicating main's demo article (wps WP-0702, WP-0703) under the same slug — merged to ONE article (main's demo article kept, the branch's blueprint section moved into it before 'What is still coming', wps now 'WP-0702, WP-0703, WP-0712'). Re-verified on the rebased tree: node --check app.js OK, programctl validate 0 errors, program-tools 85/85, Rust blueprint core 6/6 + server HTTP 1/1, Vitest blueprint 7/7, Playwright blueprint spec 1/1 (run from the repo root — the spec only starts its server from there) + lane spec 5/5 (no regression). Next: PR opened from the -r2 branch; watch CI; merge when green.
- 2026-11-14 muse: Closed out — editions confirmed delivered: desktop, workspace, demo. Desktop and Team Workspace share the blueprint component and service (AC-4): core 6/6 + server HTTP 1/1 + Vitest 7/7 + snapshot at build time, PR #761 CI green on merge. Demo: renderLaneBlueprint + route + dashboard card in app.js, Playwright spec green in CI, blueprint verified live on main (5 stages, 12 steps, matches the spec). /help/process-lanes names WP-0712 (wps field). Status in_review → done, delivered: [desktop, workspace, demo].
