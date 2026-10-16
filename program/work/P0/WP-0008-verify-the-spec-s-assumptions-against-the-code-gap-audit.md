---
id: WP-0008
title: Verify the spec's assumptions against the code (gap audit)
phase: 0
epic: E0.3 Architecture decisions
status: ready
owner: unassigned
agent: claude
depends_on: []
requirements: [REQ-PRE-008]
estimate: M
branch: 
pr: 
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

- [ ] AC-1: Every spec 'existing piece' has a verdict with evidence
- [ ] AC-2: Discrepancies are logged in `program/RAID.md` with an owner
- [ ] AC-3: Phase 1 work packages updated where the audit changes their scope

## Test plan

- Not a code change; reviewed by a second agent or the owner
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
