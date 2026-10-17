---
id: WP-0012
title: Clean-room controls
phase: 0
epic: E0.4 Security and compliance baseline
status: done
owner: muse
depends_on: []
requirements: [REQ-CLEAN-001, REQ-CLEAN-002, REQ-CLEAN-003, REQ-CLEAN-004, REQ-CLEAN-005]
estimate: S
editions: []
delivered: []
branch: wp/WP-0012-clean-room-controls
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/642
updated: 2026-10-17
---

# WP-0012 — Clean-room controls

## Goal

The spec requires a clean room: ideas from CrewAI, Agent Framework, LangGraph and AutoGen, never code, prompts or structure. With AI implementers this needs explicit, checkable rules.

## Scope

- `THIRD_PARTY_NOTICES` at the repo root (spec assumes it exists; it does not)
- Clean-room rules in AGENTS.md: implementer agents must not fetch or open those projects' source; analyst work goes in `program/research/` as plain-language behaviour notes only
- PR template checklist (no copied prompt, no transliterated function, design traced to spec)
- A banned-vocabulary check in CI for new code: Crew, Kickoff, Superstep, Magentic, StateGraph, GraphFlow, Swarm in identifiers (allow-list for docs and research notes)

## Acceptance criteria

- [x] AC-1: THIRD_PARTY_NOTICES exists and lists current notable dependencies' licences
- [x] AC-2: Vocabulary check runs in CI and fails on a planted identifier
- [x] AC-3: AGENTS.md contains the analyst/implementer rule

## Test plan

- `program/tools/check_vocabulary.py` with a self-test
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-17 claude: Closeout (review by Claude Code). Merged in PR #642. AC-1: THIRD_PARTY_NOTICES at the repo root lists the direct dependencies and their licences. AC-2: the `clean-room vocabulary` CI job runs `check_vocabulary.py` on changed files and its `--self-test`, which asserts a planted `crew`/`kickoff` identifier is flagged. AC-3: AGENTS.md "Clean room" rule separates analysts and implementers.
