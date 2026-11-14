---
id: WP-0715
title: Describe a process: the Admin Assistant drafts a lane
phase: 7
epic: E7.5 Authoring
status: done
owner: muse
planned_for: muse
depends_on: [WP-0703, WP-0712]
requirements: [REQ-LANE-022]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [process-lanes]
branch: wp/WP-0715-describe-a-process-drafts-a-lane-r2
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/772
updated: 2026-11-14
---

# WP-0715 — Describe a process: the Admin Assistant drafts a lane

## Goal

A fast start: describe the job, review a drafted lane.

## Scope

- Assistant tool that drafts a lane definition (stages, steps, data fields) from a description, validated against the schema, opened in the blueprint view

## Acceptance criteria

- [x] AC-1: The result is always a draft and always valid against the schema
- [x] AC-2: The assistant cannot publish a lane
- [x] AC-3: The prompt is in the prompt catalog and tested with the scripted provider
- [x] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [x] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [x] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/assistant_draft_lane.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-11-12 muse: Claimed — the owner assigned the next two packages to muse ("assign next two work packages to yourself and start building it"). Status proposed → in_progress on branch wp/WP-0715-describe-a-process-drafts-a-lane. ACs reviewed and stand as written. Next: read ADR-0023, ADR-0024, SOLUTION-DESIGN.md §17 and the prompt catalog (WP-0130), then build the draft tool. Note: this branch stacks on the WP-0712 branch, which stacks on the WP-0703 branch (PR #756, CI red, fixes in progress) — the drafted lane opens in the blueprint view, so integrate with WP-0712's view as it lands; rebase onto main after #756 and the WP-0712 PR merge.
- 2026-11-12 muse: Built and pushed (commits 4de0249, 897811c) — AC-1..AC-6 all ticked, each proven by a test run locally. Draft tool: `assistant_draft_lane_service::draft_lane` (admin-only, renders catalog prompt `assistant/draft-lane` v1, one model call with tools off, normalizes the reply to draft v1 / format 1.0.0, imports via `import_lane_definition` which itself hardcodes draft — so a reply claiming "published" is stored as a draft); Tauri command `draft_lane_from_description` + Team Workspace async admin route `POST /api/admin/process-lanes/drafts` (same service, both editions). Drafts are form + check steps only — record/approval steps need the Action Type catalog (WP-0705) and agent steps need a named agent, so a person adds those when refining. Demo: deterministic drafter on the warehouse dashboard (first sentence = lane, each following sentence = one form step, stages by verb keywords), drafts in `data.draftLanes`, validated by a JS mirror of `validate_definition`. Help: `/help/process-lanes` "Describe a process" section + Releases v0.99.0 + Roadmap + llms.txt/llms-full.txt (merged with WP-0712's entries). Blueprint integration LANDED: WP-0712's view merged in (78a426d, commit 897811c) — `laneBlueprintDef` also resolves drafts (cached), each draft summary has a View blueprint button, the button binding is a named function re-run after in-place list updates; desktop needs no new UI because `list_lane_blueprints` lists every lane with its latest version, so a draft appears there with status draft and opens by lane key. Tests run locally: core `assistant_draft_lane` 6/6, `prompt_catalog` 7/7, `process_lane_blueprint` 6/6; server HTTP `assistant_draft_lane_http` 2/2, `process_lane_blueprint_http` 1/1; program tools 82 OK; demo Node harness 27/27; Playwright `assistant-draft-lane` 2/2 + `warehouse-lane-blueprint` 1/1 in Chromium. The scripted provider keeps its queue in thread-locals, so a reply scripted on the test thread cannot reach a model call on a server worker thread — the HTTP happy path is therefore proven in two halves (service call on the test thread + list over HTTP). NOT proven locally: the Tauri command's compile (`cargo check -p lanesra-os` needs GTK/WebKit system libraries this VM does not have; the install was declined — CI's tauri job is the proof) and the full Rust suite (CI). Deliberate scope boundary (follow-up, not in this WP): the draft operation is not wired into the chat tool loop (`chat_service`) — the ACs don't require it and WP-0703's lane commands follow the same staged pattern. Status stays `in_progress` with `pr:` blank — the parent sets `in_review` when the PR exists. Next: parent opens the PR; CI is the final proof (fmt/clippy on the merged tree, full suite, tauri, e2e).
- 2026-11-14 muse: Merged main (3bf87f6, #762 + #763) into the branch — 9 files conflicted, all resolved keeping both sides: app.js takes main's lane-runner route and #762 dashboard, plus this WP's draft section, draft-form binding and blueprint-button helper; laneBlueprintDef keeps main's generic fetch (literal receiving fetch, AC-5) and this WP's draft lookup; the branch's separate model-focused help article stays dropped (main's demo article is the one article, per the WP-0712 -r2 rebase) and the 'Describe a process' section is transplanted into it, wps now WP-0702, WP-0703, WP-0712, WP-0715. The two #761 fixes are kept, not reverted: the blueprint spec uses the shared static server, the blueprint test imports the desktop-local example copy. programctl validate 0 errors; program tools 85/85; lane harness 86/86. Next: PR opened; CI is the final proof (fmt/clippy, full Rust suite, tauri, e2e).
- 2026-11-14 muse: PR #765 opened (draft) from the merged branch. Status in_progress → in_review. CI is the final proof.
- 2026-11-14 muse: Folded in PR #764 and PR #766 at the owner's direction (minimize PRs). #764: run_data.set marked "provisional, demo only" in the Pick & Pack definition's description; embedded LANE_WH_PICKPACK copy in app.js regenerated (file-order minified dump, parsed-equal). #766: WP-0702 done, delivered [demo]; WP-0712 done, delivered [desktop, workspace, demo]. STATUS.md regenerated. Verified on this tree: node --check OK, programctl validate 0 errors, self-tests pass, lane harness 86/86. PRs #764 and #766 closed as folded into #765.
- 2026-11-14 muse: Rebuilt as -r2: PR #765 could not merge (mergeable_state dirty). Cause: the push tool writes one linear commit per push, so the merge commit that brought main (3bf87f6) into this branch lost its second parent — GitHub computed the merge base as an old commit (f877b2f) and the 3-way merge conflicted in 9 files. Fix: fresh branch from main with the same tree (diff main...#765 head applied), branch field updated to -r2. PR #765 closed as superseded; new PR to be opened from -r2.
- 2026-11-14 muse: PR #772 opened (draft) from the -r2 branch. Status in_progress → in_review. PR #765 closed as superseded.
- 2026-11-14 muse: PR #772 merged (ad631ef). WP-0715 done, delivered [desktop, workspace, demo]. This closes the WP and the two items folded into #772: the #764 provisional run_data.set label (in the Pick & Pack definition's description) and the #766 closeout (WP-0702 done [demo], WP-0712 done [desktop, workspace, demo]).
