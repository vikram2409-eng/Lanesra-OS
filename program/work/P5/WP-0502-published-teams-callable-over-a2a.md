---
id: WP-0502
title: Published Teams callable over A2A
phase: 5
epic: E5.1 Agent control plane
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0114, WP-0325, WP-0411]
requirements: [REQ-API-010]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: protocol endpoints need a running server; nothing to simulate in the demo
docs: [agent-control-plane, api-access-and-webhooks]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0502 — Published Teams callable over A2A

## Goal

Let other systems call a published Lanesra Team as an A2A agent, so Lanesra Teams take part in an organisation's wider agent estate. Pulled forward from v2 (ADR-0020).

## Scope

- A2A server endpoint and agent card per published Team, behind scoped API keys and the network rule of ADR-0018
- Task lifecycle mapped onto runs: start, stream, input required (approvals and human input), cancel
- Caller identity mapped to an API client principal; the run is attributed to it

## Out of scope

- AG-UI (stays in WP-0901)

## Acceptance criteria

- [ ] AC-1: A conformance test client discovers the agent card, starts a task, receives streamed updates and a final result
- [ ] AC-2: A call without the scope, from outside the allowed network, or to an unpublished Team is refused
- [ ] AC-3: An approval raised inside the run surfaces to the caller as input required and resumes when answered
- [ ] AC-4: Every A2A call appears in run history with its API client

## Test plan

- `desktop/server/tests/a2a_server.rs` with the scripted provider
- Protocol conformance fixtures written from the public protocol description (clean room)
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR from WP-0114; ADR-0018. Feature flag `a2a_server`.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
