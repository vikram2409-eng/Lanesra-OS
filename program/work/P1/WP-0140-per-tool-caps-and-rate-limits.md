---
id: WP-0140
title: Per-tool caps and rate limits
phase: 1
epic: E1.5 Tools and firewall
status: proposed
owner: unassigned
depends_on: [WP-0122]
requirements: [REQ-LOOP-014, REQ-TOOL-008]
threats: [T-03, T-06]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [policy-engine-tool-call-firewall]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0140 — Per-tool caps and rate limits

## Goal

Each tool binding caps calls per step, failures before disable, and result size kept in context; rate limits per tool, connector and workspace are enforced in the firewall before the call.

## Scope

- Caps on binding
- Large results stored and referenced
- Rate limiter in firewall

## Acceptance criteria

- [ ] AC-1: Exceeding each cap behaves as specified and is traced
- [ ] AC-2: Rate-limited call never reaches the tool

## Test plan

- `desktop/core/tests/tool_caps.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
