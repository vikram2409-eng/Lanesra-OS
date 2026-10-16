---
id: WP-0010
title: Workspace feature flags
phase: 0
epic: E0.1 Program and engineering readiness
status: proposed
owner: unassigned
depends_on: [WP-0009]
requirements: [REQ-PRE-006]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0010 — Workspace feature flags

## Goal

The build spans ~7 months. Unfinished capabilities must merge to `main` behind flags so `main` stays releasable and the online demo and desktop releases do not expose half-built features.

## Scope

- Flag registry in core (`feature_flags` table or settings JSON, per ADR) with defaults per edition
- Server and Tauri commands to read flags; frontend hook `useFeature(flag)`
- Admin-only toggle screen hidden unless a developer setting is on
- Convention: every new user-visible capability names its flag in its work package

## Acceptance criteria

- [ ] AC-1: A flag off hides the UI entry point and makes its API return 404/feature-disabled
- [ ] AC-2: Flags default off in release builds
- [ ] AC-3: Tests cover both states for one sample flag

## Test plan

- `desktop/core/tests/feature_flags.rs`, one vitest for the hook, one E2E toggle spec
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
