---
id: WP-0224
title: Real-model evaluation without paid API keys
phase: 2
epic: E2.3 Evaluation and publish
status: proposed
owner: unassigned
depends_on: [WP-0210, WP-0211]
requirements: [REQ-EVAL-012]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0224 — Real-model evaluation without paid API keys

## Goal

The program has no model API keys (owner decision, ADR-0004). Gate 2 still needs evidence that the
templates work with a real model, not only scripted ones. Use a local open-weight model through the
existing Ollama support, record the runs, and replay them in CI.

## Scope

- A documented local evaluation command that runs a suite against an Ollama model on the owner's or a contributor's machine
- Results and model responses saved as recorded runs (WP-0210) and committed as fixtures, so CI replays them for free
- Provider adapters for paid providers are verified against request/response fixtures taken from their public API documentation
- Record model name, version and hardware with every result

## Acceptance criteria

- [ ] AC-1: One command runs a template suite on a local Ollama model and writes a recorded-run fixture
- [ ] AC-2: CI replays that fixture with identical scores
- [ ] AC-3: Each paid-provider adapter has fixture-based contract tests

## Test plan

- `desktop/core/tests/provider_contracts.rs`; replay tests from WP-0210
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Refine at the Gate 1 review.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
