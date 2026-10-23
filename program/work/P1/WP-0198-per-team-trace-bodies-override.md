---
id: WP-0198
title: Per-Team override of trace bodies
phase: 1
epic: E1.2 Run model and trace
status: proposed
owner: unassigned
depends_on: [WP-0111]
requirements: [REQ-OBS-002]
estimate: S
editions: [desktop, workspace]
delivered: []
editions_reason: an admin setting for Team Workspace and desktop; the online demo does not record traces
docs: [run-traces]
branch: 
pr: 
updated: 2026-10-23
---

# WP-0198 — Per-Team override of trace bodies

## Goal

ADR-0013 #2 and ADR-0008 §4 set `trace_bodies` per workspace "with an optional override per Team". WP-0111 built the workspace setting; this adds the Team override once Teams exist.

## Scope

- A Team's own `trace_bodies` value, falling back to the workspace's
- A run uses the setting of the Team it runs for; the trace records which applied
- AI Admin only, audited, like the workspace setting

## Acceptance criteria

- [ ] AC-1: A run for a Team with an override stores bodies as that override says, and its root span records the Team setting
- [ ] AC-2: Changing a Team override is Administrator-only and audited

## Test plan

- `desktop/core/tests/trace_spans.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Blocked until Lanesra OS has Teams (no Teams table exists today). Split from WP-0111 on 2026-10-23.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-23 claude: Written when WP-0111 was refined: the per-Team override needs Teams, which do not exist yet.
