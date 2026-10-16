---
id: WP-0143
title: Tool catalog, credentials and delegation guard audit
phase: 1
epic: E1.5 Tools and firewall
status: proposed
owner: unassigned
depends_on: [WP-0008]
requirements: [REQ-TOOL-001, REQ-TOOL-007, REQ-LOOP-012]
estimate: S
editions: []
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0143 — Tool catalog, credentials and delegation guard audit

## Goal

Confirm and lock in with tests that every tool source passes the full firewall chain, tools never see credentials, and delegation beyond depth 4 returns a readable refusal while the run continues.

## Scope

- Tests only, plus fixes for any gap the tests reveal
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - Agent tool dispatch does not check the agent's own `action_names` (I-16, security): write a test proving an agent cannot call an unlisted tool (including admin tools), then enforce membership before dispatch
  - Detect A→B→A cycles in nested `run_agent_team` nodes (only direct self-recursion is blocked)

## Acceptance criteria

- [ ] AC-1: Each tool source has a firewall-order test
- [ ] AC-2: Credential never present in tool input or trace
- [ ] AC-3: Depth-5 delegation refused, run continues

## Test plan

- `desktop/core/tests/firewall_chain.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy; ADR-0010 migration conventions. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
