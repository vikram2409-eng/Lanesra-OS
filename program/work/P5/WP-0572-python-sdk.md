---
id: WP-0572
title: Python SDK
phase: 5
epic: E5.8 Developer surface
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0325]
requirements: [REQ-DEV-003]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: a client library; nothing to show in the demo
docs: [developer-sdk]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0572 — Python SDK

## Goal

Deliver the SDK decided in ADR-0017 so developers can script agents, runs and evaluations.

## Scope

- Typed clients generated from the API description; streaming events; pagination, retries and idempotency keys; examples; published package

## Acceptance criteria

- [ ] AC-1: Each client method is covered by a test against a server running the scripted provider
- [ ] AC-2: A streamed run yields the documented events in order
- [ ] AC-3: SDK and API versions are checked for compatibility in CI

## Test plan

- `sdk/python/tests/`; CI job running them against the server image
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0017.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
