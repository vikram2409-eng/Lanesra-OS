---
id: WP-0014
title: Design token inventory and accessibility tooling
phase: 0
epic: E0.2 Test infrastructure
status: in_review
owner: muse
depends_on: []
requirements: [REQ-PRE-011]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: []
docs_reason: internal design tokens and accessibility tooling; no user-facing behaviour
branch: wp/WP-0014-design-tokens-a11y
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/644
updated: 2026-10-17
---

# WP-0014 — Design token inventory and accessibility tooling

## Goal

The mockups use the online demo's palette and new components (risk badges, status pills, trace items, setup rail). Map them to the desktop UI kit before screens are built, and add automated a11y checks.

## Scope

- Inventory: each colour/token in the spec's palette mapped to a desktop CSS variable; gaps added
- Component list needed by Phases 1-3 (RiskBadge, LifecyclePill, StatusDot, EffectNote, Toast, BudgetMeter, TraceItem, ProgressRail) with which exist in `desktop/src/components/ui`
- Add `vitest-axe` (or axe-core) for component tests and `@axe-core/playwright` for E2E
- Light and dark theme check for each new token (WCAG AA)

## Acceptance criteria

- [x] AC-1: `program/design/TOKENS.md` committed
- [x] AC-2: Axe runs in unit tests and fails on a planted violation
- [x] AC-3: Missing primitive components listed as tasks inside WP-0160 / WP-0203

## Test plan

- One axe test per existing UI primitive
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Dependency on CI (WP-0003) removed 2026-10-16: the inventory and the unit-level axe check do not need CI; the E2E axe hook waits for WP-0006.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-17 claude: Closeout (review by Claude Code). Merged in PR #644. AC-1: program/design/TOKENS.md. AC-2: desktop/src/__tests__/a11y.test.tsx runs axe-core on rendered components and its "flags a planted violation" test fails on an image without alt text (vitest, CI web job). AC-3: "Component tasks (from WP-0014 inventory)" sections added to WP-0160 and WP-0203.
- 2026-10-17 claude: Not done yet: `editions: [desktop, workspace, demo]` but `delivered:` is empty. The axe checks cover the desktop/Team Workspace React UI (same frontend); the online demo (app.js) has no accessibility check. Either add an axe check for the demo (e.g. in the WP-0006 Playwright job) and record `delivered`, or narrow `editions` with an `editions_reason:`. Status stays in_review.
