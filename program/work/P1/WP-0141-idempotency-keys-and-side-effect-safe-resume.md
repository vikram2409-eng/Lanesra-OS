---
id: WP-0141
title: Idempotency keys and side-effect-safe resume
phase: 1
epic: E1.5 Tools and firewall
status: proposed
owner: unassigned
depends_on: [WP-0110]
requirements: [REQ-TOOL-005, REQ-GOV-023]
threats: [T-03, T-06]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: internal engine work with no user-visible surface of its own; the online demo's simulated runs reflect it through the Playground work packages
docs: []
docs_reason: internal write ledger
branch: 
pr: 
updated: 2026-10-16
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

- [ ] AC-1: Kill mid-step then resume → no duplicate record (test with process restart simulation)
- [ ] AC-2: Retry after a transient failure never double-writes

## Test plan

- `desktop/core/tests/idempotent_resume.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
