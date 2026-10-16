---
id: WP-0146
title: Bring-your-own-model provider coverage and model catalog
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0005, WP-0122]
requirements: [REQ-LOOP-022, REQ-LOOP-023]
estimate: XL
branch: 
pr: 
updated: 2026-10-16
---

# WP-0146 — Bring-your-own-model provider coverage and model catalog

## Goal

Organisations using the Playground bring their own models: OpenAI, Anthropic Claude, Google Gemini,
Azure OpenAI, Google Vertex AI, AWS Bedrock, Ollama, or on-premises model servers with vLLM as the
first-class on-prem target (owner, 2026-10-16). Every Playground and orchestration feature
must work with whichever they configure, online or fully offline. Today the AI Gateway supports
`anthropic`, `google_gemini` and `openai_compatible` (`desktop/core/src/models/ai.rs:16`); close the
gaps and stop the loop from assuming one provider's behaviour.

## Scope

- Provider kinds: keep `anthropic`, `google_gemini`, `openai_compatible` (OpenAI, LM Studio, TGI, other compatible servers); add:
  - `vllm`: first-class on-prem target on its OpenAI-compatible API, with vLLM's tool-calling and guided JSON options, served-model discovery and health check
  - `ollama`: native API (model list, pull status, local tool calling)
  - `azure_openai`: resource endpoint + deployment name + `api-version`; auth by API key or Microsoft Entra ID (client credentials / managed identity)
  - `google_vertex_ai`: project + region + publisher model (Gemini and partner models such as Claude on Vertex); auth by service-account key or workload identity, OAuth token refresh
  - `aws_bedrock`: region + model id or inference profile, Converse API with tool use; auth by access key/secret, assumed IAM role or instance profile, AWS Signature V4 signing
- On-premises connectivity: custom base URL, private CA certificate bundle, optional mTLS client certificate, HTTP proxy, extra auth headers, timeouts; all secrets AES-256-GCM at rest as today
- Model catalog per workspace: capabilities (native tool calling, structured output, context window, streaming, vision), price per input/output token (default 0 for on-premises, editable), data-residency tag for the data-boundary policy (WP-0123)
- Capability discovery where the provider exposes it, admin override where it does not
- The loop uses the catalog: strict JSON tool format when native tool calling is missing (WP-0122), compaction sized to the catalog context window (WP-0125), cost from catalog prices (forecast WP-0319)
- Cloud credentials (Entra ID, service accounts, IAM) stored encrypted like other secrets; short-lived tokens cached in memory only, never persisted
- Region / residency recorded per connection so the data-boundary policy can restrict which data reaches which region

## Acceptance criteria

- [ ] AC-1: An agent and a pipeline run end to end, with tool calls, on each of the nine provider kinds, proven by contract fixtures (hosted) and real local runs on vLLM and Ollama
- [ ] AC-1b: Azure OpenAI (key and Entra ID), Vertex AI (service account) and Bedrock (SigV4 with static keys and assumed role) authenticate correctly against contract fixtures, including token expiry and refresh; SigV4 checked against AWS's published test vectors
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

- XL: split at refinement (Gate 0 review), e.g. (a) catalog + on-prem connectivity + vLLM + Ollama, (b) Azure OpenAI, (c) Vertex AI, (d) Bedrock.
- Use maintained, audited libraries for SigV4 and Google/Entra OAuth where possible; record them in THIRD_PARTY_NOTICES.
- The build team has no hosted-provider keys (ADR-0004); hosted adapters are proven by contract fixtures and by organisations' own conformance runs (WP-0224).

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
