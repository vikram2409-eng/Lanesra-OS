---
id: WP-0133
title: Guardrails as versioned policy objects
phase: 1
epic: E1.4 Task contracts and guardrails
status: in_review
owner: claude
depends_on: [WP-0009]
requirements: [REQ-GOV-007]
threats: [T-01]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [guardrails]
branch: claude/enterprise-build-planning-jxy2kq
pr: pending
updated: 2026-11-05
---

# WP-0133 — Guardrails as versioned policy objects

## Goal

Output guardrails (PII leak, tone, required disclaimer, schema conformance, external domain allow-list) as versioned, audited, reusable policy objects with rule-based or LLM-judge checkers.

## Scope

- `ai_guardrails` (rule, judge, schema, pii) with versions
- Built-in guardrail library
- Admin UI to list and edit

## Acceptance criteria

- [x] AC-1: Each built-in guardrail has positive and negative tests
- [x] AC-2: Editing creates a new version; runs record the version

## Test plan

- `desktop/core/tests/guardrails.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0011 prompt catalog; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-05 claude: Refined proposed -> ready and claimed (dep WP-0009 done). Scope confirmed against REQ-GOV-007 and ADR-0011 (prompt catalog for the judge), ADR-0010 (migration), ADR-0012 (feature flag). Building: ai_guardrails versioned policy objects with rule/judge/schema/pii checkers, a built-in library, evaluation service, and an admin screen.
- 2026-11-05 claude: Backend built and both ACs proven in core/tests/guardrails.rs (7 tests). Migration 0090 adds ai_guardrails (versioned, builtin_key, checker_type, current_version), ai_guardrail_versions (append-only configs) and ai_guardrail_evaluations (a run's recorded check + version). guardrail_service: a pure evaluate() for the four checker kinds - rule (must_contain / forbid_substrings / domain_allowlist via regex host extraction), pii (reuses dlp_service::scan), schema (a small dependency-free JSON-schema check), judge (interprets a supplied model verdict) - plus the five built-ins, create/edit (an edit writes a new version), ensure_builtins, and evaluate_for_run which records the version. Verified: core fmt/clippy --all-targets clean, cargo-deny ok, migration_upgrade + migrate unit (90) pass.
- 2026-11-05 claude: Stays in_review. Remaining for done (editions): the admin screen to list/edit guardrails (behind a guardrails feature flag, ADR-0012), the async judge model call rendered through the prompt catalog (ADR-0011) and wired where runs check output, the desktop Tauri + server commands, and the online-demo mirror.
