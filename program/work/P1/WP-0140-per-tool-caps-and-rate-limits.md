---
id: WP-0140
title: Per-tool caps and rate limits
phase: 1
epic: E1.5 Tools and firewall
status: in_review
owner: claude
depends_on: [WP-0122, WP-0111]
requirements: [REQ-LOOP-014, REQ-TOOL-008]
threats: [T-03, T-06]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [policy-engine-tool-call-firewall]
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/720
updated: 2026-10-24
---

# WP-0140 — Per-tool caps and rate limits

## Goal

Each tool binding caps calls per step, failures before disable, and result size kept in context; rate limits per tool, connector and workspace are enforced in the firewall before the call.

## Scope

- Caps on binding
- Large results stored and referenced
- Rate limiter in firewall

## Acceptance criteria

- [x] AC-1: Exceeding the calls-per-step cap blocks the call with a classified error (ADR-0009); exceeding the failures-before-disable threshold disables the tool binding; exceeding the result-size cap stores the result and passes a reference instead of keeping it in context; each enforcement is traced per ADR-0008
- [x] AC-2: Rate-limited call never reaches the tool

## Test plan

- `desktop/core/tests/tool_caps.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-19 muse: Refined proposed → ready. Made AC-1 precise: calls-per-step cap → classified error block (ADR-0009); failures-before-disable → binding disabled; result-size cap → store + reference; each traced per ADR-0008. AC-2 unchanged (rate-limited call never reaches tool). Verified: REQ-LOOP-014, REQ-TOOL-008 exist; ADR-0009/0010/0012 Accepted; depends_on WP-0122 merged. No code changed.
- 2026-10-23 claude: Not started. AC-1 asks for each enforcement to be traced per ADR-0008, and the span store that tracing writes to (`ai_run_spans`) is WP-0111, still proposed. Building the caps now would leave AC-1 open or invent a second trace format. Suggest adding WP-0111 to `depends_on` at the next refinement and building WP-0111 first.
- 2026-10-24 claude: Built, now that WP-0111's trace store exists.
  - Migration 0086: `ai_agent_tool_caps` (per agent and tool: calls per step, failures before disable, result size, plus live failure state), `ai_tool_rate_limits` (tool, connector or workspace scope), `ai_tool_call_window`, `ai_tool_results`.
  - `services::tool_limits_service`:
    - The agent loop checks a binding before each call. Over the per-step cap it refuses with `policy.tool_cap`. A binding disabled after N failures in a row refuses with `policy.tool_disabled` until an Administrator re-enables it; policy refusals do not count as failures. A result over the size cap is stored whole and the model gets its start plus `tool_result:<id>`. A delegated agent's run is its own step.
    - The Tool-Call Firewall checks rate limits after the policy allows a call and before it runs, for agents and the Assistant chat. Over a limit it refuses with `policy.rate_limited` and a retry time. Only allowed calls are counted, in a window kept for 24 hours.
    - The three codes are added to ADR-0009's list, as its §3 allows.
  - Traced (ADR-0008, AC-1): a refused call's tool span records `lanesra.tool.outcome: capped` and `lanesra.tool.cap`; a stored result records `result_size` and `lanesra.tool.result_ref`; a disabled-after-failure records `disabled_after_failures`; a rate refusal is a policy span with verdict `rate_limited` and the error code.
  - Admin APIs: Administrator only, audited; Tauri commands and server dispatch. The desktop gets a **Tool limits** editor on each agent and **Rate limits** in the Policy Engine panel. The online demo has the same panels and rules: its simulated agents are refused when disabled or rate limited, and it can simulate a capped step.
  - Help: two sections in `policy-engine-tool-call-firewall`.
  - Tests: `core/tests/tool_caps.rs` (8), checking side effects on real records.
- 2026-10-24 claude: In review in #720.
