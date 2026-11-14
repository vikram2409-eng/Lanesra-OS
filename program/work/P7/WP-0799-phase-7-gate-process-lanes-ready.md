---
id: WP-0799
title: Phase 7 gate: Process Lanes ready
phase: 7
epic: E7.7 Track gate
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0708, WP-0709, WP-0713, WP-0714, WP-0717, WP-0720]
requirements: [REQ-NFR-008]
estimate: M
editions: []
delivered: []
branch: 
pr: 
updated: 2026-11-14
---

# WP-0799 — Phase 7 gate: Process Lanes ready

## Goal

Prove a person can do a whole job on one screen in every edition.

## Scope

- First-time-user test of Receive Shipment; parity walkthrough across editions; gate file G7

## Acceptance criteria

- [ ] AC-1: A first-time user completes Receive Shipment on the lane screen alone and the records match the worked example
- [ ] AC-2: The same walkthrough gives the same result in desktop, Team Workspace and the online demo
- [ ] AC-3: `programctl trace --fail-on-gaps` passes for Phase 7 requirements

## Test plan

- Evidence recorded in `program/gates/G7-process-lanes.md`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
