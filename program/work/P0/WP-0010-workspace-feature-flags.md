---
id: WP-0010
title: Workspace feature flags
phase: 0
epic: E0.1 Program and engineering readiness
status: in_progress
owner: claude
depends_on: [WP-0009]
requirements: [REQ-PRE-006, REQ-PRE-021]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: []
docs_reason: developer-only setting; documented in desktop/README.md
branch: claude/enterprise-build-planning-jxy2kq
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

- [x] AC-1: A flag off hides the UI entry point and makes its API return 404/feature-disabled
- [x] AC-2: Flags default off in release builds
- [x] AC-3: Tests cover both states for one sample flag

## Test plan

- `desktop/core/tests/feature_flags.rs`, one vitest for the hook, one E2E toggle spec
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-10-16 claude: Built per ADR-0012:
  - **Core:**
    - `models/feature_flag.rs`.
    - `services/feature_flag_service.rs`: `REGISTRY` with the first real flag `playground.shell` (WP-0203, building), `is_enabled`, `require_enabled`, `list`, `set_override`, and `developer_mode()`, which is a compile-time `false` in release builds.
    - `repositories/feature_flag_repo.rs`.
    - Migration 0080 `feature_flag_overrides`, with an ADR-0010 header.
  - **Desktop:** Tauri commands `list_feature_flags` and `set_feature_flag_override` (Edition::Desktop).
  - **Team Workspace:** the same two commands in `server/src/dispatch.rs` (Edition::Workspace).
  - **Frontend:** API calls; the `useFeature`, `useFeatureFlags` and `useShowFeatureFlagsAdmin` hooks; Admin → Workspace → Feature flags (listed only when something is toggleable).
  - **Online demo:** `DEMO_RELEASED_FEATURES` plus `demoFeature()` in `app.js`; `check_feature_flags.py` (7 self-tests) runs in the `demo` CI check, so the demo can list only `ga` flags (REQ-PRE-021).
  - **Tests:**
    - core: 6 integration tests (`core/tests/feature_flags.rs`) and 2 unit tests;
    - server: 3 HTTP tests (`server/tests/feature_flags_http.rs`);
    - frontend: 6 vitest tests (`src/__tests__/featureFlags.test.tsx`);
    - tools: 7 self-tests.
  - **Acceptance criteria:**
    - AC-1: an off flag hides the UI entry point (vitest) and the guard returns `not_found` / `feature_disabled: <key>` (core).
    - AC-2: building flags default off and cannot be enabled outside developer mode, which a release build never has (core, server and registry unit tests).
    - AC-3: both states tested for `playground.shell`.
  - **Local runs:** clippy (core and server, `-D warnings`) and fmt are clean; tsc, vitest (46/46) and the build pass. The Tauri crate cannot be compiled in this container (no webkit2gtk), so CI checks it.
  - **Next:** WP-0203 wraps the Playground behind `playground.shell`; WP-0007 (migration runner transaction and header check) and WP-0110 (run schema) can start.
