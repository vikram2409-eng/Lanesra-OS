---
id: WP-0003
title: Pull request CI pipeline and branch protection
phase: 0
epic: E0.1 Program and engineering readiness
status: ready
owner: unassigned
depends_on: [WP-0002, WP-0004, WP-0016]
requirements: [REQ-PRE-001]
estimate: M
editions: []
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0003 — Pull request CI pipeline and branch protection

## Goal

No automated checks run on pull requests today (only a Windows release build on tags). Muse AI cannot run tests itself (it works through GitHub), so CI is the only place its changes are tested. With several AI agents committing, every PR must prove it does not regress anything before it can merge.

## Scope

- `.github/workflows/ci.yml` on pull_request and push to main, jobs in parallel:
  - rust: `cargo fmt --check`, `cargo clippy -- -D warnings` (baseline from WP-0004), `cargo test --locked -p lanesra-core -p lanesra-server -p lanesra-cli`
  - tauri: `cargo check --locked -p lanesra-os` on ubuntu with webkit2gtk deps
  - web: `npm ci`, `npx tsc --noEmit`, `npm test`, `npm run build`
  - demo: `node --check app.js site-modern-pages.js`
  - program: programctl validate, status --check, self-tests
  - migrations: new migration numbers are contiguous and above main's maximum (see ADR on migrations)
- Rust and npm caching; total wall-clock target under 15 minutes
- Turn on branch protection for `main` (owner decision, ADR-0004: Claude Code does it) - required checks, PR required, no force-push, no deletion. Claude Code applies it through the GitHub API or, if its tools cannot, gives the owner the exact settings to click. Record the final settings in PROTOCOL.md

- Tiered testing rule (owner decision, 2026-10-16), written into `program/testing/TEST-STRATEGY.md`, PROTOCOL §5 and the finish checklist:
  - while building, the AI runs only the tests for the crates and files it touched (minutes)
  - every pull request runs the full regression suite automatically in CI, sharded and cached; it must be green to merge
  - a full local run is only for changes that touch everything (formatting, toolchain, migrations framework) or when CI is down
  - nightly scheduled run for slow extras (full E2E, performance)
- `programctl brief` and BRIEFING tell the AI which targeted test command fits its files

## Acceptance criteria

- [ ] AC-1: CI runs on every PR and push to main and is green on main
- [ ] AC-2: A deliberately failing test in a throwaway PR turns CI red (screenshot or run link in PR)
- [ ] AC-3: Required-check names are listed in `program/PROTOCOL.md` and enabled by the owner
- [ ] AC-5: TEST-STRATEGY, PROTOCOL §5 and FINISH-CHECKLIST state the tiered rule; the full suite is required in CI, not in the AI's session
- [ ] AC-4: Median CI duration recorded in `program/testing/REGRESSION.md`

## Test plan

- The workflow itself is the test; prove both a green and a red run
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
