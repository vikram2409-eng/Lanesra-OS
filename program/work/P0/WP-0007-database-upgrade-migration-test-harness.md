---
id: WP-0007
title: Database upgrade (migration) test harness
phase: 0
epic: E0.2 Test infrastructure
status: done
owner: muse
depends_on: [WP-0003]
requirements: [REQ-PRE-005]
estimate: M
editions: []
delivered: []
branch: wp/WP-0007-fixtures-and-closeout
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/649
updated: 2026-10-18
---

# WP-0007 — Database upgrade (migration) test harness

## Goal

This build adds many tables and columns to a schema that customers already hold data in (78 migrations). Prove every new migration upgrades a real older database without loss.

## Scope

- Fixture databases created by the last released desktop version (and the current main) with representative data, stored under `desktop/core/tests/fixtures/db/`
- A test that opens each fixture, runs migrations, and checks row counts and key invariants are preserved
- A test that migration numbers are contiguous, unique and that `MIGRATIONS` lists every file
- Script to regenerate fixtures, documented
- From ADR-0010 (WP-0009): run each migration and its `schema_migrations` row in one transaction (`run_migrations` has none today, so a failed migration leaves a half-applied schema); add the migration header check to `check_migrations.py` for migrations above 0079

## Acceptance criteria

- [x] AC-1: Upgrade test passes for the current HEAD
- [x] AC-2: A canary migration that drops a populated column is added in the PR, the upgrade test fails on it, then the canary is removed before merge (failure shown in PR checks)
- [x] AC-3: Fixture regeneration documented in TEST-STRATEGY.md

## Test plan

- `desktop/core/tests/migration_upgrade.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions. ADR-0010 applies to any migration this work package adds.

## Progress log

- 2026-10-17 muse: AC-2 proven. Canary migration 0081_canary_drop_companies_name.sql (ALTER TABLE companies DROP COLUMN name) added on branch wp/WP-0007-canary-ac2 (PR #650, closed unmerged). CI rust (tests) FAILED as expected with exit 100 ("no such column: companies.name"), proving the harness catches destructive migrations. Red CI run: https://github.com/vikram2409-eng/Lanesra-OS/pull/650/checks

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

## Refinement log

- 2026-10-17 muse: Moved proposed → ready. AC-2 made precise (canary migration pattern). ADR-0010 accepted; WP-0003 done; test file named; size M.

## Progress log

- 2026-10-17 muse: Claimed WP-0007. Building: desktop/core/tests/migration_upgrade.rs (AC-1/AC-2), migrate.rs transaction wrapping per ADR-0010 + migration_versions() test helper, check_migrations.py ADR-0010 header check for migrations >0079, TEST-STRATEGY.md fixture docs (AC-3), desktop/core/tests/fixtures/gen_fixtures.py. Untested locally — see CI.
- 2026-10-17 claude: Review by Claude Code after merge of PR #647. Delivered: run_migrations wraps each migration and its schema_migrations row in one transaction (ADR-0010), migration_versions(), the numbering test, gen_fixtures.py and its TEST-STRATEGY.md section (AC-3 ticked). Not yet delivered, so status stays in_progress: (1) no fixture databases are committed under desktop/core/tests/fixtures/db/ (scope item 1), so `upgrade_preserves_data` migrates a fresh database and re-runs migrations; it never upgrades an older, populated schema. AC-1 needs a test that opens each committed fixture (last released desktop version, current main), runs migrations and checks row counts and invariants. (2) AC-2 has no recorded run: add the canary migration on the PR, link the red check run here, then remove it. Next: generate and commit the fixtures with gen_fixtures.py, rewrite the upgrade test to open them, record the AC-2 canary run.

- 2026-10-17 muse: Closeout (fixtures and AC-2 canary). Generated fixture databases with gen_fixtures.py: 0078_v0_15_0.db (last released desktop v0.15.0, 78 migrations) and 0080_main.db (current main, 80 migrations), committed under desktop/core/tests/fixtures/db/. Rewrote migration_upgrade.rs to open each fixture, run migrations, and verify row counts + canary column data. Untested locally — see CI.
- 2026-10-17 claude: The upgrade test opened the committed fixtures and migrated them in place, so after one local run `0078_v0_15_0.db` was already at the latest schema and later runs upgraded nothing (and the two tests shared one file). It now migrates a private temp copy of each fixture (`open_copy` in migration_upgrade.rs). AC-2 still needs the red canary CI run linked here.
- 2026-10-18 claude: Done: PRs #647 and #649 merged, AC-2 canary proven in #650 (red CI, closed unmerged), ticked in #659; the upgrade test migrates temp copies of the fixtures (#648).
