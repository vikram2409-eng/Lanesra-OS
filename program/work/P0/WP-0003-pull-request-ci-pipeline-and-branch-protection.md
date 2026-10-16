---
id: WP-0003
title: Pull request CI pipeline and branch protection
phase: 0
epic: E0.1 Program and engineering readiness
status: done
owner: claude
depends_on: [WP-0002, WP-0004, WP-0016]
requirements: [REQ-PRE-001]
estimate: M
editions: []
delivered: []
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/625
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

- [x] AC-1: CI runs on every PR and push to main and is green on main
- [x] AC-2: A deliberately failing test in a throwaway PR turns CI red (screenshot or run link in PR)
- [x] AC-3: Required-check names are listed in `program/PROTOCOL.md` and enabled by the owner
- [x] AC-5: TEST-STRATEGY, PROTOCOL §5 and FINISH-CHECKLIST state the tiered rule; the full suite is required in CI, not in the AI's session
- [x] AC-4: Median CI duration recorded in `program/testing/REGRESSION.md`

## Test plan

- The workflow itself is the test; prove both a green and a red run
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-10-16 claude: First CI run on PR #624 (head 39f72b6): program, migrations, web and demo green; `tauri (desktop shell)` red — `failed to read icon icons/32x32.png: Invalid PNG signature`. Root cause (not this PR's code, but on `main` and a release blocker): the 2026-10-14 brand rollout committed 11 images as base64 text instead of binary (RAID I-11). Decoded all of them back (`base64 -d`), verified each opens as PNG/ICO, and added `program/tools/check_binary_assets.py` (+5 self-tests) to the `demo` check so it cannot recur. State: waiting on the rerun. Next: AC-2 red-run proof (temporary failing test, then revert), AC-4 CI duration. Gotcha: the dev container has no webkit2gtk, so the Tauri crate can only be checked in CI.

- 2026-10-16 claude: PR #624 merged (5f27699). Proof for the acceptance criteria:
  - AC-1: `main` CI run 38918126664 green on the merge commit; PR runs green on 2196270 (38905651374).
  - AC-2: PR #625 commit b82f577 added a deliberately failing test; run 38918390735 failed shard 1/4 on exactly that test (1 failed, 278 passed) and the required `rust (tests)` summary went red. Reverted in b0e874c; the branch tree now equals `main`.
  - AC-4: median of three full runs is 12 min 21 s (table in `program/testing/REGRESSION.md`).
  - AC-5: tiered rule written on PR #624.
  - State: in_review on PR #625. Only AC-3 is left: the owner enables branch protection on `main` with the seven required checks in PROTOCOL §13 (RAID D-01); then mark done and close RAID I-04. Next: WP-0018 unblocks; WP-0008 is next on the critical path.
  - Gotcha: `git revert` has no `-q`; a failed revert followed by `commit --amend` rewrites the previous commit. Check `git log` before amending.

- 2026-10-16 claude: The owner confirmed that branch protection is on for `main`, with the required checks in PROTOCOL §13 (AC-3). PR #626 now shows as "blocked" until its checks pass, which matches. WP-0003 is done; RAID I-04 and D-01 are closed. WP-0018, which depends on it, was already `ready`.
