---
id: WP-0130
title: Prompt catalog, written fresh
phase: 1
epic: E1.4 Task contracts and guardrails
status: in_progress
owner: claude
depends_on: [WP-0009]
requirements: [REQ-LOOP-016]
estimate: L
editions: [desktop, workspace]
delivered: [desktop, workspace]
editions_reason: internal engine work with no user-visible surface of its own; the online demo's simulated runs reflect it through the Playground work packages
docs: []
docs_reason: internal prompt assets
branch: claude/enterprise-build-planning-jxy2kq
pr: 
updated: 2026-10-23
---

# WP-0130 — Prompt catalog, written fresh

## Goal

All system prompts become versioned catalog assets written fresh in calm language, diffable, and evaluated when changed.

## Scope

- Catalog storage per ADR
- Migrate every existing prompt string in core into the catalog
- Rewrite where needed; no pressure language
- Prompt version recorded on spans: `render_active` returns the `PromptRef` (id and version) each call used; writing it onto the span needs the span store from WP-0111
- Out of scope: fencing `untrusted` variables before insertion (WP-0145; the variables are marked in front matter now)

## Acceptance criteria

- [x] AC-1: No hard-coded system prompt strings remain in services (grep test)
- [x] AC-2: Changing a prompt bumps its version and is visible in a diff

## Test plan

- `desktop/core/tests/prompt_catalog.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0011 prompt catalog; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-23 claude: Reassigned from Muse to Claude by the owner (Muse is unavailable for about a week). Muse's branch `wp/WP-0130-prompt-catalog` was last updated on 2026-10-18; Claude will build from it where it is usable and record what was kept.
- 2026-10-23 claude: Muse's branch held no commits beyond main; Muse's earlier work had already merged (the `prompts` module and eight prompt files, wired into chat, the agent loop, voice and the judge). Finished the catalog to ADR-0011: front matter is parsed (serde_yaml) and `render_active` checks every required variable is given and no undeclared one is; the active version is the highest `active` one, so drafts and retired versions are never picked; a missing prompt is now an error everywhere instead of an empty system prompt (chat and voice used to fall back to ""). The last hard-coded prompt (the report-directive prompt in `agent_service`) moved to `assistant/report-directive.v1`, rewritten with the contract first. `voice/planner` v2 replaces v1 (retired) to drop capitalised instructions, the first change made through the catalog's own process. Every prompt now has an evaluation suite (`*.cases.json`; the judge's front matter named one that did not exist). `prompts/catalog.lock` pins each released body by SHA-256. `core/tests/prompt_catalog.rs` (7 tests) checks files against the registry, front matter, placeholders against variables, contiguous versions with one active, every suite, the style rules (no shouting, pressure phrases or other frameworks' names), the lock (AC-2) and that no service keeps prompt text (AC-1); both AC tests were shown to fail on a deliberate violation, then restored.
- 2026-10-23 claude: A second prompt was still hard-coded: the structured-output repair message in the agent loop. It is now `loop/output-repair.v1`, and the AC-1 check also matches its kind of opening; a new loop test (`an_answer_that_misses_the_output_schema_gets_the_catalog_repair_prompt`) sends an answer that misses the schema and checks the catalog text goes back. Tool descriptions stay with the tool definitions in code: they are part of each tool's schema, not a system prompt.
