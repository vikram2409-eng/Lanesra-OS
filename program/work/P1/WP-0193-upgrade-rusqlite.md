---
id: WP-0193
title: "Upgrade rusqlite 0.31 to 0.40"
phase: 1
epic: E0.1 Program and engineering readiness
status: proposed
owner: unassigned
depends_on: []
requirements: [REQ-PRE-002, REQ-PRE-001]
threats: []
estimate: M
editions: [desktop, workspace]
editions_reason: Dependency upgrade with no user-visible change; the online demo has no build tooling or dependencies
delivered: []
docs: []
docs_reason: no user-visible change
branch: 
pr: 
updated: 2026-10-22
---

# WP-0193 — Upgrade rusqlite 0.31 to 0.40

## Goal

Move the core, server and desktop shell to rusqlite 0.40 (closed Dependabot #237) so the SQLite layer stays supported.

## Scope

- The upgrade, its lockfile and any code it forces
- Nothing else: no refactors riding along

## Out of scope

- Other dependencies' majors

## Acceptance criteria

- [ ] AC-1: `rusqlite` is 0.40 in every crate and `Cargo.lock`; the bundled SQLite version is recorded in the PR
- [ ] AC-2: Every migration upgrade fixture still upgrades (WP-0007 tests)
- [ ] AC-3: Full suite green; no behaviour change in backups (`backup` feature)

## Test plan

- The full regression suite and the build jobs in CI
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-22 claude: Written after the owner approved the recommendations in RAID Q-25.
