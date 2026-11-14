---
id: WP-0706
title: Approvals, branches with exits, pause, handoff and cancel
phase: 7
epic: E7.2 Steps and rules
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0704]
requirements: [REQ-LANE-008, REQ-LANE-009, REQ-LANE-010, REQ-LANE-012]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: core runtime; shown to users by WP-0707
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-14
---

# WP-0706 — Approvals, branches with exits, pause, handoff and cancel

## Goal

Cover everything that is not the straight path.

## Scope

- Approval steps through the approvals inbox; declared branches; publish refuses a dead end; pause, resume, handoff; reopen within the current stage; final runs; cancel that lists posted writes; one active run per lane per record; declared exclusions between lanes

## Acceptance criteria

- [ ] AC-1: An approval step waits in the inbox and each response routes the run along its declared branch
- [ ] AC-2: A lane whose hold or rejection branch has no way out cannot be published, and the message names the step
- [ ] AC-3: A second run of the same lane on the same record is refused with a link to the active one; a different lane may start unless excluded
- [ ] AC-4: Cancelling after writes leaves them in place and lists them in the audit trail
- [ ] AC-5: Handoff changes who can act and is audited
- [ ] AC-6: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-7: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_branches.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
