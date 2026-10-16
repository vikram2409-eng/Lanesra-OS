---
id: WP-0122
title: Native function calling and strict JSON tool format
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
depends_on: [WP-0005]
requirements: [REQ-LOOP-004]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0122 — Native function calling and strict JSON tool format

## Goal

Use each provider's native tool calling (Anthropic, OpenAI-compatible, Gemini, Ollama where supported); otherwise a strict JSON tool-call format with schema validation. No free-text ReAct parsing.

## Scope

- Provider capability matrix
- Strict JSON fallback with repair-free parsing and clear error to the model
- Malformed tool calls count toward failure caps

## Acceptance criteria

- [ ] AC-1: Tool calls round-trip for each provider adapter (scripted provider emulating each wire format)
- [ ] AC-2: Malformed JSON returns an error message to the model, never executes a tool

## Test plan

- `desktop/core/tests/tool_calling_formats.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
