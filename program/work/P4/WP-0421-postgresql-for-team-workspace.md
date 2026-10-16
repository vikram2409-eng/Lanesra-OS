---
id: WP-0421
title: PostgreSQL for Team Workspace
phase: 4
epic: E4.1 Scale
status: proposed
owner: unassigned
depends_on: [WP-0420, WP-0180]
requirements: [REQ-PRE-023]
estimate: XL
editions: [workspace]
editions_reason: The database choice exists only for the Team Workspace server; desktop stays on SQLite and the online demo runs in the browser
delivered: []
docs: [set-up-team-workspace]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0421 — PostgreSQL for Team Workspace

## Goal

An organisation can choose PostgreSQL instead of the built-in SQLite file for its Team Workspace server, and can move an existing workspace across (ADR-0013 #4, ADR-0014 §6).

## Scope

- A PostgreSQL implementation of the WP-0420 storage layer: connection pool, TLS to the database, and migrations from `migrations/postgres/`.
- Configuration: `LANESRA_DATABASE_URL` for the server and its control database. Each workspace gets its own PostgreSQL database or schema (decide in the WP-0420 ADR). SQLite stays the default.
- Backups on PostgreSQL:
  - Lanesra's own export, so `.lanesra` files are portable between the two databases;
  - documentation for `pg_dump`;
  - key recovery is unchanged (WP-0180).
- A migration tool that copies a SQLite workspace into PostgreSQL, verifies row counts and checksums, then switches the workspace over.
- CI: the full regression suite also runs against a PostgreSQL service container.
- Help: the `set-up-team-workspace` article covers choosing and configuring a database.

## Acceptance criteria

- [ ] AC-1: The full regression suite passes on PostgreSQL in CI
- [ ] AC-2: A workspace moved from SQLite to PostgreSQL keeps every record, relationship, run, audit row and decryptable secret (test)
- [ ] AC-3: A `.lanesra` backup taken on one database restores on the other
- [ ] AC-4: The help article explains when to choose PostgreSQL and how to configure it

## Test plan

- A CI matrix job runs the suite on SQLite and on PostgreSQL; a migration-tool test uses fixture data
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy (a busy or unreachable database is `transient`); ADR-0010 migration conventions; ADR-0012 feature flags; ADR-0014. ADR-0010 applies to any migration this work package adds.
- MySQL or MariaDB would each be a separate work package, if ever wanted.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
