---
id: WP-0130
title: Prompt catalog, written fresh
phase: 1
epic: E1.4 Task contracts and guardrails
status: proposed
owner: unassigned
depends_on: [WP-0009]
requirements: [REQ-LOOP-016]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: internal engine work with no user-visible surface of its own; the online demo's simulated runs reflect it through the Playground work packages
docs: []
docs_reason: internal prompt assets
branch: 
pr: 
updated: 2026-10-16
---

# WP-0130 — Prompt catalog, written fresh

## Goal

All system prompts become versioned catalog assets written fresh in calm language, diffable, and evaluated when changed.

## Scope

- Catalog storage per ADR
- Migrate every existing prompt string in core into the catalog
- Rewrite where needed; no pressure language
- Prompt version recorded on spans

## Acceptance criteria

- [ ] AC-1: No hard-coded system prompt strings remain in services (grep test)
- [ ] AC-2: Changing a prompt bumps its version and is visible in a diff

## Test plan

- `desktop/core/tests/prompt_catalog.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
