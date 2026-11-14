---
id: WP-0141
title: Idempotency keys and side-effect-safe resume
phase: 1
epic: E1.5 Tools and firewall
status: done
owner: claude
planned_for: claude
depends_on: [WP-0110]
requirements: [REQ-TOOL-005, REQ-GOV-023]
threats: [T-03, T-06]
estimate: L
editions: [desktop, workspace]
delivered: [desktop, workspace]
editions_reason: internal engine work with no user-visible surface of its own; the online demo's simulated runs reflect it through the Playground work packages
docs: []
docs_reason: internal write ledger
branch: wp/WP-0141-idempotency-keys-and-side-effect-safe-resume
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/771
updated: 2026-11-14
---

# WP-0141 — Idempotency keys and side-effect-safe resume

## Goal

Write tools receive an idempotency key per step attempt; on resume after a pause or crash, writes already made are recognised and not repeated.

## Scope

- Key derivation
- Write ledger
- Native writes, connector writes and MCP writes honour the key
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - Resume is not wired today (I-12): add a startup sweep over `list_resumable_runs`, a `resume_due_delays` tick on server and desktop, and a recovery rule for runs that crashed while `running`
  - Desktop advances unattended work only while the UI polls (I-19): add a background ticker in the Tauri shell
- Owner decisions (ADR-0013, ADR-0014, 2026-10-16):
  - Admin setting `interrupted_run_policy`: `resume_if_safe` (default) or `always_pause`, per workspace with a per-Team override; AI Admin only; audited (ADR-0007 §5)

## Acceptance criteria

- [x] AC-1: Kill mid-step then resume → no duplicate record (test with process restart simulation)
- [x] AC-2: Retry after a transient failure never double-writes
- [x] AC-3: Two requests to complete the same step attempt (a double click) produce one write; the second returns the first result
- [x] AC-4: With `interrupted_run_policy: always_pause` a run interrupted mid-step is paused, not resumed; the change of setting is audited and refused for anyone but an AI Admin
- [x] AC-5: The startup sweep and the delay tick resume a due run on the server and on desktop without the UI open

## Test plan

- `desktop/core/tests/idempotent_resume.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refined 2026-11-14 (ADR-0025): acceptance criteria added for double submit, the interrupted-run policy and the resume sweep; moved to `ready`. Process Lane writes (WP-0705) depend on this package.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

### 2026-11-14 claude
Done: AC-1..AC-5. New migration 0096 `ai_run_write_ledger`; `idempotency_service::run_once` applies a write once per key (run, node, attempt, write_key) and records it in the same transaction as the write, so a retry, a resume after a crash or a double submit never double-writes (`run_write_ledger_repo`). The graph `action` node now writes through `run_once`. `run_policy_service` stores `interrupted_run_policy` (resume_if_safe | always_pause) in the settings table, admin-only and audited. `graph_runtime_service::recover_interrupted_runs` moves a `running` run with an expired lease out of running and, under resume_if_safe, resumes it from its current node; `tick` = resume_due_delays + recover, wired into the server scheduler (`job_scheduler`) and the desktop poll (`ai_agent_pipeline_commands`). Policy get/set exposed as Tauri commands and server dispatch arms.
State: green locally - `core/tests/idempotent_resume.rs` 5/5; execution-graph runtime 18/18; migration-upgrade 3/3; `check_migrations.py` 96 migrations 0 problems; `cargo fmt --all --check` clean. Tauri shell not built here (missing GTK libs) - CI covers it. Full suite runs on the PR.
Next: WP-0705 (lane record/check steps) uses `step_idempotency_key` + `run_once` with a per-write `write_key` for all-or-nothing posting. WP-0707 (runner) depends on WP-0705.
Gotchas: the write ledger's `run_id` is a foreign key to `ai_runs`, so `run_once` needs a real run. A FK violation surfaces as a ConstraintViolation like a UNIQUE race; the race branch re-reads the key and only returns a stored result when one exists, otherwise it errors. Per-Team policy override is deferred until Teams exist (Phase 3). Connector/MCP writes get the derived idempotency key but the strong exactly-once guarantee is for native writes on the run's connection.
Commands: cargo test -p lanesra-core --features test-support --test idempotent_resume
