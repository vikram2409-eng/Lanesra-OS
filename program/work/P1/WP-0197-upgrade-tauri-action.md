---
id: WP-0197
title: "Upgrade tauri-action to 1.0 in the desktop release workflow"
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

# WP-0197 — Upgrade tauri-action to 1.0 in the desktop release workflow

## Goal

Move `desktop-release.yml` to tauri-apps/tauri-action 1.0 (closed Dependabot #666). It runs only when a desktop release is cut, so CI on a pull request cannot prove it.

## Scope

- The upgrade, its lockfile and any code it forces
- Nothing else: no refactors riding along

## Out of scope

- Other dependencies' majors

## Acceptance criteria

- [ ] AC-1: The workflow uses tauri-action 1.0 pinned by SHA with inputs migrated
- [ ] AC-2: A dry-run release from a test tag produces the Windows installer and draft release
- [ ] AC-3: The release runbook notes the change

## Test plan

- The full regression suite and the build jobs in CI
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-22 claude: Written after the owner approved the recommendations in RAID Q-25.
