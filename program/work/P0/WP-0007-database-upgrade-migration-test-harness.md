---
id: WP-0007
title: Database upgrade (migration) test harness
phase: 0
epic: E0.2 Test infrastructure
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0003]
requirements: [REQ-PRE-005]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0007 — Database upgrade (migration) test harness

## Goal

This build adds many tables and columns to a schema that customers already hold data in (78 migrations). Prove every new migration upgrades a real older database without loss.

## Scope

- Fixture databases created by the last released desktop version (and the current main) with representative data, stored under `desktop/core/tests/fixtures/db/`
- A test that opens each fixture, runs migrations, and checks row counts and key invariants are preserved
- A test that migration numbers are contiguous, unique and that `MIGRATIONS` lists every file
- Script to regenerate fixtures, documented

## Acceptance criteria

- [ ] AC-1: Upgrade test passes for the current HEAD
- [ ] AC-2: Adding a migration that drops a populated column makes the test fail (shown in PR)
- [ ] AC-3: Fixture regeneration documented in TEST-STRATEGY.md

## Test plan

- `desktop/core/tests/migration_upgrade.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
