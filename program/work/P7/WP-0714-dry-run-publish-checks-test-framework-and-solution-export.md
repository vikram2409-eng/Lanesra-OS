---
id: WP-0714
title: Dry run, publish checks, test framework and Solution export
phase: 7
epic: E7.5 Authoring
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0705, WP-0706]
requirements: [REQ-LANE-021]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: wp/WP-0714-design-spec-r2
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/778
updated: 2026-11-14
---

# WP-0714 — Dry run, publish checks, test framework and Solution export

## Goal

Make a lane safe to publish and portable.

## Scope

- Dry run with writes simulated; publish checks for bindings, roles, branch exits and replaced workflows; lane cases in the Unified Test Run; Solution export and import

## Design

Design spec for this WP, written 2026-11-14 on the ADR-0025 prep track
(Claude's instruction, ferried by the owner). Docs only: this WP is
dependency-blocked on WP-0705 (record/check writes) and WP-0706
(approvals, branches with exits, pause/handoff/cancel), and the ADR-0025
demo freeze holds until WP-0707 ships on desktop + Team Workspace. The
build starts when the engine clears; this section is the design it builds
from. Status stays `proposed` until then.

### 1. What exists today

The publish path already enforces part of this WP, in three layers. The
design below does not re-litigate them; it names them, and adds the checks
that are missing.

- `validate_definition` (`desktop/core/src/models/process_lane.rs`,
  WP-0701): lane id/name/version/status shape; at least one stage, at
  least one step per stage; stage and step ids unique and kebab-case;
  entry/skip conditions well-formed; run-data fields have no duplicate
  keys and known types; every step has a role except check steps, which
  must not have one; approval steps declare branches and only
  check/approval steps may; record and approval steps have at least one
  write; **every branch target resolves to a real stage and step**.
- `validate_compiled` (`process_lane_service.rs`, WP-0703): every
  compiled edge references a known node; then the graph's own
  `validate_for_publish` (at least one node, exactly one trigger).
- `publish_lane` (WP-0703): admin-only; refuses when the lane has no
  draft version; parses the definition, compiles the lane, runs
  `validate_compiled`, creates the graph (source id `{lane}@v{n}`), and
  flips the version to published. Runs already started keep their
  version's graph; new runs use the new one.
- Solution export (WP-0703): the export arm in
  `industry_package_service` writes the lane's published definition.
  Import is a follow-up: installing a `process_lanes` entry from a
  Solution manifest creates a new draft version here.

What none of those layers does today: name a failed check in a way the
author can act on (an import refusal is a bare validation error, not a
publish report); check that a branch has a way out; check that a
`$run_data` / `$record` / `$item` reference names something declared;
check that a `replaces` reference names a real workflow action; or check
that a write's Action Type is adopted rather than provisional. Those are
the new checks in §2.

### 2. Publish checks — the full enumeration

Publish is refused when any REFUSE check fails, with one named reason
per failed check (AC-2). A WARN does not block publish; it is listed in
the publish report and shown live in the builder (WP-0718). Checks
already enforced today are marked `[exists]`; the WP-0714 work for
those is to surface them as named publish failures, not to re-implement
them.

| ID | Check | Today | Refusal / warning reason |
|---|---|---|---|
| P-01 | Lane id starts `lane-` + kebab-case; name non-empty; version ≥ 1; status draft or published | [exists] `validate_definition` | `lane id must start with 'lane-' followed by kebab-case`, etc. |
| P-02 | At least one stage; at least one step per stage; stage and step ids unique and kebab-case | [exists] `validate_definition` | `stage '<id>' needs at least one step`; `step id '<id>' is used twice in stage '<stage>'` |
| P-03 | Every step has a role, except check steps, which must not have one | [exists] `validate_definition` | `step '<id>' needs a role`; `check step '<id>' must not have a role` |
| P-04 | Approval steps declare branches; only check and approval steps declare branches | [exists] `validate_definition` | `approval step '<id>' must declare its branches`; `only check and approval steps declare branches` |
| P-05 | Record and approval steps have at least one write | [exists] `validate_definition` | `record step '<id>' must declare at least one write` |
| P-06 | Every branch target names a real stage and step | [exists] `validate_definition` | `step '<id>' branches to unknown step '<stage>/<step>'` |
| P-07 | Start, stage-entry and stage-skip conditions are well-formed (condition grammar: known operators, resolvable field paths) | [exists] `validate_definition` | the condition's validation error, naming the condition and step |
| P-08 | Run-data fields: no duplicate keys; known types (`string`, `integer`, `number`, `boolean`, `date`) | [exists] `validate_definition` | `run_data declares field '<key>' twice`; `run_data field '<key>' has unknown type '<type>'` |
| P-09 | **Branch exits.** Every check declares both a pass and a fail outcome; every approval declares approve and reject. Every hold/reject branch declares a way out: continue to a named step, or end with a named outcome. A branch with no exit refuses publish. | Partly done — WP-0706's `validate_every_step_reaches_an_end` (in `validate_definition`, `process_lane.rs`) already refuses a lane in which a step can never reach the end, naming the step. What is still new in WP-0714 is the outcome-coverage half: every check declaring both pass and fail, every approval declaring approve and reject. | `check '<id>' has no fail branch`; `approval '<id>' has no reject branch`; `branch '<on>' of step '<id>' has no exit — declare a target step or an end outcome`; the reachability refusal names the stuck step |
| P-10 | **Check last in stage (WARN).** A check step that is not the last step of its stage warns, because the runner takes the next step in definition order on pass — if the steps after the check are the hold branch's steps, the happy path runs them. This exact bug happened on Pick & Pack and was fixed by moving the check to the end of its stage. A warning, not a refusal: a mid-stage check is legitimate when the next step is the pass continuation. The dry run (§3) makes the consequence visible. | NEW | `check '<id>' is not the last step of stage '<stage>' — the step after it runs on the pass path; move the check to the end of the stage, or confirm the next step is the pass continuation` |
| P-11 | **No undeclared run-data references.** Every `$run_data.<field>` in a `binds_to`, a write input, a condition or a validation names a field declared in `lane.run_data.fields` (or a `per_child` field for a repeating step). Every `$record.<field>` names a field on the anchor object; every `$item.<field>` names a field on the `repeat_over` children. A typo in a path fails silently today — the schema checks the path's shape, not that the field exists. | NEW | `step '<id>' references undeclared run-data field '<field>'`; `step '<id>' references unknown record field '<field>' on '<object>'`; `step '<id>' references unknown child field '<field>'` |
| P-12 | **No provisional Action Types in a publishable lane.** Every `writes[].action_type` must be in the WP-0705 catalog (adopted). A provisional id refuses publish. Consequence, stated plainly: the four warehouse lanes cannot publish until WP-0705 adopts their Action Types (`purchase_order.create`, `purchase_order.set_status`, `purchase_order.close_receipt`, `warehouse.post_stock_move`, `sales_order.set_status`, `picking.set_status`, `shipment.create`, `sales_order.ship`, `cycle_count.set_status`, and the provisional `run_data.set`, which ADR-0025 decision 5 says is not an Action Type in the core at all — the demo keeps it only until WP-0705 fixes the rule, RAID Q-32). | NEW | `step '<id>' uses provisional Action Type '<type>' — not in the adopted catalog` |
| P-13 | **Replaced workflows exist.** Every `replaces[]` entry names a real workflow id and a real action index on it. Today nothing checks the reference. | NEW | `step '<id>' replaces unknown workflow '<workflow_id>'`; `step '<id>' replaces action <n> of workflow '<workflow_id>', which has no such action` |
| P-14 | **Run-data types match their uses (WARN).** A field bound to a record field or used in a numeric comparison must have a compatible type (for example `counted_qty` is `integer`, not `string`). | NEW | `run-data field '<key>' is '<type>' but is used as '<needed>' by step '<id>'` |
| P-15 | **Start condition references declared fields.** The lane's `start_condition` may only reference declared run-data fields and anchor-object fields (same rule as P-11, named separately because it gates the whole lane). | NEW | `start_condition references undeclared field '<field>'` |
| P-16 | **Only a draft publishes.** Publishing a lane with no draft version refuses (the draft is the thing being published). | [exists] `publish_lane` | `lane '<key>' has no draft version to publish` |
| P-17 | **Runs keep their version (guarantee, not a check).** Publishing a new version does not disturb runs already started on the old version — they keep their graph id (`{lane}@v{n}`); only new runs use the new version. | [exists] WP-0703 | — |

Publish report shape: the refusal lists every failed check (not just the
first), each with its id (P-01..P-16), the step it belongs to, and the
named reason. The author fixes all of them and republishes; there is no
partial publish.

### 3. Dry run — what it asserts against a seed record

A dry run walks the lane against a seed record and writes nothing
(AC-1). It is the author's preview before publish, and the place the
check-not-last bug (P-10) becomes visible instead of silent.

- **Seed record.** The anchor record and its children come from the
  workspace's seed data — for the warehouse lanes, the demo seed (for
  example PO-2026-001 with its two lines, 40× HG-1001 and 30× HG-1002).
  The dry run reads the seed record; it never writes it.
