---
id: WP-0011
title: Security baseline: threat model and scanning
phase: 0
epic: E0.4 Security and compliance baseline
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0003]
requirements: [REQ-PRE-009]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0011 — Security baseline: threat model and scanning

## Goal

Enterprise agents act on business records with real tools. Write the threat model before building, so every work package can cite the threats it must handle, and add automated scanning to CI.

## Scope

- `program/security/THREAT-MODEL.md`: assets, trust boundaries (UI, server, core, model providers, MCP servers, connectors), STRIDE plus LLM-specific threats (prompt injection direct and indirect, excessive agency, data exfiltration via tools, approval spoofing, cost exhaustion, memory poisoning)
- Each threat names mitigations and the requirement IDs that deliver them; gaps become RAID risks
- CI: `cargo deny` (or `cargo audit`) and `npm audit --omit=dev` with a documented exception list
- Secret scanning (GitHub secret scanning or gitleaks) on PRs
- A security review checklist added to the PR template for work packages tagged security

## Acceptance criteria

- [ ] AC-1: Threat model reviewed by the owner
- [ ] AC-2: Scanners run in CI and fail on new high-severity findings
- [ ] AC-3: Every Phase 1 work package touching tools, approvals or auth links to its threats

## Test plan

- Scanner jobs proven by a known-vulnerable test dependency in a throwaway branch
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
