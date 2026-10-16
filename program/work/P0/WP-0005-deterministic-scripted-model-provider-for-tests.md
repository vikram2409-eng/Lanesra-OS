---
id: WP-0005
title: Deterministic scripted model provider for tests
phase: 0
epic: E0.2 Test infrastructure
status: ready
owner: unassigned
depends_on: []
requirements: [REQ-PRE-003]
estimate: M
editions: []
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0005 — Deterministic scripted model provider for tests

## Goal

Every later work package tests agent behaviour (limits, retries, guardrails, delegation, modes). Those tests must be deterministic and run offline without API keys. Provide a scripted provider behind the AI Gateway that returns pre-defined responses and records every request.

## Scope

- A `scripted` provider kind in the AI Gateway, available only under `cfg(test)` or a test-support feature, never selectable in the product UI or settings API
- A script format: ordered responses (text, tool calls, final answer), token counts, simulated latency, and injected failures (rate limit 429, timeout, 5xx, malformed tool JSON)
- Request capture: tests can assert on the messages, tools and model parameters sent
- Helpers in a shared test-support module used by core and server tests
- Port at least the existing pipeline topology tests to use it where they currently stub differently

## Acceptance criteria

- [ ] AC-1: A test can script a 3-round tool-calling exchange and assert the exact tool calls executed
- [ ] AC-2: Injected 429 and timeout errors surface as the gateway's normal error types (so retry logic can be tested)
- [ ] AC-3: The provider cannot be configured through `ai_settings` or any API in a release build (test proves it)
- [ ] AC-4: Documented in `program/testing/TEST-STRATEGY.md` with an example

## Test plan

- `desktop/core/tests/scripted_provider.rs` covering responses, failures, capture and release-build exclusion
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0011 prompt catalog. ADR-0010 applies to any migration this work package adds.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
