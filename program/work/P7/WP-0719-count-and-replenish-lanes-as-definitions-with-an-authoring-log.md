---
id: WP-0719
title: Count and Replenish lanes as definitions, with an authoring log
phase: 7
epic: E7.6 Shipped lanes
status: done
owner: muse
planned_for: muse
depends_on: [WP-0701]
requirements: [REQ-LANE-025]
estimate: S
editions: []
delivered: []
branch: wp/WP-0719-count-and-replenish-lanes
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/767
updated: 2026-11-14
---

# WP-0719 — Count and Replenish lanes as definitions, with an authoring log

## Goal

Write the two remaining warehouse lanes as data only, to test the definition format on new shapes of work and to record where authoring by hand is hard. That record decides when the visual builder is worth building (ADR-0025 decision 8).

## Scope

- `program/spec/schemas/examples/lane-wh-count.json`: the Count lane on `cycle_count` (count lines, variance review, manager approval, count adjustment)
- `program/spec/schemas/examples/lane-wh-replenish.json`: the Replenish lane on `reorder_rule` (triggered rule, draft purchase order, manager approval)
- Both validated against `process-lane.schema.json` by the existing CI check
- `program/research/lane-authoring-log.md`: for each of the four warehouse lanes, what was hard to express, what the schema could not express, and what a builder would need to show

## Out of scope

- Any runner, demo or core behaviour for these lanes (ADR-0025 decision 4, the demo freeze). No change to `app.js`.
- Changing the schema. If a lane cannot be expressed, record it in the log and add a `Q-` item to `program/RAID.md`.

## Acceptance criteria

- [x] AC-1: Both definitions validate against the schema in CI
- [x] AC-2: The Count lane uses a repeating step over count lines and states, in the log, whether several counters working at once can be expressed with per-item assignment (ADR-0025 decision 6) or needs parallel steps
- [x] AC-3: Every branch in both lanes has a declared way out, and each step that replaces a workflow action declares it
- [x] AC-4: The authoring log covers all four warehouse lanes and ends with a ranked list of the five things that were hardest to write by hand

## Test plan

- The schema check in the `program` CI job (extended to the two new examples if it lists files by name)
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- ADR-0023, ADR-0024, ADR-0025. Source for the two journeys: `program/spec/process-lane-spec.md` section B8.
- Pick and Pack already exists as a definition from pull request #762.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-14 muse: Built — both definitions validate against the schema (4 examples validate, up from 2); branch targets verified to resolve, no `run_data.set` in either new definition, each step that replaces a workflow action declares it (Count: wf-wh-06 action 0; Replenish: wf-wh-02 action 0). Authoring log written covering all four lanes, ending with the five hardest things. AC-2 answered in the log: several counters at once is per-item assignment (WP-0710), not parallel steps — but the schema cannot declare it yet (Q-33). Gaps raised as Q-33..Q-36. No app.js or core change (demo freeze). Next: PR opened; CI is the final proof (schema check on the two new examples).
- 2026-11-14 muse: PR #767 opened (draft). Status in_progress → in_review. CI is the final proof.
- 2026-11-14 muse: PR #767 merged (2b3c5ab). WP-0719 done. Delivered is [] — definitions and the authoring log only, no product edition to deliver (editions: []). Q-33..Q-36 stay Open; the schema-extension proposal for them is program/spec/schema-extension-proposal-q33-q36.md (separate prep-track PR).
