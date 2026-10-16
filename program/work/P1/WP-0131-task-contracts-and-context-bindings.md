---
id: WP-0131
title: Task contracts and context bindings
phase: 1
epic: E1.4 Task contracts and guardrails
status: proposed
owner: unassigned
depends_on: [WP-0110]
requirements: [REQ-CORE-003, REQ-CORE-004, REQ-LOOP-002]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0131 — Task contracts and context bindings

## Goal

A Task declares instruction, expected output, optional output schema, context bindings with token budgets, guardrails, retry budget, assignee and approval gate; context assembly records a receipt.

## Scope

- `ai_tasks` model attached to today's pipeline steps and graph agent nodes
- Binding resolver for step:, record:, knowledge:, memory:
- Per-binding trimming and context receipt

## Acceptance criteria

- [ ] AC-1: Nothing reaches the prompt except bound inputs (test asserts captured request)
- [ ] AC-2: Receipt lists every binding with tokens used and trimmed

## Test plan

- `desktop/core/tests/task_contracts.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
