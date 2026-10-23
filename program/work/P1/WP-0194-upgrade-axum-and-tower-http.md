---
id: WP-0194
title: "Upgrade axum 0.8 and tower-http 0.6"
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

# WP-0194 — Upgrade axum 0.8 and tower-http 0.6

## Goal

Move the Team Workspace server to axum 0.8 with axum-extra and tower-http 0.6 (closed Dependabot #239, #234).

## Scope

- The upgrade, its lockfile and any code it forces
- Nothing else: no refactors riding along

## Out of scope

- Other dependencies' majors

## Acceptance criteria

- [ ] AC-1: axum 0.8, axum-extra and tower-http 0.6 in `Cargo.lock`; route path syntax migrated (`/:id` to `/{id}`)
- [ ] AC-2: Every server route test passes unchanged, including SSE and static file serving
- [ ] AC-3: The server image builds

## Test plan

- The full regression suite and the build jobs in CI
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-22 claude: Written after the owner approved the recommendations in RAID Q-25.
