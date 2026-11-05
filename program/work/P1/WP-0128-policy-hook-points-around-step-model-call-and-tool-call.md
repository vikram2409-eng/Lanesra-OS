---
id: WP-0128
title: Policy hook points around step, model call and tool call
phase: 1
epic: E1.3 Agent loop
status: done
owner: claude
depends_on: [WP-0122, WP-0133]
requirements: [REQ-LOOP-017]
threats: [T-03]
estimate: L
editions: [desktop, workspace, demo]
delivered: [AC-1, AC-2]
docs: [policy-engine-tool-call-firewall]
branch: claude/enterprise-build-planning-jxy2kq
pr: 739
updated: 2026-11-05
---

# WP-0128 — Policy hook points around step, model call and tool call

## Goal

Admins attach policies and guardrails at three points in the loop, configured as Policy objects, no code.

## Scope

- Hook execution order and short-circuit semantics
- Verdicts recorded on spans

## Acceptance criteria

- [x] AC-1: A policy attached at each of the three hook points (step boundary, model call, tool call) can block execution; the first blocking policy short-circuits the remaining hooks in order
- [x] AC-2: Every policy verdict (allow/block) is recorded on the trace span per ADR-0008 and emitted as an auditable policy-decision event

## Test plan

- `desktop/core/tests/policy_hooks.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0008 trace and events; ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-19 muse: Refined proposed → ready. Split AC-1 into AC-1 (block + short-circuit at the three hook points: step boundary, model call, tool call) and AC-2 (verdicts on trace spans per ADR-0008 + auditable policy-decision events). Verified: REQ-LOOP-017 exists; ADR-0008/0009/0010/0012 Accepted; depends_on WP-0122 merged, WP-0133 still proposed (listed per Definition of Ready). No code changed.
- 2026-11-05 claude: Implemented both ACs. A guardrail (WP-0133 policy object) now attaches at one of three loop hook points via a new `hook_point` column on `ai_guardrails` (migration 0091; 'off' default so existing guardrails are unchanged). `policy_engine_service::{HookPoint, HookVerdict, evaluate_loop_hook, evaluate_loop_round}` run the attached guardrails at each point; the first failing guardrail blocks, and `evaluate_loop_round` evaluates step→model→tool in order, short-circuiting (AC-1). Each verdict (allow and block) is recorded on a `policy` trace span carrying `lanesra.policy.hook` and emitted as a new `policy.decided` run event with its JSON schema (AC-2). Wired for real: step boundary in `graph_runtime_service` and `ai_orchestration_service` (pipeline steps); model call and tool call in `chat_service`'s agent round. Editions: desktop + Team Workspace share the GuardrailsAdmin "Runs at" selector (new column + create field); demo guardrails tab shows where each runs and explains the three hook points. Docs: policy-engine-tool-call-firewall help article gains a "Policy hook points in the agent loop" section. Tests: `core/tests/policy_hooks.rs` (9, engine + real pipeline runs) + a new guardrailsAdmin UI test; patched run_events.rs expected set. Next: open PR, drive CI green.
