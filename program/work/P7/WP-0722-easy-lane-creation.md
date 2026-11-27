---
id: WP-0722
title: Easy lane creation: object-first entry and guided start
phase: 7
epic: E7.5 Authoring
status: done
owner: muse
planned_for: muse
depends_on: [WP-0718]
requirements: [REQ-LANE-024]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [process-lanes]
branch: wp/WP-0722-easy-lane-creation-r2
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/823
updated: 2026-11-27
---

# WP-0722 — Easy lane creation: object-first entry and guided start

## Goal

An admin creates a lane from the thing the lane is about — the object —
instead of starting from a blank builder and picking the object second.

Owner direction (2026-11-27): "make the create lane easy for admin."

## Scope

1. **Object-first entry.** A "New lane" action on the object's page (the
   custom object's record list) opens the builder with that object already
   picked — no Object card, no second step. The blank-lane Object card stays
   as the fallback for lanes started from the Lanes section.
2. **Guided start.** Creating a lane asks three things, then lands in the
   builder: name, object (pre-filled when coming from the object), and start
   point — blank, describe it with the Assistant (WP-0715 draft), or a
   template when one exists (WP-0717).
3. **Keep the fast path fast.** No new mandatory screens inside the builder;
   the chevron canvas, live checks and dry-run preview are unchanged.
4. **Docs.** `/help/process-lanes` gains the object-first flow; sitemap /
   llms / Releases / Roadmap updated.

## Design

- Create the process *for* the table (D365 binds a BPF to its entity);
  Pega's case type *is* the object. The builder already hides the Object
  card once `def.object_key` is set and blocks publish/dry-run until it is
  — so object-first entry is a pre-filled blank definition handed to the
  existing `LaneBuilderPage` (`initialDefinition`), not a new builder mode.
- The guided start is one shared dialog (`NewLaneStart`): name, object
  select, start-point radio. Blank lands in the builder with a blank
  definition on that object. Describe calls the WP-0715 draft
  (`draft_lane_from_description`, new `api.draftLaneFromDescription`
  wrapper through the shared dual-transport `callAdminAction`) and lands in
  the builder with the drafted definition — still a draft, never published
  by the Assistant. Template is offered disabled with "No templates yet"
  until WP-0717 lands; no templates are added here.
- Desktop and Team Workspace share the React code and the same api calls
  (AC-3 at this layer). The demo mirrors it in `app.js`: the guided start
  dialog over the demo's deterministic `draftLaneFromDescription`, and a
  "New lane" action on each warehouse object list page.
- Does NOT add objects (no OOTB objects in the product — Custom Objects
  only) and does NOT add templates (WP-0717, scope decision pending).
- Does NOT change the builder canvas, checks, or publish rules.

## Acceptance criteria

- [x] AC-1: From an object's page, "New lane" opens the builder with that object set; publish and dry run are available without ever touching the Object card.
- [x] AC-2: The guided start offers blank / describe / template and lands in the builder with the choice applied.
- [x] AC-3: Desktop and Team Workspace behave the same from the same code; demo parity where the demo has objects.
- [x] AC-4: Help + sitemap + llms + Releases + Roadmap updated.

## Test plan

- vitest: `NewLaneStart` dialog (options, pre-filled object, blank/describe/template-disabled, confirm payload); `LaneBuilderPage` with a pre-filled object hides the Object card; `api.draftLaneFromDescription` wrapper (mocked transport).
- Playwright E2E (demo): guided start offers the three start points; object-first from a warehouse object list opens the builder with that object set; describe drafts deterministically.
- Tag every new test with `Covers: REQ-LANE-024 (WP-0722 AC-n)` so `programctl trace` finds it.
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md).

## Notes