- **Fixtures.** The author supplies the values a person would type:
  run-data fields (for example `carrier`, `delivery_note_number`) and
  per-child values, so the dry run can evaluate form-dependent checks.
  A check that depends on a fixture the author did not supply reports
  `unevaluated (needs fixture)` instead of guessing.
- **The walk.** In definition order: evaluate the start condition
  (a refusal reports `would refuse to start, <reason>`); evaluate each
  stage's entry/skip condition; evaluate each check against the seed
  record's values plus the fixtures; follow the branch the outcome
  selects; record which steps execute, which are skipped (and why), and
  which outcome the run would end with.
- **Simulated writes.** For each record step, resolve every `$`
  reference against the seed record plus the fixtures and report the
  exact writes it would post — Action Type, inputs with values, and per
  child row for a repeating step — without posting anything: no
  StockMove, no status change, no ledger row, no approval created. A
  write whose `$` reference does not resolve is reported as a failure,
  not skipped.
- **What it asserts.**
  1. The walk reaches an end (no dead end, no infinite loop). Repeating
     steps are bounded by `LANE_REPEAT_MAX_ITERATIONS`.
  2. Every write's `$` references resolve against the seed record and
     the fixtures.
  3. Every check's condition evaluates to pass or fail — never
     `unknown` (an `unknown` means a missing fixture or an undeclared
     reference, and names which).
  4. No step would post a zero-quantity or no-op write. The Count
     lane's post-adjustments step is the case: zero-variance lines
     would post a zero-quantity COUNT_ADJUSTMENT, which is not a real
     adjustment. The dry run flags each such line and reports
     `would skip <n> children with nothing to post` (the Q-34 filter,
     once the schema has it, is what makes the skip declarative).
  5. The simulated end state matches the author's expectation. The
     author names the expected outcome when starting the dry run; the
     dry run asserts the walk ends with it and reports the actual
     outcome when it does not.
