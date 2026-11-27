---
id: WP-0723
title: Admin Assistant creates and edits lanes
phase: 7
epic: E7.5 Authoring
status: in_progress
owner: muse
planned_for: muse
branch: wp/WP-0723-assistant-create-edit-lanes
depends_on: [WP-0715, WP-0718]
requirements: [REQ-LANE-022]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
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

- [ ] AC-1: A described lane with record/approval/agent steps drafts validly when every write uses an adopted Action Type and every agent is named; anything else is refused with the named reason.
- [ ] AC-2: An edit description applied to an existing lane stores a new draft version of that lane; the published version is untouched until the admin publishes.
- [ ] AC-3: The Assistant cannot publish — no publish path in either service, and a reply claiming "published" is normalized to draft.
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; demo parity on the warehouse sample data.
- [ ] AC-5: Help + sitemap + llms + Releases + Roadmap updated.

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
