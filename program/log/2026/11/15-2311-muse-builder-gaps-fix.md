# Session log — builder-gaps fix batch: all 17 probe gaps (docs second pass)

Date: 2026-11-15. Agent: Muse (docs worker, second pass). Branch:
`fix/builder-gaps-all-17`, base `origin/main` 84277fd (PR #787 merged).
Four workers: core/service, desktop frontend, online demo, docs. No worker
pushed, opened a PR, merged, or committed — changes left in the working
tree for the coordinator to commit.

This log was written by the second-pass docs worker from the three build
workers' reported summaries and the first docs worker's landed tracker
edits. It was not written by any of the build workers, so per-gap claims
below are their reports, not this worker's own verification.

## The 17 gaps and what each got

Gap numbers are the probe report's
(`~/workspace/lanesra-build/builder-probe-report.md`).

1. **No run-data field editor (desktop + demo).** FIXED both. Desktop:
   editor beside the plumbing table (key/type/required), duplicate-key
   rejection, a "Bind to declared field" select, atomic declare+add
   (fixes a stale-state race where the second `setDef` clobbered the
   declaration). Demo: Add control beside the table + empty state,
   duplicate/bad-key inline refusal.
2. **No add-write-input UI (desktop).** FIXED. Editable name+value per
   input, +Input, per-input Remove. (The demo already had this — probe
   D3b.)
3. **Kind-change to check yields an unusable check step (desktop).**
   FIXED. The check fieldset now initialises with the demo's exact
   default on kind-change, and never clobbers an existing check.
4. **No delete for steps or stages (desktop + demo).** FIXED both.
   Confirm before delete; deleting a step also deletes the branches
   targeting it and reports the count. Demo stage confirm names the
   step count.
5. **Publish stays disabled after save (desktop).** FIXED. Save adopts
   the returned `lane_key`, so Publish enables immediately.
6. **No object picker on a new lane (desktop).** FIXED. Object picker
   (`listCustomObjects(true)`, custom objects only) on a new lane;
   Publish disabled with "Pick an object first…" until one is chosen.
   Core half: P-01 now refuses an object-less lane
   (`a lane must name its object`).
7. **Self-loop branches pass every check (desktop + demo).** FIXED.
   Core: P-06 refuses a branch targeting its own step (`branch 'fail'
   on step 'x' targets the step itself — a check that fails would
   retry forever`). UI: target selects exclude the step itself;
   "Add branch" / "Add missing branch" defaults to the next step, else
   end `{outcome:"held"}`. Demo adds the P-06 self-loop refusal in its
   own `lanePublishChecks` as defense in depth.
8. **P-12 dead end on all warehouse lanes.** FIXED — adoption, not
   removal. Core: `adopt_builder_action_type` (admin-gated with
   `publish_lane`'s exact gate), Tauri command + `lib.rs`
   registration, server dispatch arm, `api.ts` wrapper. Rules: a
   `run_data.set` adoption is refused (ADR-0025 error); a
   non-`<object>.set_status` adoption is refused (no faithful
   template); a prefix that does not match the target is refused
   (cross-record). The adopted template is `set_fields {"status":
   "status"}`, one required `status` param, capability `update`.
   Adopting the same (name, target) twice returns the existing row
   (idempotent). Desktop: an "Adopt <action_type>" button on each
   provisional write's P-12 note; success bumps `catalogVersion` and
   the P-12 clears. Three `*.set_status` types are now adopted on
   their own anchors — `purchase_order.set_status`,
   `sales_order.set_status`, `cycle_count.set_status` — but the lanes
   still cannot publish: exactly 15 P-12 refusals remain (see
   "Owner-approved calls" below). The demo's P-12 note already named
   adoption; its provisional list was verified duplicate-free (10
   unique).
9. **Imported warehouse lanes show a wall of P-11/P-13 refusals.**
   FIXED (core half). `PublishReport.framing: Option<String>`, set
   when the lane's object is neither a `CORE_ENTITY_TYPES` entry nor
   a valid dynamic entity type. Test: Receiving in a fresh workspace
   gets framing + P-11s + exactly 2 P-13s; in the seeded workspace
   `framing == None`. Demo verified n/a: Receiving P-11/P-13 both
   Pass in the seed (objects and `wf-wh-*` workflows exist there).
10. **Dry run has no fixture input (desktop).** FIXED. Per form
    step/field type-aware fixture inputs, per declared run-data field,
    an approval-decision select per approval step, all passed to
    `runDryRun`. Demo: no change needed — per-line `counted_qty`
    fixture inputs already existed and the demo's dry-run reporting
    already names the anchor, expected outcome, and "Nothing was
    written" (probe D5).
11. **Role is free text; a bogus role passes every check.** FIXED on
    the desktop via the picker; core half DELIBERATELY NOT DONE (see
    "Deliberately not done" below). Desktop: role picker from
    `listAccessRoles()` when non-empty, else free-text fallback with
    a visible warning — the fresh-workspace list is empty, so the
    fallback is the live path; the picker path is vitest-covered.
    Demo: no role picker reported in this batch (demo lanes keep
    their seed roles).
12. **One early refusal floods the check list with "not evaluated".**
    FIXED on desktop: collapsed to one line — `N check(s) not
    evaluated — fix P-0Y first: <message>`. Demo verified n/a (no
    `not_evaluated` concept in `app.js`).
13. **Demo builder cannot grow a lane.** FIXED. Add step/stage
    controls (unique ids; a new stage starts with one blank step).
    A blank lane's Publish is DISABLED ("Publish disabled — blank
    lane"); one existing test was updated as allowed (blank-lane test
    now asserts Publish disabled, then enables after adding a field;
    the click still shows the "demo cannot publish" note).
14. **Dry-run error message misleads (desktop).** FIXED. The exact
    brief text ("Can't dry-run this draft: step '…' needs a role.
    Fix the checks above, then dry-run here."); the generic fallback
    keeps the same two-sentence shape. Demo n/a (no "not available
    yet" text in the demo).
15. **Count: no per-item assignment, no skip filter (Q-33/Q-34).**
    FIXED — schema + controls. Schema: `repeat_over.assign`
    (role or child_field + field) and `step.skip_when` (bare
    `condition_or_group`; schema rule requires `repeat_over`).
    Core P-01 validates both (`skip_when` on a non-repeating step;
    `assign` child_field with no field). Desktop: assign control
    (role|child_field + field input) and a `skip_when` builder
    reusing ConditionBuilder with the correct direct
    ConditionOrGroup shape (the first `{condition:{...}}` wrapping
    was wrong — fixed); steps with `skip_when` no longer warn; a
    "Repeat over" input was added for steps lacking it. Demo: Count
    assign + skip controls on the step panel and the Count per-lane
    surface (with the WP-0710 note); the no-filter warning disappears
    once a skip is set.
16. **Replenish supplier picker lists reorder rules, not suppliers
    (Q-35).** FIXED — schema + controls. Schema: `form_field.resolve`
    + `field_resolve` definition, kind `latest_record` only; core
    P-01 refuses any other `resolve` kind. Desktop: resolve control
    ("Resolve from latest record") with a follow-up note. Demo: the
    supplier picker now lists `data.supplier` (separate from the
    dry-run anchor select) with a resolve control on the supplier
    field. Note: no "Related record (supplier)" select existed
    before — the supplier previously came only from a fixture
    default. Write-input `resolve` NOT implemented (see "Deliberately
    not done").
17. **No `replaces` editor.** FIXED both. Desktop: workflow select
    via `api.listWorkflowRules(object_key)` when rules exist, else
    free-text with a note that no workspace-wide workflow list API
    exists. Demo: workflow select from `data.workflowRules` + a
    numeric action index with a 0–N hint; add/remove; round-trip
    preserved.

Cross-cutting core fixes in the same batch: P-09 labelling
(`reachability: ` / `outcome coverage: ` prefixes on pass reasons,
a `not_evaluated` reason, validate refusals, per-check outcome
refusals; the `validate_definition stopped at P-0Y` and `has no fail
branch` substrings are intact); Action-Type picker dedupe (by name,
adopted beats provisional); P-11 refs (`skip_when` refs and
`resolve.where` `$`-values collected; `resolve.field` never
collected). Dry-run walker: NO CHANGE NEEDED — a new test proves a
lane carrying all three new shapes dry-runs identically to the
unadorned lane. Schema shapes validated against 5 examples; the demo
verified `skip_when` is a bare `condition_or_group` leaf, with no
`{condition:}` wrapper.

Per-line Count table decision (both builders, same call, differing
only in placement): per-line edits are DRY-RUN FIXTURES for the
count step (`per_child[line].counted_qty`), NOT lane edits —
`counted_qty` is record data, the stored definition is byte-identical
after an edit, and the next dry run simulates the edited counts. The
desktop table edits feed the dry run; the demo keeps its per-lane
table as an explicitly read-only preview with a note. A per-child
fixtures UI is out of scope for this batch.

## Owner-approved calls (three)

1. **Gap 8 adoption, with the exact P-12 remainder.** Adopting the
   three faithful `*.set_status` types leaves exactly 15 P-12
   refusals, test-asserted in
   `adopting_warehouse_set_status_types_leaves_exactly_the_cross_object_p12_refusals`:
   Receiving 3 (`warehouse.post_stock_move` x2,
   `purchase_order.close_receipt` x1); Pick & Pack 8
   (`picking.set_status` x2, `warehouse.post_stock_move` x3,
   `shipment.create` x1, `sales_order.ship` x1, `run_data.set` x1);
   Count 1 (`warehouse.post_stock_move` x1); Replenish 2
   (`purchase_order.create` x1, `purchase_order.set_status` x1 — no
   row exists for the `reorder_rule` anchor, so adopting there would
   be unfaithful); Supplier Check 1 (`run_data.set` x1). Totals:
   `warehouse.post_stock_move` x6, `picking.set_status` x2,
   `run_data.set` x2, and one each of `purchase_order.close_receipt`,
   `purchase_order.create`, `purchase_order.set_status`,
   `shipment.create`, `sales_order.ship`. Recommendation (recorded,
   not decided): all thirteen cross-object writes need WP-0705
   follow-up (new invoke effects / a warehouse-specific invoke
   path), NOT a lane rework — no lane step was reworked and no lane
   behaviour changed. `run_data.set` x2 is never adopted (ADR-0025
   decision 5). The `llms.txt` / `llms-full.txt` sentence was updated
   in this batch's second pass to state this position (see "Docs"
   below); it no longer says every warehouse Action Type is
   provisional.
2. **Q-33 / Q-34 / Q-35 accepted and implemented.** Owner accepted
   all three on 2026-11-15 23:11 EST via the fix-all-17 plan
   approval. Recorded in
   `program/spec/schema-extension-proposal-q33-q36.md` (each item
   separately, with date and batch) and in `program/RAID.md` rows
   42–44 (status Answered, with implementation pointers to the
   schema change + builder controls). Schema:
   `repeat_over.assign`, `step.skip_when`, `form_field.resolve` +
   `field_resolve` def. Rust model mirrors it, serde-defaulted.
3. **Q-36 untouched.** Q-36 (Counter role, `role_wh_counter`) stays
   OPEN and untouched — its RAID row and proposal section are
   byte-identical to `main` (verified: `git diff` shows no added or
   removed line mentioning Q-36 or `role_wh_counter` in either file).

## WP-0718 closeout

Status `in_review` → `done`, `delivered [desktop, workspace,
demo]`, folded into this batch — the closeout the WP has been waiting
for since PR #784 merged; no separate PR. Progress log entry added to
`program/work/P7/WP-0718-visual-lane-builder.md` (AC ticks and Design
section intact; history not rewritten). `program/STATUS.md`
regenerated: Phase 7 now 13 done / 0 in review, 39/61 points, 64%;
"Active now" is empty.

## Bugs found by testing (frontend E2E) + one adjacent fix

1. **First save remounted the builder.** After the first save the
   page switched to the server-load branch, wiping the "Saved as
   version" confirmation, the dry-run report, and the fixtures.
   Fixed in `LaneBuilderPage.tsx`: a lane created in this session
   keeps its in-memory definition after first save; only lanes opened
   by key load from the server.
2. **Untouched approval select sent no fixture.** The select showed
   "Approve" but sent nothing, so the dry run reported the approval
   "unevaluated". Now sends its displayed decision (vitest-covered).
3. **Agent steps offered a Writes editor (adjacent fix).** StepPanel
   offered the Writes editor for agent steps, but core P-05 refuses
   writes on anything but record/approval. Removed `agent` from the
   writes fieldset condition, matching the demo.

## Deliberately not done

- **Gap 11, core half (P-03 role existence).** Lane roles are
  `role_wh_*` ids from the demo seed, not AccessRole rows:
  `access_role_service::list` returns UUID ids ("Full Access" /
  "Standard User"); `actor_may_act` compares `r.id == role_id`, so no
  `role_wh_*` id can ever match; there is no builtin "admin" role.
  Extending P-03 would refuse all five warehouse lanes and every
  existing test using `role_wh_*` roles. The frontend picker's
  warning is the fix for gap 11.
- **Write-input `resolve`.** NOT implemented. `WriteOp.inputs` is an
  unstructured map with no place for a `resolve` — only form fields
  resolve (Q-35 as accepted).

## Deliberate follow-ups (left for later work, not dropped)

- **WP-0710 runtime** for `assign` / `skip_when`: the runner honors
  both once WP-0710 lands. The builder and Help say so where each
  field appears.
- **Resolve runtime**: runtime support for `resolve` is a follow-up
  (RAID Q-35 row points it at WP-0705).
- **Cross-object Action Types (WP-0705)**: the 13 remaining
  cross-object P-12 refusals need new invoke effects / a
  warehouse-specific invoke path — not a lane rework.
- **`run_data.set`**: never adopted, per ADR-0025 decision 5.
- **Per-child fixtures UI**: out of scope for this batch (fixtures
  exist; a dedicated UI does not).

## Verification (as reported by the build workers)

- Core: definition 19/19, publish 24/24 (20+4), builder 21/21
  (19+2), builder_http 2/2 (1+1). `cargo fmt` clean; clippy
  warning-free; `validate` 0 errors.
- Frontend: full vitest 213/213 (26 files; builder 60/60); `tsc`
  clean; `npm run build` green (chunk-size warning only,
  pre-existing); `lane-builder.spec.ts` 5/5 against a fresh server
  on a fresh data dir, including the new from-scratch test (object
  picker → five kinds → nested condition → branch → run-data write
  → fixtures → dry run ending `done` → save → reload → publish).
- Demo: `lane-builder-demo.spec.ts` 23/23 (12 existing + 11 new);
  full demo E2E 43/43 (32 baseline + 11 new). `node --check`
  clean; `validate` 0 errors.
- Docs (first worker): `programctl validate` 0 errors,
  `programctl docs --check` 0 problems; STATUS.md regenerated and
  shows WP-0718 done. Docs (this second pass): `programctl
  validate` and `programctl docs --check` re-run after the llms
  P-12 second pass — see "Docs" below for the results.

## Caveats (not hidden)

- The full 3-package Rust suite did NOT complete in-session. The
  no-flag compile fails on a pre-existing `agent_loop_limits.rs`
  test-support issue; with the flag the full run progressed ~1h on
  this 1.5-CPU VM before the worker stopped it; output was piped to
  `tail`, so no pass claim — CI is the full-suite proof. The Tauri
  shell was not run (no GTK libs in this VM). `tsc` was not run by
  the core worker.
- The frontend `lane-builder.spec.ts` is not re-runnable on a dirty
  data dir (pre-existing non-idempotency: the AC-2 test publishes the
  shared seed lane).
- The demo's v0.99.7 Releases entry still says "every warehouse
  Action Type is provisional, so P-12 refuses each one by name" —
  that is a historical entry describing the state when v0.99.7
  shipped, left as-is; the current position lives in `llms.txt` /
  `llms-full.txt` and in this log.

## Docs

First docs worker (items 1–4): WP-0718 closeout, Q-33/Q-34/Q-35
acceptance (proposal doc + RAID rows 42–44), Help
(`/help/process-lanes`, new "Builder additions" section), Releases
v0.99.10, Roadmap first element replaced (WP-0710 second element
unchanged), `llms.txt` / `llms-full.txt` builder-capability entries,
STATUS.md regen. Sitemap: no change (no route changed). The Help /
Releases / Roadmap blocks landed in `app.js` via the demo worker
from the first docs worker's follow-up blocks (verified there
against the landed code).

Second pass (this worker): the llms P-12 sentence. The first pass
had left the stale sentence ("the five warehouse lanes cannot
publish until their Action Types are adopted (every warehouse Action
Type is provisional in the demo, so P-12 refuses each one by
name)") intact in both files. It now reads, in both `llms.txt` and
`llms-full.txt`: "the five warehouse lanes still cannot publish: the
builder's Adopt flow has adopted the three faithful `*.set_status`
types on their own anchors (`purchase_order.set_status`,
`sales_order.set_status`, `cycle_count.set_status`), but P-12 still
refuses 15 writes — six `warehouse.post_stock_move`, one
`purchase_order.close_receipt`, one `purchase_order.create`, one
`purchase_order.set_status` (Replenish, anchored on a `reorder_rule`
with no status row to adopt onto), two `picking.set_status`, one
`shipment.create`, one `sales_order.ship`, and two `run_data.set`.
The thirteen cross-object writes need WP-0705 follow-up (new invoke
effects / a warehouse-specific invoke path); `run_data.set` is never
adopted (ADR-0025 decision 5: setting run data is a step property,
not an Action Type)." No warehouse lane is claimed publishable.

## Files changed (whole batch, working tree, uncommitted)

- Core (13): `desktop/core/src/models/process_lane.rs`,
  `desktop/core/src/services/process_lane_publish.rs`,
  `desktop/core/src/services/process_lane_builder.rs`,
  `desktop/src-tauri/src/commands/process_lane_commands.rs`,
  `desktop/src-tauri/src/lib.rs`,
  `desktop/server/src/dispatch.rs` (adopt arm only),
  `desktop/src/lib/api.ts` (adopt wrapper only),
  `desktop/src/lib/types.ts` (model additions +
  `PublishReport.framing`),
  `program/spec/schemas/process-lane.schema.json`,
  core tests (`desktop/core/tests/process_lane_definition.rs`,
  `process_lane_publish.rs`, `process_lane_builder.rs`),
  `desktop/server/tests/process_lane_builder_http.rs`.
- Frontend (8, +2151/−141):
  `desktop/src/features/builder/LaneBuilder.tsx`,
  `StepPanel.tsx`, `StageChevrons.tsx`, `LaneBuilderPage.tsx`,
  `builderUtils.ts`, `LaneBuilder.test.tsx`,
  `builderUtils.test.ts`,
  `desktop/e2e/tests/lane-builder.spec.ts`.
- Demo (2): `app.js` (+403/−32),
  `desktop/e2e/tests/lane-builder-demo.spec.ts` (+188/−3).
- Docs: `llms.txt`, `llms-full.txt`,
  `program/work/P7/WP-0718-visual-lane-builder.md`,
  `program/spec/schema-extension-proposal-q33-q36.md`,
  `program/RAID.md`, `program/STATUS.md` (regenerated),
  `program/log/2026/11/15-2311-muse-builder-gaps-fix.md` (this log).
- `desktop/test-results/` (untracked, E2E output — not part of the
  batch; leave for the coordinator to decide).

## CI fix on PR #788 (2026-11-16, after the gate's first green check)

- CI run 43855419056 on head aaafa3ef: everything green except
  `rust (tests 3/4)` — exactly one failure out of 323 tests in that
  partition: `lanesra-core::process_lane_branches`
  `a_dead_end_branch_blocks_publish_and_names_the_step`
  (`desktop/core/tests/process_lane_branches.rs:285`).
- Cause: the fixture is a check whose pass/fail branches both target
  the check itself — both a self-loop AND a dead end. Before this
  batch the dead-end refusal ("has no way out") fired; the batch's
  gap-7 P-06 self-loop refusal ("targets the step itself — a check
  that fails would retry forever") now fires first, so the test's
  `assert!(err.contains("no way out"))` failed. Publish was still
  blocked and the step still named — the implementation was right,
  the fixture was stale.
- Fix (test-only, this commit): the hold stage is now a two-check
  cycle (`stuck` <-> `loop`) instead of a self-loop, so the fixture
  is a genuine dead end with no way out and no self-loop. Both
  assertions (`contains "stuck"`, `contains "no way out"`) are
  unchanged — the test still tests WP-0706 AC-2 exactly as named.
  The self-loop refusal itself stays covered by the batch's own
  tests (`builder_check_refuses_a_self_loop_branch_at_p06`,
  `import_refuses_a_self_loop_branch`).
- Lesson: a full /tmp (512M tmpfs, filled by two killed
  `cargo install cargo-nextest` temp dirs) silently zeroed the
  first local repro log — clear /tmp before re-running, and log to
  ~/workspace, not /tmp.
