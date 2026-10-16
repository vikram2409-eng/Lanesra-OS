---
id: WP-0002
title: Repair dependency lockfiles and pin toolchains
phase: 0
epic: E0.1 Program and engineering readiness
status: ready
owner: unassigned
depends_on: []
requirements: [REQ-PRE-002]
estimate: S
branch: 
pr: 
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

- [ ] AC-1: `npm ci` succeeds from a clean checkout in `desktop/`
- [ ] AC-2: `cargo build --locked -p lanesra-core -p lanesra-server -p lanesra-cli` succeeds
- [ ] AC-3: `npm run build` and `npm test` pass after `npm ci`
- [ ] AC-4: Release workflow's install step would pass (verified by running the same commands on Linux and noting Windows-specific risk)

## Test plan

- Run the commands above in a fresh clone and paste the output into the PR
- No new tests; CI in WP-0003 enforces `--locked` and `npm ci` from then on
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
