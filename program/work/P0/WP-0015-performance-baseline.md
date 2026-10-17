---
id: WP-0015
title: Performance baseline
phase: 0
epic: E0.2 Test infrastructure
status: ready
owner: muse
depends_on: [WP-0005]
requirements: [REQ-PRE-012]
estimate: M
editions: []
delivered: []
branch: wp/WP-0015-perf-baseline
pr: 
updated: 2026-10-17
---

# WP-0015 — Performance baseline

## Goal

Later phases add a trace event per round, checkpoints and budgets to every run. Measure today's costs first so regressions are visible and budgets can be set.

## Scope

- Benchmarks (criterion or a timed integration test) for: agent loop overhead per round with the scripted provider, graph runtime step latency, checkpoint write time, SQLite bytes per run
- Record results in `program/testing/REGRESSION.md` with machine details (per owner instruction 2026-10-17)
- Proposed budgets for Phase 1 (e.g. trace emission adds < 5 ms per round)

## Acceptance criteria

- [ ] AC-1: Baseline numbers recorded and reproducible with one command
- [ ] AC-2: Budgets agreed and written into WP-0111 and WP-0112 acceptance criteria

## Test plan

- `desktop/core/tests/perf_baseline.rs` (ignored by default, run with -- --ignored; uses the scripted provider from WP-0005 for deterministic numbers)
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

## Refinement log

- 2026-10-17 muse: Moved proposed → ready. Scope clear: benchmarks with scripted provider (deterministic), baseline recorded in program/testing/REGRESSION.md (owner override of PERFORMANCE.md). AC-1 testable via `cargo test -- --ignored perf_baseline`. AC-2 (budgets agreed) is manual. Depends on WP-0005 (done). REQ-PRE-012 listed. Size M.
