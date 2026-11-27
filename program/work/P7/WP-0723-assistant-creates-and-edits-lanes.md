---
id: WP-0723
title: Admin Assistant creates and edits lanes
phase: 7
epic: E7.5 Authoring
status: done
owner: muse
planned_for: muse
branch: wp/WP-0723-assistant-create-edit-lanes
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/824
depends_on: [WP-0715, WP-0718]
requirements: [REQ-LANE-022]
estimate: L
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [process-lanes]
updated: 2026-11-27
---

# WP-0723 — Admin Assistant creates and edits lanes

## Goal

The Assistant does the authoring; the admin reviews and publishes. Create and
edit both go through the Assistant, so a lane goes from description to
publishable draft without hand-editing JSON or clicking through every step.

Owner direction (2026-11-27): "Admin Assistant can create the lane and edit the
created lane. Let admin user publish it — but Assistant can create and edit so
the work gets faster."

## Scope

1. **Create — full lane, not just form + check.** Extend `draft_lane` and the
   `assistant/draft-lane` prompt past the WP-0715 narrow contract:
   - record steps using ADOPTED Action Types only (set_status, reassign_owner,
     set_fields today); anything else is refused with the named reason, not guessed.
   - approval steps (writes + approve/reject branches) where the description
     states the decision.
   - agent steps only when the description names the agent.
   - the draft is still always a draft; the Assistant still cannot publish.
2. **Edit — revise an existing lane by description.** New service
   (`assistant_edit_lane_service`) + new catalog prompt `assistant/edit-lane` v1:
   takes a lane key + a change description, loads the current definition, one
   model call with tools off, normalizes the reply to a new draft version of
   the SAME lane, validates against the schema + publish checks, stores as a
   draft version. Refuses when the description is empty, the lane key is
   unknown, the reply is not a definition, or validation fails — nothing stored
   on refusal. Never publishes; never touches a published version (edits land
   as a new draft version; the admin publishes).
3. **Assistant UI.** From the Blueprint page ("Ask Assistant to edit" on a lane)
   and from inside the builder; the result opens in the builder for review.
4. **Tests.** New prompt eval cases (create-full + edit cases); core tests for
   the edit service (refusals, draft-version storage, no publish path);
   E2E draft-then-edit-then-publish.
5. **Docs.** `/help/process-lanes` covers Assistant create + edit; sitemap /
   llms / Releases / Roadmap updated.

## Acceptance criteria

- [x] AC-1: A described lane with record/approval/agent steps drafts validly when every write uses an adopted Action Type and every agent is named; anything else is refused with the named reason.
- [x] AC-2: An edit description applied to an existing lane stores a new draft version of that lane; the published version is untouched until the admin publishes.
- [x] AC-3: The Assistant cannot publish — no publish path in either service, and a reply claiming "published" is normalized to draft.
- [x] AC-4: Desktop and Team Workspace behave the same from the same code; demo parity on the warehouse sample data.
- [x] AC-5: Help + sitemap + llms + Releases + Roadmap updated.

## Test plan

- Core tests for the edit service; prompt eval cases; E2E draft-then-edit-then-publish.
- Tag every new test with `Covers: REQ-LANE-022 (WP-0723 AC-n)` so `programctl trace` finds it.
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md).

## Notes

