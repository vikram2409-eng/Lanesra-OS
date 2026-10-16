---
id: WP-0105
title: Enterprise sign-in on the desktop edition
phase: 1
epic: E1.1 Enterprise sign-in
status: proposed
owner: unassigned
depends_on: [WP-0102, WP-0103]
requirements: [REQ-GOV-028]
estimate: L
editions: [desktop]
delivered: []
editions_reason: desktop-edition sign-in
docs: [single-sign-on]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0105 — Enterprise sign-in on the desktop edition

## Goal

The owner decided SSO must work in both editions (ADR-0004). Bring OIDC and SAML sign-in to the
Tauri desktop app, reusing the server's validation code in `core`.

## Scope

- OIDC in the system browser with PKCE and a loopback redirect (RFC 8252), never an embedded webview login
- SAML through a short-lived loopback listener as the assertion consumer
- Token and session storage in the OS keychain; offline grace period and re-authentication rules
- Provider presets and test notes for Okta, Microsoft Entra ID and Google (OIDC); Entra ID and Okta (SAML)
- Desktop keeps a local admin login for offline use, governed by an admin setting

## Acceptance criteria

- [ ] AC-1: Desktop sign-in works end to end against the same mock IdPs used by WP-0102 and WP-0103
- [ ] AC-2: The loopback listener binds to 127.0.0.1 only, uses a random port, accepts exactly one response and validates state and PKCE
- [ ] AC-3: Tokens are never written to the SQLite database or logs (scan test)
- [ ] AC-4: Provider presets documented with the exact console settings for each IdP

## Test plan

- `desktop/core/tests/sso_shared_validation.rs` (validation shared by server and desktop)
- Tauri command tests for the loopback flow with a mock IdP
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review together with WP-0101.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
