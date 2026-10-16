---
id: WP-0103
title: SAML 2.0 sign-in for Team Workspace
phase: 1
epic: E1.1 Enterprise sign-in
status: proposed
owner: unassigned
depends_on: [WP-0101, WP-0010]
requirements: [REQ-GOV-001]
estimate: L
editions: [workspace]
delivered: []
editions_reason: server sign-in; desktop sign-in is WP-0105; the online demo has no real accounts
docs: [single-sign-on]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0103 — SAML 2.0 sign-in for Team Workspace

## Goal

Users sign in with the customer's SAML 2.0 IdP.

## Scope

- SP metadata endpoint, IdP metadata import, signed assertion validation, clock skew, audience and recipient checks, replay cache
- Attribute mapping to user fields

## Acceptance criteria

- [ ] AC-1: Valid signed assertion signs in; unsigned, wrong-audience, expired and replayed assertions are rejected
- [ ] AC-2: XML signature wrapping attacks covered by test vectors
- [ ] AC-3: Audit entries as for OIDC

## Test plan

- `desktop/server/tests/saml.rs` with fixture assertions
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.
- Use a maintained SAML library; record it in THIRD_PARTY_NOTICES. Never hand-roll XML-DSig.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
