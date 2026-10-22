---
id: WP-0196
title: "Upgrade vite 8, vitest 5 and plugin-react 6"
phase: 1
epic: E0.1 Program and engineering readiness
status: proposed
owner: unassigned
depends_on: []
requirements: [REQ-PRE-002, REQ-PRE-001]
threats: []
estimate: S
editions: [desktop, workspace]
editions_reason: Dependency upgrade with no user-visible change; the online demo has no build tooling or dependencies
delivered: []
docs: []
docs_reason: no user-visible change
branch: 
pr: 
updated: 2026-10-22
---

# WP-0196 — Upgrade vite 8, vitest 5 and plugin-react 6

## Goal

Move the desktop build and test tooling to vite 8, vitest 5 and @vitejs/plugin-react 6 (closed Dependabot #669, #661, #668).

## Scope

- The upgrade, its lockfile and any code it forces
- Nothing else: no refactors riding along

## Out of scope

- Other dependencies' majors

## Acceptance criteria

- [ ] AC-1: The three packages are on their new majors with config migrated
- [ ] AC-2: `npm run build`, `npm test` and the Tauri build pass
- [ ] AC-3: Build output size is recorded before and after in the PR

## Test plan

- The full regression suite and the build jobs in CI
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-22 claude: Written after the owner approved the recommendations in RAID Q-25.
