---
id: WP-0102
title: OIDC sign-in for Team Workspace
phase: 1
epic: E1.1 Enterprise sign-in
status: proposed
owner: unassigned
depends_on: [WP-0101, WP-0010, WP-0106]
requirements: [REQ-GOV-001]
threats: [T-11]
estimate: L
editions: [workspace]
delivered: []
editions_reason: server sign-in; desktop sign-in is WP-0105; the online demo has no real accounts
docs: [single-sign-on]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0102 — OIDC sign-in for Team Workspace

## Goal

Users sign in to the Team Workspace server with the customer's OpenID Connect provider.

## Scope

- Authorization code flow with PKCE, discovery, JWKS caching and rotation, nonce/state checks
- Admin configuration screen (issuer, client id, secret stored encrypted, scopes, claim mapping)
- Login page shows SSO option; local login can be disabled by admin except break-glass

## Acceptance criteria

- [ ] AC-1: Sign-in works against a local mock OIDC provider in tests
- [ ] AC-2: Each protocol check in ADR-0016 §2 (OIDC) is a negative test: invalid signature, wrong audience, expired token, replayed nonce, `none` algorithm, symmetric algorithm, unknown kid without JWKS refetch, issuer mismatch, clock skew >60s — all rejected
- [ ] AC-3: Client secret never returned by any API
- [ ] AC-4: Audit entries for sign-in success, failure and config change
- [ ] AC-5: SSO identity linked by (issuer, subject) pair, never by email alone (ADR-0016 §3); unverified email never links

## Test plan

- `desktop/server/tests/oidc.rs` with an in-process mock IdP; E2E sign-in spec
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
