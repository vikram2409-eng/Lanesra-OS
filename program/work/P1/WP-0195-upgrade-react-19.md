---
id: WP-0195
title: "Upgrade the desktop frontend to React 19"
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

# WP-0195 — Upgrade the desktop frontend to React 19

## Goal

Move the desktop frontend to React 19 and its types (closed Dependabot #665, #667).

## Scope

- The upgrade, its lockfile and any code it forces
- Nothing else: no refactors riding along

## Out of scope

- Other dependencies' majors

## Acceptance criteria

- [ ] AC-1: react, react-dom and their types are 19.x
- [ ] AC-2: `npx tsc --noEmit`, `npm test` and the Playwright suite pass
- [ ] AC-3: No deprecated API remains (string refs, legacy context)

## Test plan

- The full regression suite and the build jobs in CI
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-22 claude: Written after the owner approved the recommendations in RAID Q-25.
