---
id: WP-0113
title: Full-trace conformance suite (Phase 1 gate test)
phase: 1
epic: E1.2 Run model and trace
status: done
owner: claude
depends_on: [WP-0112, WP-0122, WP-0150, WP-0107, WP-0108, WP-0109]
requirements: [REQ-OBS-003, REQ-NFR-003]
estimate: M
editions: []
delivered: []
branch: claude/enterprise-build-planning-jxy2kq
pr: 732
updated: 2026-11-03
---

# WP-0113 — Full-trace conformance suite (Phase 1 gate test)

## Goal

Prove the Phase 1 gate - every run produces a full trace - and that 100% of tool calls have a complete audit trail, across every way a run can start.

## Scope

- Matrix: chat agent, pipeline (sequential, consensus, peer review), workflow agent action, graph agent node, schedule trigger, webhook trigger, MCP-initiated, CLI-initiated
- For each: trace completeness check and tool audit completeness check

## Acceptance criteria

- [x] AC-1: Every matrix cell passes
- [x] AC-2: Suite is part of the regression gate

## Test plan

- `desktop/core/tests/trace_conformance.rs`, `desktop/server/tests/trace_conformance.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0008 trace and events; ADR-0010 migration conventions. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-01 claude: Refined proposed → ready and claimed. Dependencies done: WP-0112, WP-0122, WP-0150, and WP-0100's three parts WP-0107 to WP-0109 (WP-0100 itself was split and is closed as dropped). Interpretation of the matrix: MCP and the command-line tool start no run directly; a record they create starts one through a "Run AI agent" workflow, so those cells create a record over MCP `tools/call` and with the real CLI (`lanesra records create`) and check the run it starts. A chat with an agent is a conversation, not a run, and keeps no trace (WP-0111); its cell checks the tool audit and that no trace is written.
- 2026-11-01 claude: Built. `core/tests/support/trace_conformance.rs` defines the two checks: a full trace (one closed root, every span closed and parented in one trace, a step span per agent step, model calls in steps with token counts, each tool call in a model call with one firewall verdict, run events 1..n from `run.started` to a terminal event, each schema-valid) and a complete tool audit (one `ai_agent_tool_call` entry per tool call that reached the firewall, naming agent, tool and effective principal). Core cells (7 tests, one proving the checks refuse a broken trace and a missing audit) and server cells (3 tests: webhook route, MCP-started, CLI-started). The scripted provider gains a shared queue for a run a server executes on its own thread (test builds only). Gate G1 checks 1, 2, 3, 5 and 6 recorded with evidence. AC-2: both files run in CI's required `rust (tests)` check and are listed in REGRESSION.md.
- 2026-11-01 claude: In review in #732.
- 2026-11-03 claude: #732 merged to main. The conformance suite runs in CI's required `rust (tests)` check and is listed in REGRESSION.md; both ACs proven. Done.
