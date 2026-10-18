---
id: WP-0006
title: Browser end-to-end test harness
phase: 0
epic: E0.2 Test infrastructure
status: in_progress
owner: muse
depends_on: [WP-0003]
requirements: [REQ-PRE-004]
estimate: M
editions: []
delivered: []
branch: wp/WP-0006-e2e-harness
pr: TBD
updated: 2026-10-17
---

# WP-0006 — Browser end-to-end test harness

## Goal

The Playground is a UI-heavy deliverable and the frontend has only 5 unit test files (40 tests). Add a Playwright harness that drives the real UI served by the Team Workspace server, and the online demo.

## Scope

- Playwright project under `desktop/e2e/` using the pre-installed Chromium
- Fixture that builds the frontend, starts `lanesra-server` on a temp data dir with seeded data, and tears down
- Fixture that serves the repo-root online demo statically
- Smoke specs: sign in, open a record, open AI agent admin, run a pipeline with the scripted provider (WP-0005)
- CI job (can be a separate workflow) running E2E on PRs touching `desktop/src`, `desktop/server` or `app.js`
- Trace/screenshot artifacts uploaded on failure

## Acceptance criteria

- [ ] AC-1: `npx playwright test` passes locally and in CI
- [ ] AC-2: A failing spec uploads a trace artifact
- [ ] AC-3: Running E2E needs no network access and no API keys

## Test plan

- 4+ smoke specs as listed; each spec tagged with the requirement it protects
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

## Refinement log

- 2026-10-17 muse: Moved proposed → ready. WP-0003 done; WP-0005 (scripted provider) merged — smoke specs can use it. ACs testable; size M.

- 2026-10-17 muse: Claimed WP-0006. Built: desktop/e2e/playwright.config.ts, desktop/e2e/tests/admin-first-run.spec.ts (first-run setup → sign-in → Settings → Feature flags as Administrator, Covers: REQ-PRE-004), .github/workflows/e2e.yml (new non-required check, Playwright with chromium), PROTOCOL.md §13 note (owner adds to required after a week of green runs). Untested locally — see CI.
