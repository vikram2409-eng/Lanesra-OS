---
id: WP-0101
title: Identity and sign-in architecture ADR
phase: 1
epic: E1.1 Enterprise sign-in
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0009]
requirements: [REQ-GOV-001]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0101 — Identity and sign-in architecture ADR

## Goal

Decide how OIDC and SAML fit both editions - the owner requires SSO on desktop and on Team Workspace (ADR-0004): the single-user desktop app and the multi-user Team Workspace server (`server/src/session.rs`), and how directory identities map to users, roles and approvers.

## Scope

- Session model, token storage, logout and session revocation
- First-class providers: Okta, Microsoft Entra ID and Google (OIDC); Okta and Entra ID (SAML); Keycloak as the local test IdP
- Account linking for existing local users; break-glass local admin
- Desktop edition design (system browser + loopback, keychain storage) feeding WP-0105

## Acceptance criteria

- [ ] AC-1: ADR accepted
- [ ] AC-2: WP-0102..0104 acceptance criteria updated from it

## Test plan

- Not a code change
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
