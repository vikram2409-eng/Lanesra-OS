---
id: WP-0420
title: Storage layer that can run on SQLite or PostgreSQL
phase: 4
epic: E4.1 Scale
status: proposed
owner: unassigned
depends_on: [WP-0181]
requirements: [REQ-PRE-023]
estimate: XL
editions: [workspace, desktop]
editions_reason: The desktop edition stays on SQLite but runs on the same storage layer; the online demo keeps its browser storage
delivered: []
docs_reason: Internal refactor with no user-visible change; documentation comes with WP-0421
branch: 
pr: 
updated: 2026-10-16
---

# WP-0420 — Storage layer that can run on SQLite or PostgreSQL

## Goal

The owner wants organisations to be able to run Team Workspace on PostgreSQL (ADR-0013 #4). Every service and repository currently takes a synchronous `rusqlite::Connection`:
- 85 repository files;
- 12 of 111 services also run SQL themselves;
- 79 migrations (about 4,800 lines) in SQLite's dialect;
- about 45 SQLite-only statements.

This package puts a database handle at the repository boundary that SQLite or PostgreSQL can back. There is no user-visible change.

## Scope

- A `Db` handle and transaction type used by all repositories. It hides the driver, the parameter style and the type mapping. SQLite keeps working exactly as before.
- Move the SQL out of the 12 services into repositories (ADR-0014 §5.1).
- Rewrite the remaining SQLite-only statements in portable SQL, or put them behind a dialect hook:
  - `datetime('now')` and `strftime`;
  - `GROUP_CONCAT`;
  - `INSERT OR IGNORE` and `INSERT OR REPLACE`;
  - `rowid`;
  - `COLLATE NOCASE`.
- Keep migrations per dialect (`migrations/sqlite/`, `migrations/postgres/`), plus a schema-equivalence test that compares the two catalogues.
- Decide between async and sync access in an ADR first: the server is async, while the core is sync today.

## Acceptance criteria

- [ ] AC-1: Every repository goes through the storage layer. No service runs SQL itself (a CI check enforces this)
- [ ] AC-2: The full regression suite passes on SQLite, unchanged
- [ ] AC-3: An ADR records the driver, the async/sync choice and the dialect strategy

## Test plan

- Existing suite unchanged and green; new schema-equivalence test
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0014 tenancy and database portability. ADR-0010 applies to any migration this work package adds.
- Scheduled after general availability of the Playground (ADR-0013). It is large and mechanical, so split it by repository family at refinement.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
