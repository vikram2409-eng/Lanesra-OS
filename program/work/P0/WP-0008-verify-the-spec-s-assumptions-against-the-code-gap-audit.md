---
id: WP-0008
title: Verify the spec's assumptions against the code (gap audit)
phase: 0
epic: E0.3 Architecture decisions
status: in_review
owner: claude
depends_on: []
requirements: [REQ-PRE-008]
estimate: M
editions: []
delivered: []
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/626
updated: 2026-10-16
---

# WP-0008 — Verify the spec's assumptions against the code (gap audit)

## Goal

The spec says 'most of the plumbing already exists'. Before designing on top of it, confirm each claim with file and line references and record every discrepancy. Two are already known: there is no THIRD_PARTY_NOTICES file, and there is no OIDC/SAML code.

## Scope

- For every row of the spec's 'Where this fits' table and 'Code map', record: exists / partly / missing, with `path:line` evidence and notes
- Confirm the three legacy pause-for-human paths in `ai_orchestration_service` and where each lives
- Confirm `MAX_ROUNDS`, delegation depth guard, the 13 node types and how parallel branches execute
- Confirm how events_stream SSE is structured today and who consumes it
- Record where desktop (Tauri) and server diverge for AI features
- Output: `program/research/gap-audit.md`; each discrepancy becomes a RAID item or an ADR

## Acceptance criteria

- [x] AC-1: Every spec 'existing piece' has a verdict with evidence
- [x] AC-2: Discrepancies are logged in `program/RAID.md` with an owner
- [x] AC-3: Phase 1 work packages updated where the audit changes their scope

## Test plan

- Not a code change; reviewed by a second agent or the owner
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-10-16 claude: Audit written to `program/research/gap-audit.md`. Every row of the spec's "Where this fits" table and Code map has a verdict with `path:line` evidence, plus each extra question in Scope (pause paths, MAX_ROUNDS, delegation guard, node types, parallel branches, SSE, desktop/server divergence).
  - Method: three read-only search passes in parallel (runtime, governance, UI and editions), then the key claims re-read by hand (list in the audit's Method section).
  - AC-2: 12 issues (RAID I-12 to I-23) and 4 tech-debt items (T-02 to T-05), each routed to a work package. Most serious: runs not resumed after restart (I-12); agent tool calls not checked against the agent's own tool list (I-16, security, to prove with a test in WP-0143); published version not what runs (I-18); air-gap fails open (I-17).
  - AC-3: scope notes added to WP-0100, 0112, 0121, 0123, 0132, 0141, 0143, 0146, 0150 (Phase 1), plus WP-0009, WP-0013 and WP-0306; new proposed WP-0170 (run the published agent version).
  - State: in_review on PR #626, documentation only. Next: a second agent or the owner reviews (test plan); then WP-0009 (foundational ADRs) can start with RAID I-23 as input.
  - Gotcha: the code comment at `graph_runtime_service.rs:33-37` points to a "three legacy paths" note that does not exist; the audit's §2 has the real list.