- **What it reports.** The full path: stages visited, steps executed /
  skipped (with reasons), checks passed / failed (with reasons),
  branches taken, writes simulated (with resolved inputs), and the end
  outcome. The report is the evidence the author reads before
  publishing; it is also what the builder (WP-0718) shows as the live
  preview.

The dry run needs WP-0705 (write `$`-reference resolution) and WP-0706
(branch outcomes, approvals) to walk the lane — hence this WP depends
on both. It does not need the runner (WP-0707): it is a simulation
against a seed record, not a run.

### 4. Test framework — the lane case assertion format

Lane cases run in the Unified Test Run (AC-1..AC-3's test plan,
`desktop/core/tests/process_lane_publish.rs`). A case is a named test
against a seeded workspace; it runs in a transaction that rolls back,
so cases are repeatable and leave no trace.

Case shape:

```json
{
  "case": "receiving-happy-path",
  "lane": "lane-wh-receiving",
  "version": 1,
  "anchor": { "object": "purchase_order", "id": "po-2026-001" },
  "fixtures": { "run_data": { "carrier": "FastFreight", "delivery_note_number": "DN-8812" } },
  "assert": [
    { "run.ends_with": "received" },
    { "step.completed": "record-receipt" },
    { "step.completed": "close-po" },
    { "write.posted": { "action_type": "warehouse.post_stock_move", "count": 4 } },
    { "write.posted": { "action_type": "purchase_order.close_receipt", "inputs": { "po_id": "po-2026-001" } } },
    { "record.equals": { "object": "purchase_order", "id": "po-2026-001", "field": "status", "value": "RECEIVED" } }
  ]
}
```

Assertion kinds (each names the step, check or write it covers; a
failing assertion reports expected vs actual):

| Assertion | Meaning |
|---|---|
| `run.ends_with: <outcome>` | The run ends with this named outcome. |
| `step.<id>.completed` | The step completed. |
| `step.<id>.skipped: <reason>` | The step was skipped, for this reason (stage skip, optional skip, child filter). |
| `check.<id>.passed` / `check.<id>.failed: <reason>` | The check evaluated to pass / fail, with the reason it reports. |
| `write.posted: {action_type, inputs?, count?}` | A write with this Action Type was posted, with these resolved inputs (when given) and this many rows (when given). |
| `write.not_posted: {action_type, reason}` | No write with this Action Type was posted, for this reason — the assertion for the Q-34 case (zero-variance lines post nothing). |
| `record.equals: {object, id, field, value}` | The anchor record (or a related record) has this field value after the run. |
| `run_data.equals: {field, value}` | A run-data field has this value after the run. |

Cases live with the lane: they are part of the lane's Solution export
(§5), so a lane carries its own tests wherever it goes. The Unified
Test Run discovers them from the export and runs each case against the
seeded workspace.

### 5. Solution export shape

A lane exports and imports in a Solution with its versions (AC-3).
The manifest entry for a lane:

