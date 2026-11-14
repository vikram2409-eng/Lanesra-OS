---
id: WP-0705
title: Record and check steps: real, atomic, repeat-safe writes
phase: 7
epic: E7.2 Steps and rules
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0703, WP-0141]
requirements: [REQ-LANE-006, REQ-LANE-007]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: core runtime; shown to users by WP-0707
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-14
---

# WP-0705 — Record and check steps: real, atomic, repeat-safe writes

## Goal

Make "writes are real" safe: one step, one outcome, never twice.

## Scope

- Record steps call Action Types; a step's writes commit together; idempotency key per step attempt (WP-0141); check steps with the shared conditions; required and optional steps; skip with reason

## Acceptance criteria

- [ ] AC-1: A step with several writes either applies all of them or none (failure injected mid-step)
- [ ] AC-2: Completing the same step twice, or resuming after a crash, does not repeat a write
- [ ] AC-3: A Business Rule that blocks the write blocks the step with its message
- [ ] AC-4: A failed check blocks advance with the reason; a required step cannot be skipped; a skip of an optional step is recorded
- [ ] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_writes.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
