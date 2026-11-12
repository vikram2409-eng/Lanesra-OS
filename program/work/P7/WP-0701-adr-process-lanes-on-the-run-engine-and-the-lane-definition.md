---
id: WP-0701
title: ADR: Process Lanes on the run engine, and the lane definition schema
phase: 7
epic: E7.1 Lane foundation
status: in_progress
owner: muse
planned_for: muse
depends_on: []
requirements: [REQ-LANE-001, REQ-LANE-003]
estimate: M
editions: []
delivered: []
branch: wp/WP-0701-lane-model-and-schema
pr: 
updated: 2026-11-12
---

# WP-0701 — ADR: Process Lanes on the run engine, and the lane definition schema

## Goal

Fix the model before the demo and the product are built, so both use one definition format and the lane does not become a second engine.

## Scope

- ADR: lane, stage, step and run data; how a lane compiles to an execution graph (source kind `lane`, stage as grouping); the new human step node; statuses from ADR-0007 plus waiting for a person; labelling of the live item; the "replaces" rule for workflows; cancel, hold and branch exits
- A JSON Schema for the lane definition, used by `app.js` and the Rust core
- The demo-first exception to REQ-PRE-021 for this feature, with its limits

## Out of scope

- Any product or demo code (WP-0702 onward). This package delivers a decision record, a JSON Schema, one example definition and a CI check.

## Deliverables

- `program/decisions/ADR-0024-process-lane-model-and-definition-schema.md` (next free ADR number at the time of writing; use the template)
- `program/spec/schemas/process-lane.schema.json`
- `program/spec/schemas/examples/lane-wh-receiving.json`: the Receive Shipment lane of the spec's Part B, with the ADR-0023 changes applied
- A schema check wired into the `program` CI job, so an invalid example fails the build

## Acceptance criteria

- [ ] AC-1: The ADR is Accepted and maps every part of the owner's Process Lane spec (A3.1 to A3.9) to the model, noting each change from the spec
- [x] AC-2: The JSON Schema validates the Receive Shipment definition from the spec's Part B
- [x] AC-3: The ADR lists what the demo may do before the editions and what it may not (no behaviour the core will not have)

## Test plan

- `programctl validate`; schema lint in CI
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Source: the owner's Process Lane spec, [`program/spec/process-lane-spec.md`](../../spec/process-lane-spec.md), and ADR-0023, which changes it in ten places (decision 4) and wins where they differ. The spec's reference to "WP-0114" as its pattern is not a package in this program; WP-0114 here is the external agent ADR.
- Runtime today: `desktop/core/src/models/execution_graph.rs` (`NODE_TYPES` has condition, action, agent, approval, loop; no human step), `action_type_service::invoke`, `approval_service`.
- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refined and moved to `ready` on 2026-11-12; planned for Muse AI by the owner.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-11-12 muse: **Done** — ADR-0024 (Proposed) in `program/decisions/`; `program/spec/schemas/process-lane.schema.json` (draft 2020-12, follows the team-format precedent); `program/spec/schemas/examples/lane-wh-receiving.json` (Receive Shipment, 5 stages, 12 steps, 8 run-data fields); `program/tools/check_lane_schema.py` + 5 self-tests + a new `program` CI job step (`pip install "jsonschema>=4.10,<5"` then the check). **State** — AC-2 proven locally (`jsonschema.validate` passes on the example; 9 negative tests prove the schema rejects a check step with a role, a form step with writes, a branch with no exit, a bad expression/operator, an approval without branches, a bad lane id, and a record/approval step without writes); the gate was proven to fail on a corrupted example (exit 1 with the exact location) and pass again after restore. AC-3 is ADR §13. AC-1 is NOT ticked: the ADR is Proposed until the owner accepts it. `programctl validate` 0 errors; program-tools self-tests 77 passing (was 70 after WP-0200; +5 mine, +2 from other merged work); REGRESSION.md bullet added. **Next** — owner accepts or amends ADR-0024; then set `status: in_review` → `done` on merge, and WP-0702/WP-0703 start from this schema. **Gotchas** — (1) Local clone was stale (October); fetched origin/main into a fresh worktree `~/workspace/lanesra-build/wp0701` because local main carries unpushed tracker commits (WP-0122/WP-0130) and cannot fast-forward. (2) `jsonschema` is NOT a CI dependency anywhere in the repo, so the program.yml step installs it explicitly; the check scripts are otherwise stdlib-only by convention. (3) The spec's hybrid "form + record" steps (3.1, 3.2) split into form + record steps so each step compiles to exactly one graph node — recorded in ADR §3 and the A3.2 row. (4) Action Type ids in the example are provisional until WP-0705 adopts them. (5) `program/STATUS.md` was regenerated with `programctl status` (the program-bot will re-commit it on the PR branch if it drifts).
