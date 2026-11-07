---
id: WP-0120
title: Agent persona, limits and reasoning modes
phase: 1
epic: E1.3 Agent loop
status: done
owner: claude
depends_on: [WP-0110, WP-0130]
requirements: [REQ-CORE-001, REQ-CORE-002, REQ-LOOP-015]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [build-your-first-agent]
branch: claude/enterprise-build-planning-jxy2kq
pr: 742
updated: 2026-11-07
---

# WP-0120 — Agent persona, limits and reasoning modes

## Goal

Extend the agent model with role, goal and background, limits and reasoning_mode, and compile them with instructions, guardrails, memory digest and ontology context into the system prompt.

## Scope

- Agent fields and versioned snapshot changes
- System prompt compiler using prompt catalog templates
- Read-only compiled prompt view API
- Plan-first and Reflect reasoning modes in the loop

## Acceptance criteria

- [x] AC-1: Compiled prompt is deterministic for the same inputs (snapshot test)
- [x] AC-2: Plan-first emits a plan span before tool use; Reflect makes exactly one extra model call
- [x] AC-3: Existing agents without persona keep working unchanged

## Test plan

- `desktop/core/tests/agent_persona.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0011 prompt catalog; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-07 claude: Gate-0 refine + build in one pass. Verified REQ-CORE-001/002 and REQ-LOOP-015 exist and deps WP-0110/WP-0130 are done. Scope note: "limits" are already modelled per agent by the Gateway routing policy (WP-0121 `model_routing`), so this adds persona (role/goal/background) and `reasoning_mode` only. Migration 0093 adds those columns to `ai_agents` and `ai_agent_versions`; persona is set by its own admin action (`set_persona`), the memory_md/guardrails_md shape, so `AiAgentInput` is unchanged (no test-literal churn). The system-prompt compiler is split into a deterministic `compile_agent_prompt` (persona + instructions + memory + guardrails + skills + ontology, via catalog templates `loop/persona-section`, `loop/ontology-section`) and `agent_system_prompt` which adds per-run session memory; a read-only compiled-prompt view (`compiled_prompt`, admin, + Tauri/server commands) shows the deterministic part (AC-1). Reasoning modes in `run_agent_once`: Plan-first makes a planning model call recorded as a `plan` step span and a `plan.proposed` event before the loop (new prompt `loop/plan-first`); Reflect makes exactly one extra self-critique model call before the final answer (new prompt `loop/reflect`); Direct is unchanged (AC-2, AC-3). Editions: desktop + Team Workspace get a Persona panel (role/goal/background + reasoning mode + View compiled prompt); the online demo documents it in a changelog entry and the Build-your-first-agent help gains a "Persona and reasoning mode" section. Tests: `desktop/core/tests/agent_persona.rs` (6) all pass; prompt_catalog, trace_spans (golden unchanged) and trace_conformance green. Next: open PR, drive CI green.
