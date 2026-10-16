---
id: WP-0145
title: Minimal untrusted-content defence for tool results
phase: 1
epic: E1.5 Tools and firewall
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0122, WP-0128, WP-0011, WP-0130]
requirements: [REQ-TOOL-015]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0145 — Minimal untrusted-content defence for tool results

## Goal

External tools (connectors, external MCP servers, web and email content) already run today, and
the full content-label system is not due until Phase 4. The owner decided to move a minimal
defence forward (ADR-0004) so prompt injection through tool results is contained from Phase 1.

## Scope

- Tool registry marks each tool's output trust: trusted (native record reads) or untrusted (web, email, connector, external MCP); unknown defaults to untrusted
- Untrusted results are fenced in the prompt with a fixed, catalogued wrapper that states the content is data, not instructions (prompt catalog, WP-0130)
- Instructions, the tool list and the agent's policy cannot be changed by tool output (enforced in code, not only in the prompt)
- Taint flag per step: once a step has read untrusted content, any write-tier or external-action tool call in that step needs approval by default (policy can relax per tool)
- Trace marks untrusted spans; the approval shows that the step read untrusted content
- Designed so Phase 4 content labels (WP-0406) extend it rather than replace it

## Acceptance criteria

- [ ] AC-1: An injection corpus (at least 30 cases: instruction override, tool-list change, exfiltration via external tool, hidden text) is run against a scripted agent; no case executes a write or external tool without approval
- [ ] AC-2: A step that read no untrusted content keeps today's approval behaviour (no regression)
- [ ] AC-3: Taint and fencing are visible in the trace and in the approval request
- [ ] AC-4: Unknown tool sources are treated as untrusted (test)

## Test plan

- `desktop/core/tests/untrusted_content.rs` with the corpus as fixtures under `desktop/core/tests/fixtures/injection/`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Threats: indirect prompt injection, excessive agency, data exfiltration (WP-0011 threat model).

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
