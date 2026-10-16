---
id: WP-0150
title: Move every pause-for-human onto ai_approvals
phase: 1
epic: E1.6 Approvals
status: proposed
owner: unassigned
depends_on: [WP-0110, WP-0007]
requirements: [REQ-GOV-011]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [approvals-inbox]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0150 — Move every pause-for-human onto ai_approvals

## Goal

Retire the three older pause mechanisms in ai_orchestration_service; every human decision goes through the durable approvals inbox.

## Scope

- Map each legacy path (found in WP-0008)
- Migration of pending legacy pauses
- Resume semantics unchanged for users
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - Legacy paths found by the audit: (a) pipeline `requires_approval` in all three topologies (`ai_orchestration_service.rs:449,508,644` → `ai_agent_runs.awaiting_approval`), (b) voice plans (`voice_execution_service.rs:257-262`; no resolver for `awaiting_approval`)
  - Policy `RequireApproval` writes an `ai_approvals` row but drops the tool call: make it pause and resume
  - Fix the stuck graph run when its approval is resolved from the generic inbox command (I-13)

## Acceptance criteria

- [ ] AC-1: Each topology's pause/resume works via ai_approvals (existing tests updated, none deleted)
- [ ] AC-2: In-flight legacy pauses survive the upgrade (migration fixture test)

## Test plan

- Update `ai_orchestration_topologies.rs`, `ai_hitl_and_observability.rs`; add `approvals_unification.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0008 trace and events; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