```json
{
  "lane": {
    "id": "lane-wh-receiving",
    "name": "Receive Shipment",
    "object_key": "purchase_order",
    "versions": [
      { "version": 1, "status": "published", "definition": { "..." : "..." }, "cases": [ { "..." : "..." } ] },
      { "version": 2, "status": "draft", "definition": { "..." : "..." }, "cases": [ ] }
    ]
  },
  "dependencies": {
    "objects": ["purchase_order", "stock_move"],
    "workflows": ["wf-wh-01"],
    "action_types": ["purchase_order.set_status", "purchase_order.close_receipt", "warehouse.post_stock_move"]
  }
}
```

Rules:

- **All versions export, not just the published one.** A draft version
  travels with the lane so the importer sees the work in progress; the
  published version is the one that runs.
- **Cases export with the version they test.** A case names its lane
  version; importing a version without its cases loses the test
  coverage, so the export refuses to drop them.
- **Dependencies are declared, not embedded.** The export lists the
  objects, workflows and Action Types the lane touches. On import, each
  dependency is checked against the target workspace: a missing object
  or workflow is reported, not silently skipped; a missing Action Type
  is reported as a provisional (P-12) — the lane imports as a draft and
  cannot publish until the catalog has it.
- **Import creates a new draft version, never overwrites a published
  one.** Installing a `process_lanes` entry from a Solution manifest
  stores the definition as a new draft version here (WP-0703's
  follow-up). Import runs the publish checks in report mode: it lists
  every failure without refusing the import, so the importer sees what
  is broken before trying to publish. A lane that fails P-09, P-11 or
  P-12 on import is a draft with a known repair list, not a broken
  publish.

### 6. Grounding — the five hardest things, and what covers each

From `program/research/lane-authoring-log.md` ("The five things that
were hardest to write by hand"), ranked hardest first. Each one is
covered by a check, a dry-run assertion, or a builder surface in this
spec or in WP-0718.

1. **Nested any/all check conditions** (Pick & Pack damage check: fully
   picked AND (undamaged OR damage-approved), with the flag set by
   another step's writes). Covered by P-11 (every `$run_data` reference
   must name a declared field — the flag and the check cannot drift
   apart), by the dry run's assertion 3 (the condition must evaluate,
   never `unknown`), and by the WP-0718 condition builder, which shows
   the group structure and links each reference to the step that sets
   it.
2. **Branch targets and step order.** Covered by P-06 (targets resolve),
   P-09 (every branch has an exit), and P-10 (the check-last warning)
   plus the dry run, which walks the happy path and shows the hold
   steps executing when the check is not last.
3. **Run-data plumbing.** Covered by P-11 (no undeclared references)
   and P-14 (types match their uses), and by the WP-0718 per-step
   reads/writes map, which lists what each step reads and writes and
   refuses a reference to a field that is not declared.
4. **The approval step's writes.** Covered by P-05 (an approval has at
   least one write), P-12 (the write's Action Type must be adopted —
   the Pick & Pack approval's provisional `run_data.set` blocks publish
   until WP-0705 settles it), and the WP-0718 approval-writes view,
   which shows what an approval writes and what happens on each branch
   before the author publishes.
5. **Provisional Action Types.** Covered by P-12 (no provisional id in
   a publishable lane) and by the WP-0718 Action-Type picker, which
   offers the adopted catalog instead of a free-text id.

### 7. Out of scope for this WP

- No schema change. The checks in §2 are enforced by the compiler and
  the publish path, not by adding fields to
  `process-lane.schema.json`. Schema extensions (per-item assignment,
  the zero-variance filter, a supplier lookup, a Counter role) are a
  separate proposal: `program/spec/schema-extension-proposal-q33-q36.md`
  (RAID Q-33..Q-36), for the owner to accept.
- No engine change. The dry run simulates; it does not execute. The
  runner is WP-0707; the condition and loop evaluation the dry run
  needs is WP-0710.
- No demo change (ADR-0025 demo freeze). The demo E2E for this WP
  (AC-5) is written when the freeze lifts, against the same behaviour
  on the warehouse sample data.

- [ ] AC-1: A dry run walks the lane and writes nothing
- [ ] AC-2: Publish is refused with a named reason for each failed check
- [ ] AC-3: A lane exports and imports in a Solution with its versions
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_publish.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-14 muse: Design spec written as a Design section (ADR-0025 prep track, Claude's instruction via the owner). Publish checks P-01..P-17 enumerated (P-01..P-08 already enforced by validate_definition / validate_compiled / publish_lane; P-09..P-15 new), dry-run assertions, lane-case assertion format, Solution export shape; grounded in the five hardest items from program/research/lane-authoring-log.md. Docs only — no core, no app.js, no schema change. WP stays proposed and unassigned: the build is blocked on WP-0705 + WP-0706 and the ADR-0025 demo freeze. ACs untouched.
