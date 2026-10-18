---
id: WP-0132
title: Output validation, guardrail retries and on_fail
phase: 1
epic: E1.4 Task contracts and guardrails
status: proposed
owner: unassigned
depends_on: [WP-0131, WP-0133]
requirements: [REQ-LOOP-006, REQ-LOOP-007, REQ-LOOP-011, REQ-TOOL-004]
threats: [T-01]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [task-contracts, guardrails]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0132 — Output validation, guardrail retries and on_fail

## Goal

Validate each step output against its schema then guardrails; return reasons and retry within budget; then fail with verdicts and follow on_fail.

## Scope

- Schema validation
- Retry loop with feedback
- on_fail edge in graph runtime
- `final_output` tool short-circuit only when schema passes
- Persist outputs, memory updates, audit at step end
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - Output-schema validation already exists (`chat_service.rs:1858-1885`, one repair retry, partial JSON Schema): extend it rather than build a second one
  - Wire DLP `redact()` into output guardrails or remove it (T-02)

## Acceptance criteria

- [ ] AC-1: Invalid output then valid → success on retry
- [ ] AC-2: Exhausted retries → failed with all verdicts
- [ ] AC-3: on_fail edge followed

## Test plan

- `desktop/core/tests/output_validation.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0011 prompt catalog; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
