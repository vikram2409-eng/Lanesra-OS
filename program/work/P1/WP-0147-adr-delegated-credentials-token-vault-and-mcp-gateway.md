---
id: WP-0147
title: ADR: delegated credentials, token vault and MCP gateway
phase: 1
epic: E1.5 Tools and firewall
status: in_review
owner: muse
planned_for: muse
depends_on: [WP-0143, WP-0108]
requirements: [REQ-CRED-001]
estimate: M
editions: []
delivered: []
branch: wp/WP-0147-adr-delegated-credentials
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/768
updated: 2026-11-14
---

# WP-0147 — ADR: delegated credentials, token vault and MCP gateway

## Goal

Fix how a tool call acts as the user in an outside system. Today a connector holds one shared credential (WP-0143 audit), so an agent cannot respect per-user permissions in the systems it reaches. The design must exist before Phase 3 adds team tools on top of the connector layer.

## Scope

- An accepted ADR covering: the grant model (user or Agent Identity, connector, scopes, expiry); token vault storage under the workspace data key (ADR-0014); how the effective identity of a run selects a grant; refusal rules for unattended runs; MCP server registry with pinned tool definitions; the MCP gateway endpoint and its keys
- Threat model additions: token theft, confused deputy, tool-definition change after approval

## Out of scope

- Any product code (WP-0510 to WP-0513)

## Acceptance criteria

- [ ] AC-1: The ADR is Accepted by the owner and includes the grant table shape and the selection rule as a decision table
- [x] AC-2: The threat model lists the three new threats with their mitigations
- [x] AC-3: WP-0142, WP-0510 and WP-0512 cite the ADR in Notes
- [x] AC-4: `programctl validate` passes

## Test plan

- Documentation only: `python3 program/tools/programctl.py validate`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Follows ADR-0006, ADR-0014, ADR-0018 and the WP-0143 audit. Design section: SOLUTION-DESIGN.md §6.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-14 muse: ADR-0026 written (Proposed) from SOLUTION-DESIGN.md §6, ADR-0006/0014/0018 and the WP-0143 audit. AC-2 (threat model: token theft, confused deputy, tool-definition change) and AC-3 (WP-0142, WP-0510, WP-0512 cite ADR-0026) done. AC-1 stays open: the ADR stays Proposed until the owner accepts it.
- 2026-11-14 muse: PR #768 opened (draft). Status in_progress → in_review. AC-1 (owner acceptance) is the only open criterion.