- Depends on WP-0718 (done). The WP-0715 draft contract (form + check steps only, always a draft) is unchanged; full-lane drafting is WP-0723, not this WP.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-27 muse: Claimed (branch wp/WP-0722-easy-lane-creation, from main a0e6f6c). WP-0723 tracker file added alongside (proposed, not built). Full Rust suite started in the background before the build (log ~/workspace/lanesra-build/wp0722-full-rust-suite.log). Next: build per the Design section.
- 2026-11-27 muse: Built, all ACs ticked, WP in_review, PR #821 draft. Desktop + Team Workspace (same React + api code): new `NewLaneStart` guided-start dialog (name, object pre-filled, blank/describe/template with template disabled — no templates yet); "New lane" on a custom object's record list (`CustomObjectRecords`, via `onNewLane` plumbed through App) and on the Lanes section (`LaneBlueprintPage`, incl. the empty state); App hands the choice to `LaneBuilderPage` as an `initialDefinition` — blank with the object set, or the WP-0715 draft parsed by `draftedLaneDefinition` (new `api.draftLaneFromDescription` through `callAdminAction`: Tauri `draft_lane_from_description`, HTTP POST /api/admin/process-lanes/drafts; always a draft, no publish path). The Object card still renders for a blank lane with no object. Demo (`app.js`): guided-start route `laneBuilderNew` (same three start points; describe reuses deterministic `draftLaneFromDescription` and stores the draft), "New lane" on each of the seven lane-object list pages, all `data-lane-builder-new` buttons route through it. Docs: /help/process-lanes gains a "Creating a lane: object-first and guided start" section + WP-0722 in `wps:` + summary; Releases v0.99.16; Roadmap Process Lanes entry (description + evidence bullet); llms.txt + llms-full.txt. Proof: builder vitest 75/75 (15 new); full vitest 243 passed + 1 flake (LanePublish axe timeout under parallel load, 8/8 alone); tsc + `npm run build` clean; `node --check` clean; programctl validate + docs --check 66/66 clean; trace links REQ-LANE-024 to WP-0722 (7 test files); demo E2E 29/29 locally (24 pre-existing incl. 4 re-routed through the guided start + 5 new) after installing the pinned chromium 1255. Full Rust suite running locally (no Rust files touched — result pending at push time, CI is the proof). Fixes found by testing (each its own behaviour, no test loosened): two `getByText` multi-match assertions narrowed to the badge / Dry-run-preview heading. No templates added, no objects added, no builder/canvas/check/publish change. Next: parent marks #821 ready and merges when CI is green; the done-flip folds into the next substantive PR after the merge.
- 2026-11-27 muse: Main moved under #821 (PRs #820, #822 merged after the a0e6f6c cut) — mergeable_state "dirty": the only conflict was program/STATUS.md, whose aggregate rows both sides regenerate differently; no content fix can merge it, so rebuilt as a fresh branch wp/WP-0722-easy-lane-creation-r2 from main 3a122c6 (same 19 build files + synced WP-0160/WP-0717/WP-0721 tracker files; STATUS.md regenerated; WP file branch field updated). PR #823 opened as draft; #821 closed as superseded. Local full Rust suite was killed by a VM restart at 102 test binaries ok / 0 failed with no EXIT line (no Rust files touched by this WP) — CI is the proof. CI on #823 head 51ced91: 18 success / 6 neutral / 0 failed required (rust format+lint, rust tests 1/4–4/4, tauri, web, demo, migrations, program all green) and 1 failure in the non-required playwright (e2e) job: `lane-builder.spec.ts` "New lane starts from a blank definition" and the gaps-batch test — both clicked "New lane" and expected to land straight in the builder, but the guided start now intercepts. Fix (same PR, no test loosened): both tests go through the dialog via a new `openNewLaneBlank` helper (blank default; name/object set in the dialog), and the helper asserts the builder heading is the lane's name — the guided start hands the builder an `initialDefinition`, so LaneBuilderPage's "New lane" h2 (no-definition path only) does not render; the builder shows the lane name ("Untitled lane" when unnamed). Verified locally exactly as CI runs it (fresh data dir, built frontend + server): full Playwright suite 79 passed / 0 failed (log ~/workspace/lanesra-build/wp0722r2-playwright-full2.log). A standalone re-run of lane-builder.spec.ts against the dirtied data dir failed two seed tests (AC-1/AC-2) because the earlier run had saved the fail branch into the shared seed lane — re-running on a fresh dir passes; CI always uses a fresh dir. Next: push this fix, wait for CI green on #823, then finalize-at-merge IN THE SAME PR (flip status to done, programctl status), mark ready, merge.
- 2026-11-27 muse: CI green on #823 head 09e5c80 (19 success / 6 neutral / 0 failed — all required checks plus playwright e2e after the guided-start re-route). Finalizing per the finalize-at-merge rule: AC-1..AC-4 already ticked and proven, delivered [desktop, workspace, demo], status flipped to done, `programctl status` re-run (validate: 260 WPs, 0 errors). Next: mark #823 ready and merge.
