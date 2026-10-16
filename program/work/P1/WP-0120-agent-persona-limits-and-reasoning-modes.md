---
id: WP-0120
title: Agent persona, limits and reasoning modes
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
depends_on: [WP-0110, WP-0130]
requirements: [REQ-CORE-001, REQ-CORE-002, REQ-LOOP-015]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [build-your-first-agent]
branch: 
pr: 
updated: 2026-10-16
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

- [ ] AC-1: Compiled prompt is deterministic for the same inputs (snapshot test)
- [ ] AC-2: Plan-first emits a plan span before tool use; Reflect makes exactly one extra model call
- [ ] AC-3: Existing agents without persona keep working unchanged

## Test plan

- `desktop/core/tests/agent_persona.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0011 prompt catalog; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
