---
id: WP-0004
title: Rust format and lint baseline
phase: 0
epic: E0.1 Program and engineering readiness
status: ready
owner: unassigned
depends_on: []
requirements: [REQ-PRE-001]
estimate: S
editions: []
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0004 — Rust format and lint baseline

## Goal

Make `cargo fmt --check` and `cargo clippy -D warnings` meaningful gates by bringing the existing code to a clean baseline (or recording a documented allow-list) before CI enforces them.

## Scope

- Run fmt and clippy across the workspace; fix mechanical findings in a formatting-only commit
- For findings not safely fixable now, add targeted `#[allow]` with a reason or a workspace `[lints]` table, and log each in the RAID log as tech debt
- Add `rustfmt.toml` / `clippy.toml` only if needed to match existing style

## Acceptance criteria

- [ ] AC-1: `cargo fmt --all --check` passes
- [ ] AC-2: `cargo clippy --workspace --exclude lanesra-os --all-targets -- -D warnings` passes
- [ ] AC-3: Full Rust test suite still passes with the same count as the baseline in REGRESSION.md

## Test plan

- Re-run the full suite; record counts before and after
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
