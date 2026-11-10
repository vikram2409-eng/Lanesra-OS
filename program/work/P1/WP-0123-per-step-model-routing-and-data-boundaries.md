---
id: WP-0123
title: Per-step model routing and data boundaries
phase: 1
epic: E1.3 Agent loop
status: done
owner: claude
depends_on: [WP-0121]
requirements: [REQ-LOOP-003, REQ-GOV-008]
threats: [T-04]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [unified-ai-gateway]
branch: claude/enterprise-build-planning-jxy2kq
pr: 
updated: 2026-11-10
---

# WP-0123 — Per-step model routing and data boundaries

## Goal

Route each step to a model by tier or exact model, through the gateway's fallback chain, and enforce which data classes may reach which provider (sensitive data forced local).

## Scope

- Model tier mapping (Fast, Balanced, Best, Local only)
- Data-class → provider allow-list policy
- Routing decision recorded on the span
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - Forced air-gap must fail closed when no local provider is configured, and check that the chosen provider is local (I-17)
  - Route the records and admin chat loop (`chat_service.rs:978-1140`) through the AI Gateway; it bypasses budgets, fallback and DLP today (I-15)
- Deferred (Gate-0 refine): the named model-tier labels (Fast/Balanced/Best/Local only) and a general data-class→provider allow-list matrix are left for a later phase — REQ-GOV-008 is marked `phase: plan`, and the per-tier primary/fallback/local routing from WP-0121/0122 already carries routing. This WP delivers the security-critical sensitive-data→local boundary and its fail-closed guarantee.

## Acceptance criteria

- [x] AC-1: A step bound to sensitive data is never sent to a non-local provider (tested)
- [x] AC-2: Fallback chain order is honoured and recorded

## Test plan

- `desktop/core/tests/model_routing.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0008 trace and events; ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-10 claude: Gate-0 refine + build in one pass. Confirmed REQ-LOOP-003 and REQ-GOV-008 exist and dep WP-0121 is done. Refined scope against the gap audit (I-15, I-17) and the ADRs: the security-critical core is the forced-air-gap data boundary (AC-1) plus the already-implemented fallback order and its recording (AC-2). The Fast/Balanced/Best named-tier UI and the general data-class→provider allow-list matrix are deferred (REQ-GOV-008 is marked `phase: plan`); the per-tier primary/fallback/local policy from WP-0121/0122 already carries routing, so no new routing model was added. Build: migration 0094 adds `is_local` to `ai_providers` (set by its own admin action `set_local`, the `set_active` shape — `AiProviderInput` unchanged, no test-literal churn). I-17: the Gateway now fails closed under forced air-gap — it dispatches only to an explicitly-set `local_fallback` provider marked `is_local`, and returns `config.not_local` when none is set, the set one can't be resolved, or it isn't local; it never falls back to the workspace default for sensitive data. The routing decision is already on the model_call span (`lanesra.served_by` + provider/model, WP-0111). I-15: the Assistant chat (`chat_service::send_message`) now goes through a new `ai_gateway_service::dispatch_workspace_default` so the System-tier daily budget applies and usage is recorded on the one shared path (no agent, so no per-agent failover/air-gap). Editions: desktop + Team Workspace get a Local column and Mark local action on the Providers table; the online demo documents it (v0.99.0 changelog + the Unified AI Gateway help gains "Which providers count as local — and failing closed" and "The Assistant goes through the Gateway too", wps WP-0123). Tests: `desktop/core/tests/model_routing.rs` (4) — air-gap routes to a marked-local provider, fails closed with no local provider, fails closed when the fallback is not local, and the fallback tier answers + is recorded; the existing `ai_gateway.rs` air-gap happy-path test now marks its local provider local. Local gates green: model_routing, ai_gateway, chat, tool_calling_formats, agent_loop_limits, ai_agent_policy_engine, tool_caps, trace_spans (golden unchanged), trace_conformance, migrate (schema 94), migration_upgrade; fmt + clippy (core/server/cli + tauri) clean; tsc + 66 vitest; app.js node --check. Next: open PR, drive CI green.
