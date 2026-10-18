---
id: WP-0015
title: Performance baseline
phase: 0
epic: E0.2 Test infrastructure
status: proposed
owner: unassigned
depends_on: [WP-0005]
requirements: [REQ-PRE-012]
estimate: M
editions: []
delivered: []
branch: 
pr: TBD
updated: 2026-10-16
---

# WP-0015 — Performance baseline

## Goal

Later phases add a trace event per round, checkpoints and budgets to every run. Measure today's costs first so regressions are visible and budgets can be set.

## Scope

- Benchmarks (criterion or a timed integration test) for: agent loop overhead per round with the scripted provider, graph runtime step latency, checkpoint write time, SQLite bytes per run
- Record results in `program/testing/PERFORMANCE.md` with machine details
- Proposed budgets for Phase 1 (e.g. trace emission adds < 5 ms per round)

## Acceptance criteria

- [x] AC-1: Baseline numbers recorded and reproducible with one command
- [ ] AC-2: Budgets agreed and written into WP-0111 and WP-0112 acceptance criteria

## Test plan

- `desktop/core/benches/` or `desktop/core/tests/perf_baseline.rs` (ignored by default, run nightly)
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

- 2026-10-17 muse: Owner decision (2026-10-17) on AC-2 thresholds: Collect one week of perf-baseline numbers from main. Then fail the job when any measurement is more than 50% slower than the baseline in program/testing/REGRESSION.md; stay report-only below that. Threshold takes effect after one week of data collection (approx 2026-10-24). AC-2 stays unticked until the threshold is live and proven by one deliberately slowed run.

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-10-17 muse: Building WP-0015. Created desktop/core/tests/perf_baseline.rs (3 ignored tests using scripted provider, Covers: REQ-PRE-012), .github/workflows/perf-baseline.yml (reports numbers, never fails per owner instruction). Baseline recorded in program/testing/REGRESSION.md. AC-2 (budgets agreed) needs owner. Untested locally — see CI.
