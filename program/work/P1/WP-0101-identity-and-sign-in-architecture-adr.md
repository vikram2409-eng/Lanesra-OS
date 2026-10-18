---
id: WP-0101
title: Identity and sign-in architecture ADR
phase: 1
epic: E1.1 Enterprise sign-in
status: in_review
owner: muse
depends_on: [WP-0009, WP-0107, WP-0108]
requirements: [REQ-GOV-001]
threats: [T-03, T-11, T-12, T-18]
estimate: M
editions: []
delivered: []
branch: wp/WP-0101-identity-adr
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/672
updated: 2026-10-18
---

# WP-0101 — Identity and sign-in architecture ADR

## ADR

ADR-0016 (Proposed): program/decisions/ADR-0016-identity-and-sign-in-architecture.md

## Goal

Decide how OIDC and SAML fit both editions - the owner requires SSO on desktop and on Team Workspace (ADR-0004): the single-user desktop app and the multi-user Team Workspace server (`server/src/session.rs`), and how directory identities map to users, roles and approvers.

## Scope

- Session model, token storage, logout and session revocation
- First-class providers: Okta, Microsoft Entra ID and Google (OIDC); Okta and Entra ID (SAML); Keycloak as the local test IdP
- Account linking for existing local users; break-glass local admin
- Desktop edition design (system browser + loopback, keychain storage) feeding WP-0105

## Acceptance criteria

- [x] AC-1: ADR accepted
- [ ] AC-2: WP-0102..0104 acceptance criteria updated from it

## Test plan

- Not a code change
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

- 2026-10-17 muse: Wrote ADR-0016 as Proposed (not Accepted). Covers Desktop, Team Workspace, Cloud SaaS. Follows ADR-0006 (principals) and ADR-0014 (one DB/key per workspace). Links T-03, T-11, T-12. No identity code. Awaiting owner and Claude review before WP-0107/WP-0108/WP-0109 build.

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

## Refinement log

- 2026-10-17 muse: Moved proposed → ready. Depends on WP-0107 (principal model) and WP-0108 (agent identities) for the ADR. AC-1 (ADR accepted) is manual; AC-2 (WP-0102..0104 ACs updated) is verifiable. REQ-GOV-001 listed. Size M. Note: owner decides who builds identity code (per 2026-10-17 instruction).
- 2026-10-18 claude: Reviewed ADR-0016 and revised it (still Proposed). Blocking fix: the draft stored accounts and sessions per workspace, contradicting ADR-0014 (control database holds accounts, memberships, sessions). Security fixes: account linking by (issuer, subject) with proof of ownership instead of email match; just-in-time creation opt-in and domain-limited with the lowest role; PKCE, state, algorithm allow-list and JWKS rotation for OIDC; SAML signed-assertion, signature-wrapping and replay checks; session id renewal, revocation on membership or role change; per-account plus per-address attempt limits; audit events; API-client principals. AC-1 waits for owner acceptance; AC-2 (update WP-0102..0104 criteria) follows acceptance.
- 2026-10-18 claude: AC-1: the owner accepted ADR-0016 on 2026-10-18 (as revised in PR #672). Owner decision the same day: Muse builds WP-0107, WP-0108 and WP-0109; Claude reviews every identity PR before merge. Remaining: AC-2, update WP-0102 to WP-0104 criteria from ADR-0016 sections 2 to 5.
