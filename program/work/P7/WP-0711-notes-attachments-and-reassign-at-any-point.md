---
id: WP-0711
title: Notes, attachments and reassign at any point
phase: 7
epic: E7.3 Runner and visibility
status: done
owner: muse
planned_for: muse
depends_on: [WP-0707]
requirements: [REQ-LANE-018]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [process-lanes]
branch: wp/WP-0711-notes-attachments-reassign
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/798
updated: 2026-11-18
---

# WP-0711 — Notes, attachments and reassign at any point

## Goal

Real work needs a few actions outside the scripted steps.

## Scope

- Add note, attach file and reassign the current step from the runner; each audited; attachments follow content labels and access

## Acceptance criteria

- [x] AC-1: Each action is available at every step and appears in the audit trail
- [x] AC-2: Reassign respects the step's role
- [x] AC-3: An attachment is visible only to those who can see the run
- [x] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [x] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [x] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_anytime_actions.rs`; vitest
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
### 2026-11-18 muse
Claimed on branch wp/WP-0711-notes-attachments-reassign from main 0db8977
(the #797 merge). WP-0653's done-status folds into this PR per the
no-small-PRs rule. Read first: REQ-LANE-018 ("add a note, attach a file,
reassign the current step; each is audited"), SOLUTION-DESIGN §17 (runner:
"Notes, attachments and reassign are available at every step"), ADR-0024
A3.8. Existing pieces found: `handoff_step` (WP-0706) already hands a step
to a different ROLE, audited as `lane_step_handoff`, stored in run-data
`__handoffs` — WP-0711's reassign is the distinct person-level action
(narrow the waiter to one person who holds the step's role, AC-2).
`attachments` table exists in schema (0001) but nothing in code reads or
writes it — the attachment write/read path is greenfield. No notes table
exists. Audit pattern is `audit_repo::record` (`audit_events` table).
Next: design the three service fns (add_note, attach, reassign) + migration
0098, then Tauri/server, desktop runner UI, demo runner UI + E2E, help/docs.
### 2026-11-18 muse (design, recorded before building)
- Notes: new table `process_lane_run_notes` (migration 0098): id,
  workspace_id, run_id, step_id, author_user_id, body, created_at. Service
  `add_lane_run_note`: run must be a lane run and not terminal
  (completed/completed_partial/failed/cancelled); step_id is the current
  human step when the run waits on one, else null. Audited as
  `lane_run_note_added` against entity (`lane_run`, run_id).
- Attachments: reuse the 0001 `attachments` table (entity_type `lane_run`,
  entity_id run_id) + bytes under `<data_dir>/lane-attachments/<run_id>/`
  (data dir is the db file's parent in both editions). Service
  `attach_lane_run_file` takes bytes (base64 over the wire, same shape on
  Tauri and server); audited as `lane_run_attachment_added`. AC-3 reading:
  an attachment is listed/served only through the run view, workspace
  scoped — there is no content-labels system to follow yet (REQ-GOV-026
  is P4, unbuilt); the run's visibility IS the attachment's visibility.
- Reassign: service `reassign_lane_step` narrows the current step's waiter
  to one person, stored in run data under `__assignees.<step_id>`
  (mirroring `__handoffs`). The new assignee must hold the step's
  effective role (`access_role_service::list_assignee_user_ids`) or be an
  admin (AC-2); only the current assignee (or an admin) may reassign.
  `get_run_view` shows the assignee as `waiting_on` and `can_complete`
  requires the viewer to be that person (or an admin). A later handoff
  clears `__assignees.<step_id>` for the step, so the new role's holders
  can act.
- Audit trail (AC-1): `LaneRunView` gains `notes`, `attachments` and
  `events` (from `audit_repo::list_for_entity("lane_run", run_id)`); the
  runner's Audit trail card renders step rows + these events. Older lane
  events stay recorded against the anchor record (existing convention);
  only the three WP-0711 actions are recorded against the run, which is
  what AC-1 needs visible.
### 2026-11-18 muse (finish — all six ACs proven, PR #798)
- Core: migration 0098 (`process_lane_run_notes`), `lane_run_note.rs` model
  (+ `author_display_name`, resolved by the service so the runner shows a
  name, never a user id), `lane_run_note_repo` + `attachment_repo` (the 0001
  `attachments` table, run-scoped reads), service fns `add_lane_run_note`
  (10,000-char cap, empty refused), `attach_lane_run_file` (base64 in —
  the one shape both the Tauri command and the server route carry; 25 MB
  cap; name with no `/`, `\` or `..`; sha256 stored; bytes under
  `<data_dir>/lane-attachments/<run_id>/`), `read_lane_run_attachment`
  (run-scoped, the only read path), `reassign_lane_step` (assignee in run
  data `__assignees.<step_id>`, must hold the effective role or be an
  admin; only the current assignee or an admin may reassign; a reason is
  required; reassigning to the current assignee refused; a later handoff
  clears the assignee), `list_reassign_candidates`. `LaneRunView` gains
  `notes`, `attachments`, `events` (oldest first), `reassign_candidates`;
  the assignee gate (`actor_may_act_on_step`) is swapped into submit,
  skip, handoff and the view's `can_complete`; per-item submit keeps the
  WP-0710 gate when no step assignee is set.
- Tests: `desktop/core/tests/process_lane_anytime_actions.rs`, 6/6 —
  note lands with its step + audit event, attachment bytes on disk and
  read-back equal + refusals (bad name, empty, invalid base64), attachment
  unreadable through another run, reassign narrows to one role holder with
  the outsider refused and the submit gate proven, handoff clears the
  reassignment, finished run takes no more notes or attachments. Fixture
  corrections (not production changes): users take the legacy "Sales" role
  + a separate access-role assignment; the run-data field type is
  "string", not "text"; the candidate assertion is order-insensitive.
- Tauri: four commands (`add_lane_run_note`, `attach_lane_run_file`,
  `read_lane_run_attachment`, `reassign_lane_run_step`) + registration.
  Server: four admin routes (note POST, attachment POST, download GET with
  Content-Disposition, reassign POST) + handlers mirroring the handoff
  route. AC-4 HTTP test `anytime_actions_work_over_http_the_same_as_on_desktop`
  in `process_lane_http.rs`: note + attach + download + reassign over the
  admin routes, outsider refused, view shows the note, the attachment, the
  three audit events and waiting_on "Casey Counter" — server HTTP 3/3.
- Desktop runner UI: `LaneRunView` gains the four fields; api wrappers
  `addLaneRunNote` / `attachLaneRunFile` / `reassignLaneRunStep`
  (callAdminAction, same strings both editions) + `downloadLaneRunAttachment`
  (Tauri bytes → blob download; Team Workspace opens the admin GET route);
  `LaneRunner.tsx` gains the "Notes, attachments and reassign" card under
  the current step's work, the Audit trail card renders the run's events,
  the step header shows "waiting on <name>" when reassigned. Vitest:
  5 new view tests + 3 new api tests; `tsc --noEmit` clean.
- Demo: run gains `notes`/`attachments`/`events` at `laneStartRun`;
  `laneAddRunNote` / `laneAttachRunFile` (metadata only — the file stays
  on the user's device) / `laneReassignRunStep` (candidates = demo users
  holding the step's role via `data.roles`; stored in run-data
  `__assignees`) / `laneStepAssignee`; the anytime card under the step's
  work; the step header shows the assignee; the runner's audit trail and
  the Object 360 run audit both render the run's events. New E2E spec
  `lane-anytime-actions.spec.ts` (port 4180), 3/3; full demo regression
  61/61. The demo has no handoff UI, so no handoff-clear is needed there.
- Docs (AC-6): process-lanes help article gains WP-0711 + the new
  "Notes, attachments and reassign at any point" section; Releases
  v0.99.13; Roadmap Process Lanes entry; llms.txt + llms-full.txt (3
  lines); sitemap already carries /help/process-lanes. Also fixed two
  stale spec headers that said "Untested locally — see CI" after the
  specs had been run locally (foundry-demo 7/7, lane-wh-count 4/4).
- WP-0653's tracker `status: done` flip is set in this branch (fold-in).
