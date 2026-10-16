---
id: WP-0004
title: Rust format and lint baseline
phase: 0
epic: E0.1 Program and engineering readiness
status: in_progress
owner: claude
depends_on: []
requirements: [REQ-PRE-001]
estimate: S
editions: []
delivered: []
branch: claude/enterprise-build-planning-jxy2kq
pr: 
updated: 2026-10-16
---

# WP-0004 — Rust format and lint baseline

## Goal

Make `cargo fmt --check` and `cargo clippy -D warnings` meaningful gates by bringing the existing code to a clean baseline (or recording a documented allow-list) before CI enforces them.

## Scope

- Run fmt and clippy across the workspace; fix mechanical findings in a formatting-only commit
- For findings not safely fixable now, add targeted `#[allow]` with a reason or a workspace `[lints]` table, and log each in the RAID log as tech debt
- Add `rustfmt.toml` / `clippy.toml` only if needed to match existing style

## Acceptance criteria

- [x] AC-1: `cargo fmt --all --check` passes
- [x] AC-2: `cargo clippy --workspace --exclude lanesra-os --all-targets -- -D warnings` passes
- [ ] AC-3: Full Rust test suite still passes with the same count as the baseline in REGRESSION.md

## Test plan

- Re-run the full suite; record counts before and after
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

### 2026-10-16 claude
Claimed (session-assigned branch `claude/enterprise-build-planning-jxy2kq`; this session may not push other branch names, so no `wp/` claim branch).
Done:
- AC-2: clippy baseline. ~84 unique warnings. Machine-applicable fixes applied and reviewed line by line (collapsible matches all fall through to a no-op `_ => {}`, so behaviour is unchanged); 102 doc-comment continuations indented from clippy's JSON suggestions; the auto-fix wrongly removed `entity_registry` from ontology_service (used only by its tests) - re-imported in the test module; a test's mutex guard scoped before the next await. `[workspace.lints.clippy]` allow-list with reasons: too_many_arguments (16), type_complexity (8), should_implement_trait (6), inconsistent_digit_grouping (cents literals). Logged as tech debt RAID T-01.
- Found and fixed: crates declared `rust-version = "1.77"` but need 1.82+ (`Option::is_none_or`); now 1.97, the pinned and tested toolchain.
- AC-1: `cargo fmt --all` with default settings, formatting-only commit 971ba7f (406 files); listed in `.git-blame-ignore-revs`. The old code matched no rustfmt width, so a one-time reformat while nothing else is in flight was the cheapest moment. rustfmt needed two passes to settle on one file.
State: clippy -D warnings clean on core, server, cli; fmt --check clean. Full test run in progress (AC-3).
Gotchas: the Tauri crate (lanesra-os) needs webkit2gtk to build, so its clippy runs in CI (WP-0003), which may surface a few more warnings there. Merge the PR with "Create a merge commit" so the blame-ignore SHA stays valid.
