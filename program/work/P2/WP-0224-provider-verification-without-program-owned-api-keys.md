---
id: WP-0224
title: Provider verification without program-owned API keys
phase: 2
epic: E2.3 Evaluation and publish
status: proposed
owner: unassigned
depends_on: [WP-0210, WP-0211, WP-0146]
requirements: [REQ-EVAL-012]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0224 — Provider verification without program-owned API keys

## Goal

Organisations bring their own models - OpenAI, Gemini, Claude, Ollama or on-premises servers
(REQ-LOOP-022). The build team has no API keys of its own (ADR-0004), so we cannot call the hosted
APIs in CI. Prove provider support another way: contract tests against each provider's published
format, real-model runs on a local model, recorded runs replayed in CI, and a conformance suite an
organisation runs from the product with its own key.

## Scope

- A provider conformance suite (in-product, via the Test button from WP-0144) that an organisation runs with its own key: text, tool calling, structured output, streaming, errors and limits, reported in plain words
- A documented local evaluation command that runs a suite against an Ollama or vLLM model on a contributor's machine
- Results and model responses saved as recorded runs (WP-0210) and committed as fixtures, so CI replays them for free
- Provider adapters for paid providers are verified against request/response fixtures taken from their public API documentation
- Record model name, version and hardware with every result

## Acceptance criteria

- [ ] AC-1: One command runs a template suite on a local Ollama model and writes a recorded-run fixture
- [ ] AC-2: CI replays that fixture with identical scores
- [ ] AC-3: Each provider adapter (OpenAI, Anthropic, Gemini, OpenAI-compatible, Ollama) has fixture-based contract tests
- [ ] AC-4: The conformance suite runs against a local OpenAI-compatible server in CI and reports each capability

## Test plan

- `desktop/core/tests/provider_contracts.rs`; replay tests from WP-0210
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Refine at the Gate 1 review.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
