---
id: WP-0145
title: Minimal untrusted-content defence for tool results
phase: 1
epic: E1.5 Tools and firewall
status: done
owner: claude
depends_on: [WP-0122, WP-0128, WP-0011, WP-0130]
requirements: [REQ-TOOL-015]
threats: [T-02, T-14]
estimate: L
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [prompt-injection-defence]
branch: claude/enterprise-build-planning-jxy2kq
pr: 
updated: 2026-11-06
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

- [x] AC-1: An injection corpus (at least 30 cases: instruction override, tool-list change, exfiltration via external tool, hidden text) is run against a scripted agent; no case executes a write or external tool without approval
- [x] AC-2: A step that read no untrusted content keeps today's approval behaviour (no regression)
- [x] AC-3: Taint and fencing are visible in the trace and in the approval request
- [x] AC-4: Unknown tool sources are treated as untrusted (test)

## Test plan

- `desktop/core/tests/untrusted_content.rs` with the corpus as fixtures under `desktop/core/tests/fixtures/injection/`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Must follow (WP-0009): ADR-0008 trace and events; ADR-0010 migration conventions; ADR-0011 prompt catalog; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Threats: indirect prompt injection, excessive agency, data exfiltration (WP-0011 threat model).

## Progress log

- 2026-10-17 muse: Refined to ready per PROTOCOL.md §4. Threats linked (WP-0011 AC-3). ACs testable, test plan names files, dependencies and requirements listed.
- 2026-11-06 claude: Implemented all four ACs. `tool_registry_service::output_trust` classifies each tool's output trusted/untrusted (native record/admin/agent-internal reads trusted; connector, external MCP and unknown sources untrusted - AC-4). An untrusted, successful tool result taints the step and is fenced in the model's context with a catalogued wrapper (`prompts/tools/untrusted-fence.v1.md`, ADR-0011) stating it is data, not instructions; the trace keeps the raw result marked untrusted. A tainted step escalates any write-tier or external-action call to administrator approval via the existing Tool-Call Firewall (`apply_tool_firewall` gains a `tainted` arg; `policy_engine_service::taint_requires_approval` decides, respecting a new per-tool relax list), recorded on the firewall span (`lanesra.policy.tainted`) and the held approval (`tainted`/reason) - AC-1, AC-3. A step that reads no untrusted content is unchanged - AC-2. Instructions, the tool list and the policy are fixed before the loop and never changed by tool output. Wired in both the agent loop (`run_agent_once`) and the Assistant chat (`send_message`). Storage: migration 0092 adds `ai_agent_policies.taint_exempt_tool_names_json`. Editions: the Policy Engine panel (desktop + Team Workspace) gains a taint-exempt field; the online demo documents it in a changelog entry and the firewall help. Docs: new `/help/prompt-injection-defence`. Tests: `desktop/core/tests/untrusted_content.rs` (7) with a 32-case corpus under `tests/fixtures/injection/`; all four ACs proven. Next: open PR, drive CI green.

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
