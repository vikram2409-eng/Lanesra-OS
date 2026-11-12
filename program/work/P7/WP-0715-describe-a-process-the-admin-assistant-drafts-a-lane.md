---
id: WP-0715
title: Describe a process: the Admin Assistant drafts a lane
phase: 7
epic: E7.5 Authoring
status: in_progress
owner: muse
planned_for: muse
depends_on: [WP-0703, WP-0712]
requirements: [REQ-LANE-022]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: wp/WP-0715-describe-a-process-drafts-a-lane
pr: 
updated: 2026-11-12
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
