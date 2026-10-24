---
id: WP-0199
title: Dollars remaining in agent turns
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
depends_on: [WP-0126, WP-0146]
requirements: [REQ-LOOP-020]
estimate: S
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-limits-and-partial-results]
branch: 
pr: 
updated: 2026-10-24
---

# WP-0199 — Dollars remaining in agent turns

## Goal

REQ-LOOP-020 asks that each agent turn is told the dollars left as well as the steps and tokens WP-0126 already reports. That needs a price per model, which arrives with the model catalog (WP-0146).

## Scope

- A cost limit per run on the agent's routing, next to the token limit
- The budget line (`loop/budget-line`, a new version) states the dollars left
- Model-call spans gain `lanesra.cost_usd` (ADR-0008)

## Acceptance criteria

- [ ] AC-1: With prices for the agent's model, the dollars left are correct each round (tested)
- [ ] AC-2: Without a price for the model, the line says so instead of guessing

## Test plan

- Extend `desktop/core/tests/agent_loop_limits.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Split from WP-0126 on 2026-10-24.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-24 claude: Written when WP-0126 was refined: dollars need model prices (WP-0146).
