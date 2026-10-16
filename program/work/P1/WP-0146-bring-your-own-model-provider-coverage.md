---
id: WP-0146
title: Bring-your-own-model provider coverage and model catalog
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
depends_on: [WP-0005, WP-0122]
requirements: [REQ-LOOP-022, REQ-LOOP-023]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0146 — Bring-your-own-model provider coverage and model catalog

## Goal

Organisations using the Playground bring their own models: OpenAI, Anthropic Claude, Google Gemini,
Ollama, or on-premises model servers (owner, 2026-10-16). Every Playground and orchestration feature
must work with whichever they configure, online or fully offline. Today the AI Gateway supports
`anthropic`, `google_gemini` and `openai_compatible` (`desktop/core/src/models/ai.rs:16`); close the
gaps and stop the loop from assuming one provider's behaviour.

## Scope

- Provider kinds: keep `anthropic`, `google_gemini`, `openai_compatible` (OpenAI, vLLM, LM Studio, TGI, other compatible servers); add a native `ollama` kind (model list, pull status, local tool-calling support)
- On-premises connectivity: custom base URL, private CA certificate bundle, optional mTLS client certificate, HTTP proxy, extra auth headers, timeouts; all secrets AES-256-GCM at rest as today
- Model catalog per workspace: capabilities (native tool calling, structured output, context window, streaming, vision), price per input/output token (default 0 for on-premises, editable), data-residency tag for the data-boundary policy (WP-0123)
- Capability discovery where the provider exposes it, admin override where it does not
- The loop uses the catalog: strict JSON tool format when native tool calling is missing (WP-0122), compaction sized to the catalog context window (WP-0125), cost from catalog prices (forecast WP-0319)
- Enterprise hosted variants to confirm with the owner (RAID Q-13): Azure OpenAI, AWS Bedrock, Google Vertex AI

## Acceptance criteria

- [ ] AC-1: An agent and a pipeline run end to end, with tool calls, on each provider kind, proven by contract fixtures (hosted) and a local OpenAI-compatible server and Ollama (real)
- [ ] AC-2: An on-premises endpoint with a private CA and a proxy connects; a wrong certificate is refused with a plain-language error
- [ ] AC-3: A model without native tool calling still runs tool-using agents through the strict JSON format
- [ ] AC-4: Cost shown in traces uses the catalog price, including zero-cost on-premises models
- [ ] AC-5: No credential appears in logs, traces or API responses (scan test)

## Test plan

- `desktop/core/tests/provider_contracts.rs` (fixtures per provider from published API formats)
- `desktop/core/tests/model_catalog.rs`
- Integration job against a local OpenAI-compatible stub and Ollama where available
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- The build team has no hosted-provider keys (ADR-0004); hosted adapters are proven by contract fixtures and by organisations' own conformance runs (WP-0224).

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
