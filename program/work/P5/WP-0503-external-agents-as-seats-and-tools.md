---
id: WP-0503
title: External agents as seats and tools
phase: 5
epic: E5.1 Agent control plane
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0501, WP-0309, WP-0315]
requirements: [REQ-FLEET-003]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: calls go to real outside endpoints; the demo shows the registry only
docs: [agent-control-plane]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0503 — External agents as seats and tools

## Goal

Let a Team use an approved external agent like any other member, without weakening the firewall.

## Scope

- A2A client; an external agent can fill a seat or be bound as a tool
- Each call goes through schema → permission → policy → approval → execute → audit with a risk tier from the registry
- Replies fenced and labelled untrusted (WP-0145, WP-0406); timeouts, retries and idempotency as for tools
- Tokens and cost recorded when the peer reports them, otherwise marked unknown

## Out of scope

- Governance views (WP-0505)

## Acceptance criteria

- [ ] AC-1: A Team run delegates a step to an external agent and continues with its reply; the trace shows the call as a child step
- [ ] AC-2: A call to an unregistered, unapproved or disabled agent is refused with a readable reason and the run continues
- [ ] AC-3: An external reply cannot add tools or change instructions; a write-tier action based on it follows the untrusted-content rule
- [ ] AC-4: A peer that hangs is cancelled at the step wall-clock limit

## Test plan

- `desktop/core/tests/external_agent_seat.rs` with a scripted A2A peer
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR from WP-0114. Mitigates excessive agency and injection threats in the threat model.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
