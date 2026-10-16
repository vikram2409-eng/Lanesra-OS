---
id: WP-0016
title: Fix tests failing on main
phase: 0
epic: E0.1 Program and engineering readiness
status: ready
owner: unassigned
depends_on: []
requirements: [REQ-PRE-001]
estimate: S
branch: 
pr: 
updated: 2026-10-16
---

# WP-0016 — Fix tests failing on main

## Goal

CI cannot become a required check while main is red. The baseline run on 2026-10-16 found failing tests (see testing/REGRESSION.md and RAID I-03). Root-cause and fix each one in product code or, if the test is wrong, fix the test with a written justification.

## Scope

- Every test listed as failing in testing/REGRESSION.md
- Root cause written in the Progress log for each (e.g. why the Auto Service 'repair completed' workflow no longer closes the originating appointment)
- No test skipped, ignored or loosened

## Acceptance criteria

- [ ] AC-1: `cargo test --no-fail-fast -p lanesra-core -p lanesra-server -p lanesra-cli` passes with zero failures
- [ ] AC-2: Each fix has a one-paragraph root cause in the Progress log
- [ ] AC-3: REGRESSION.md baseline updated to all-green

## Test plan

- Re-run the full suite three times to rule out flakiness; paste totals into the PR
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
