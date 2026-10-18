---
id: WP-0133
title: Guardrails as versioned policy objects
phase: 1
epic: E1.4 Task contracts and guardrails
status: proposed
owner: unassigned
depends_on: [WP-0009]
requirements: [REQ-GOV-007]
threats: [T-01]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [guardrails]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0133 — Guardrails as versioned policy objects

## Goal

Output guardrails (PII leak, tone, required disclaimer, schema conformance, external domain allow-list) as versioned, audited, reusable policy objects with rule-based or LLM-judge checkers.

## Scope

- `ai_guardrails` (rule, judge, schema, pii) with versions
- Built-in guardrail library
- Admin UI to list and edit

## Acceptance criteria

- [ ] AC-1: Each built-in guardrail has positive and negative tests
- [ ] AC-2: Editing creates a new version; runs record the version

## Test plan

- `desktop/core/tests/guardrails.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0011 prompt catalog; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
