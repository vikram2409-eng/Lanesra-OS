---
id: WP-0002
title: Repair dependency lockfiles and pin toolchains
phase: 0
epic: E0.1 Program and engineering readiness
status: in_review
owner: claude
depends_on: []
requirements: [REQ-PRE-002]
estimate: S
editions: [desktop, workspace]
delivered: []
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/492
updated: 2026-10-16
---

# WP-0002 — Repair dependency lockfiles and pin toolchains

## Goal

Make a clean install reproducible. Today `npm ci` fails because `desktop/package-lock.json` is out of sync with `package.json` (8 direct packages missing), which also breaks the release workflow; `desktop/Cargo.lock` still records lanesra-os 0.14.0; jsdom 30 needs Node >= 22.22.2.

## Scope

- Regenerate `desktop/package-lock.json` with `npm install` and commit it
- Commit the refreshed `desktop/Cargo.lock`
- Add `desktop/.nvmrc` (or `.node-version`) and `engines.node` in package.json matching jsdom's floor
- Add `desktop/rust-toolchain.toml` pinning the stable toolchain used in CI
- Point `desktop-release.yml` at the pinned Node version

## Acceptance criteria

- [x] AC-1: `npm ci` succeeds from a clean checkout in `desktop/`
- [x] AC-2: `cargo build --locked -p lanesra-core -p lanesra-server -p lanesra-cli` succeeds
- [x] AC-3: `npm run build` and `npm test` pass after `npm ci`
- [x] AC-4: Release workflow's install step would pass (verified by running the same commands on Linux and noting Windows-specific risk)

## Test plan

- Run the commands above in a fresh clone and paste the output into the PR
- No new tests; CI in WP-0003 enforces `--locked` and `npm ci` from then on
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

### 2026-10-16 claude
Owner-requested in the planning session; shares the planning branch (see WP-0016 note).
Done:
- `desktop/package-lock.json` regenerated with `npm install` (8 direct packages and their trees
  were missing). `rm -rf node_modules && npm ci` now succeeds.
- `desktop/package.json`: `engines.node` = `^22.22.2 || ^24.15.0 || >=26.0.0` (jsdom 30's floor).
- `desktop/.nvmrc` = `22`; `desktop-release.yml` uses `node-version-file: desktop/.nvmrc` instead of Node 20.
- `desktop/rust-toolchain.toml` pins Rust 1.97.0 with rustfmt and clippy.
- `desktop/Cargo.lock` refreshed (lanesra-os 0.14.0 -> 0.15.0); `cargo metadata --locked` passes.
Verified on Linux: `npm ci`, `npx tsc --noEmit` (clean), `npm test` (40/40), `npm run build` (ok),
`cargo test --locked` (full run, see WP-0016). Windows release build not run (only runs on tags);
risk: none expected beyond the Node version change.
Gotchas: the pinned toolchain makes rustup download 1.97.0 once per machine (~600 MB). Debug test
builds need ~30 GB of disk - `cargo clean` if "No space left on device".

### 2026-10-16 claude
PR opened: https://github.com/vikram2409-eng/Lanesra-OS/pull/492 (shared planning PR, owner-requested). Status in_review. After merge: status done, `delivered:` filled for its editions.
