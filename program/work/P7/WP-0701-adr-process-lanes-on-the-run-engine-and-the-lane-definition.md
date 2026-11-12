---
id: WP-0701
title: ADR: Process Lanes on the run engine, and the lane definition schema
phase: 7
epic: E7.1 Lane foundation
status: ready
owner: unassigned
planned_for: muse
depends_on: []
requirements: [REQ-LANE-001, REQ-LANE-003]
estimate: M
editions: []
delivered: []
branch: 
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
- [ ] AC-2: The JSON Schema validates the Receive Shipment definition from the spec's Part B
- [ ] AC-3: The ADR lists what the demo may do before the editions and what it may not (no behaviour the core will not have)

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
