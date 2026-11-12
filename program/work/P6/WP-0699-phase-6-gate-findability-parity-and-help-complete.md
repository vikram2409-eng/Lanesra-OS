---
id: WP-0699
title: Phase 6 gate: findability, parity and help complete
phase: 6
epic: E6.7 Track gate
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0606, WP-0607, WP-0608, WP-0613, WP-0614, WP-0632, WP-0650, WP-0651]
requirements: [REQ-ADM-009, REQ-NFR-007]
estimate: M
editions: []
delivered: []
branch: 
pr: 
updated: 2026-11-11
---

# WP-0699 — Phase 6 gate: findability, parity and help complete

## Goal

Prove the track is finished in every edition before the old tools are retired.

## Scope

- Findability test result; parity walkthrough of desktop, Team Workspace and online demo; help audit; gate file G6

## Acceptance criteria

- [ ] AC-1: The findability test meets 80% first-attempt success on all ten tasks
- [ ] AC-2: A walkthrough of ten admin tasks gives the same result in desktop, Team Workspace and the online demo (simulated writes)
- [ ] AC-3: The help audit finds no article with an old menu path and an article for every new tool

## Test plan

- Evidence recorded in `program/gates/G6-experience-and-admin.md`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
