---
id: WP-0122
title: Native function calling and strict JSON tool format
phase: 1
epic: E1.3 Agent loop
status: done
owner: claude
depends_on: [WP-0005]
requirements: [REQ-LOOP-004]
threats: [T-03, T-14]
estimate: L
editions: [desktop, workspace]
delivered: [desktop, workspace]
editions_reason: provider wire formats; the online demo uses a simulated model
docs: [unified-ai-gateway]
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/717
updated: 2026-10-23
---

# WP-0122 — Native function calling and strict JSON tool format

## Goal

Use each provider's native tool calling (Anthropic, OpenAI-compatible, Gemini, Ollama where supported); otherwise a strict JSON tool-call format with schema validation. No free-text ReAct parsing.

## Scope

- Provider capability matrix
- Strict JSON fallback with repair-free parsing and clear error to the model
- Malformed tool calls count toward failure caps (three per run)
- Out of scope: switching a given model to the strict JSON format; which models lack native tool calling comes from the model catalog (WP-0146), which will select `ToolCallFormat::StrictJson` for them

## Acceptance criteria

- [x] AC-1: Tool calls round-trip for each provider adapter (scripted provider emulating each wire format)
- [x] AC-2: Malformed JSON returns an error message to the model, never executes a tool

## Test plan

- `desktop/core/tests/tool_calling_formats.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0011 prompt catalog; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Must hold for every provider an organisation brings (REQ-LOOP-022); capabilities come from the model catalog in WP-0146.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
2026-10-18 muse: Refined to ready. Scope confirmed: native tool calling for Anthropic/OpenAI-compatible/Gemini, strict JSON fallback, malformed → validation.arguments error to model (never executes). AC-1 (round-trip per provider via scripted provider) and AC-2 (malformed JSON → error, no execution) are testable. Test plan: desktop/core/tests/tool_calling_formats.rs. Threats T-03, T-14 linked.
- 2026-10-23 claude: Reassigned from Muse to Claude by the owner (Muse is unavailable for about a week). Muse's branch `wp/WP-0122-tool-calling` was last updated on 2026-10-18; Claude will build from it where it is usable and record what was kept.
- 2026-10-23 claude: Muse's branch held no commits beyond main. Muse's earlier merged work made the OpenAI-compatible adapter fail the whole response on malformed arguments, and `tests/tool_calling_formats.rs` only parsed hand-written JSON without calling any adapter. Rebuilt to the acceptance criteria. Adapters now mark a call `malformed` (unparseable JSON, or arguments that are not an object) instead of failing the response; `ai_service::check_tool_call` also checks required parameters and declared JSON types. Both loops (chat `send_message` and the agent loop) answer a malformed call with `validation.arguments` and what was wrong, never run it, and stop the run after `MAX_MALFORMED_TOOL_CALLS` (3). Capability matrix `TOOL_CALL_CAPABILITIES` (all three adapters native); `tool_call_format` gives an unknown provider the strict JSON format; `parse_strict_json_reply` reads exactly one JSON object with no repair (no fence stripping, no surrounding text); catalog prompt `loop/json-tool-format.v1` describes tools for it. `tests/tool_calling_formats.rs` rewritten: 9 tests through the real adapters against a local server speaking each provider's wire format, including the tool result sent back in each format.
- 2026-10-23 claude: In review in #717.
- 2026-10-23 claude: Merged in #717. Done.
