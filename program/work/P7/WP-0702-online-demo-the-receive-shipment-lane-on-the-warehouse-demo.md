---
id: WP-0702
title: Online demo: the Receive Shipment lane on the warehouse demo
phase: 7
epic: E7.6 Shipped lanes
status: in_progress
owner: muse
planned_for: muse
depends_on: [WP-0701]
requirements: [REQ-LANE-025, REQ-LANE-014, REQ-NFR-008]
estimate: L
editions: [demo]
delivered: []
editions_reason: owner decision (ADR-0023): the online demo previews Process Lanes first; desktop and Team Workspace follow in WP-0703 to WP-0707 from the same definition format
docs: [process-lanes]
branch: wp/WP-0702-receive-shipment-lane-demo
pr: 
updated: 2026-11-12
---

# WP-0702 — Online demo: the Receive Shipment lane on the warehouse demo

## Goal

Prove the feature where a visitor can try it: one screen that receives PO-2026-001 with real changes to the demo's records.

## Scope

- Lane definition `lane-wh-receiving` as data in the shared schema; runner screen; real writes to the warehouse demo store (purchase order lines, stock moves, stock on hand, status); discrepancy branch; seeded run on PO-2026-005 waiting for review; reset clears runs and keeps definitions; entry points from the dashboard card, the purchase order and the Receiving Agent page

## Acceptance criteria

- [ ] AC-1: A visitor completes PO-2026-001 on the lane screen alone and the end state matches the spec: status RECEIVED, stock 88 and 52, four new stock moves
- [ ] AC-2: Entering more than ordered on a line blocks the step with the reason shown and writes nothing
- [ ] AC-3: The seeded PO-2026-005 run waits for the manager; approving puts away 18 and leaves the order partially received; holding posts nothing and offers its declared way out
- [ ] AC-4: Leaving and reopening returns to the current step; Reset restores the seed
- [ ] AC-5: The definition validates against the shared JSON Schema (CI check)
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- Demo E2E in `desktop/e2e` for the lane; `node --check app.js`; schema validation of the seeded definition
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Exception to REQ-PRE-021, recorded in ADR-0023.
- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-11-12 muse: Claimed — the owner reassigned this WP from claude to muse ("build WP-0702 and WP-0703, assign it yourself"). Status proposed → in_progress on branch wp/WP-0702-receive-shipment-lane-demo. ACs reviewed and stand as written. Next: read ADR-0023, ADR-0024, the spec Part B and SOLUTION-DESIGN.md §17, then build the demo lane. Note: this branch stacks on the WP-0701 branch (PR #755, program check green, CI/security running) — rebase onto main after #755 merges.