- Boundary (honest): full-lane creation is bounded by the Action Type catalog.
  Writes beyond the adopted set need the WP-0705 follow-up (Claude's track);
  the Assistant refuses those rather than inventing ids.
- Boundary: agent steps need a named agent in the description; the Assistant
  never picks an agent on its own.
- Follows the WP-0715 pattern exactly: admin-only, one model call, tools off,
  catalog prompt, eval cases, schema validation before storage.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-27 muse: Tracker file added on the WP-0722 branch (proposed, not built). Next: build after WP-0722 merges.
- 2026-11-27 muse: Claimed on wp/WP-0723-assistant-create-edit-lanes (main a854eec, WP-0722 merged). Baseline validate clean (260 WPs, 352 reqs). Plan: draft-lane v2 (full lane: record/approval/agent steps, adopted Action Types only) + new assistant/edit-lane v1 + assistant_edit_lane_service (new draft version, never publish) + Tauri/server/API wiring + Blueprint + builder UI + demo parity + docs. Next: core services + prompts, then tests.
- 2026-11-27 muse: Implementation complete, PR #824 open (draft). Prompts: draft-lane v2 (full lane) + edit-lane v1, catalog locks updated, eval cases 5 + 4. Services: `validate_assistant_definition` in draft service (live-catalog P-12 check + named-agent check); new `assistant_edit_lane_service` (admin-only, one model call tools off, draft-in-place when latest is draft else new draft version, never publishes). Wiring: Tauri `edit_lane_from_description`, server POST /api/admin/process-lanes/:lane_key/edits, api.ts + builderUtils. UI: Blueprint "Ask Assistant to edit" (opens revision in builder) + in-builder edit panel. Demo: deterministic `editDraftLaneFromDescription` + per-draft edit form in the Process Lanes panel. Docs: /help/process-lanes (new "Revise a drafted lane" section + full-lane draft), Releases v0.99.17, Roadmap shipped-lanes entries, llms.txt + llms-full.txt; `programctl docs --check` 66/66 clean.
- 2026-11-27 muse: Decisions (recorded in code docs + here). (a) Edit validation scope: "schema + publish checks" is interpreted as schema (hard gate) + P-12 adopted-catalog check (the publish check that applies at authoring time). The full publish report is NOT run on edit: a draft is allowed to be unpublishable, and refusing an edit because the draft fails an unrelated check would block fixing exactly the drafts the Assistant exists to fix. (b) Prompt v2 only authors `<object>.set_status` writes — the one adopted write whose inputs the model can write without guessing parameter keys; the service accepts any adopted reassign_owner/set_fields write the model does produce. (c) Agent existence is NOT checked — the lane schema names agents loosely and no publish check validates agent existence today. (d) E2E publish leg cannot run in the demo (demo has no publish path — its Publish button says so); the draft-then-edit-then-publish E2E is a service-level core test instead. (e) Playwright and the Tauri command are CI-only in this VM (no browsers, no GTK/WebKit libs).
- 2026-11-27 muse: Local verification so far: prompt_catalog 7 passed; core assistant_draft_lane 9 passed, assistant_edit_lane 10 passed (incl. E2E draft-then-edit-then-publish); server HTTP 2 + 2 passed; Vitest targeted 37 passed; `tsc --noEmit` clean; `cargo fmt --check` clean; demo Node harness 11/11; `node --check app.js` clean; `programctl validate` + `docs --check` clean. Full Rust suite and full frontend suite + build still running. Next: commit + push progress, full-suite results, REGRESSION.md entry written, finalize at merge (tick ACs, delivered [desktop, workspace, demo], status done).
- 2026-11-27 muse: Full verification green locally — full Rust suite 1,445 passed / 2 failed (both the known pre-existing ssrf_guard sandbox failures); full Vitest 254 passed + LanePublish 8/8 alone (the one full-suite failure was its 5000ms timeout under parallel load + concurrent clippy — CPU contention, not a regression); `npm run build` clean; clippy core 0 warnings (fixed 8 doc-list warnings in the draft service); demo Node harness 11/11; Playwright spec 4/4 and full e2e suite 81/81 locally (Chromium 1255 was installed after all); perf_baseline 3 passed; program tools self-tests 90 passed; `status --check` + `docs --check` clean. Pushed in 2 commits (a4ed8956 implementation, b16cd0fb program); PR #824 head b16cd0fb. CI run 46040994780 on that head: Status Success, every job green (demo, rust lint, server image, rust tests 1-4/4, web, migrations, vocabulary, tauri, rust tests aggregate). NOTE: mergeable_state reads "unstable" despite the green CI run — the failing check is outside ci.yml (e2e / perf-baseline / program-status / security are the other PR workflows); every one of those verifies green locally, so the cause is unconfirmed — possibly a transient/flaky non-required check or a cancelled-run artifact from the two pushes 20s apart. Finalize commit follows the task rule (ACs ticked, delivered set, status done); if the fresh run still reads unstable after undrafting, report it to the coordinator instead of forcing the merge.
