---
id: WP-0128
title: Policy hook points around step, model call and tool call
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0122, WP-0133]
requirements: [REQ-LOOP-017]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0128 — Policy hook points around step, model call and tool call

## Goal

Admins attach policies and guardrails at three points in the loop, configured as Policy objects, no code.

## Scope

- Hook execution order and short-circuit semantics
- Verdicts recorded on spans

## Acceptance criteria

- [ ] AC-1: A policy at each hook point can block, and the block is traced and audited

## Test plan

- `desktop/core/tests/policy_hooks.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
