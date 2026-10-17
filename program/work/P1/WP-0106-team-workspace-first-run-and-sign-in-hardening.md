---
id: WP-0106
title: Team Workspace first-run and sign-in hardening
phase: 1
epic: E1.1 Enterprise sign-in
status: proposed
owner: unassigned
depends_on: [WP-0107, WP-0108, WP-0020]
requirements: [REQ-PRE-019]
estimate: M
editions: [workspace, desktop]
editions_reason: Server-side protections; the desktop shares the throttling and session settings, and the online demo has no real sign-in
delivered: []
docs: [set-up-team-workspace]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0106 — Team Workspace first-run and sign-in hardening

## Goal

The Team Workspace review on 2026-10-16 found that a server reachable by others is not safe to expose yet:

- **Anyone can claim a fresh server (RAID I-26).** The first-run wizard creates the workspace and its Administrator for whoever reaches the server first (`workspace_service::first_run_setup`). On a public address, a stranger can take over a new install before its owner does.
- **No sign-in throttling (RAID I-27).** Failed sign-ins are audited (`auth_service.rs:41-57`), but nothing slows down password guessing. The server's rate limiter covers only API clients.
- **No public base URL.** OIDC and SAML (WP-0102, WP-0103) need the server's external URL for redirect and assertion endpoints. The server knows only its bind address.
- **Session lifetime is fixed.** Enterprise customers need to set the session lifetime and the idle timeout.

These protections come before enterprise sign-in, which builds on all four.

## Scope

- **Setup token.**
  - On first start with no workspace, the server generates a one-time setup token, prints it to the log, and writes it to `/data/setup-token`.
  - `first_run_setup` requires that token over HTTP.
  - The token is deleted after use.
  - Desktop first-run is unchanged: the local process is trusted.
- **Sign-in throttling.**
  - Exponential delay per account and per client address after failed attempts.
  - A temporary lockout after N failures. Defaults: 10 failures, 15 minutes; an admin can change both.
  - A clear message to the user and an audit entry for each event.
  - It applies to password sign-in and, later, to local fallback accounts under SSO.
- **Public base URL.** `LANESRA_PUBLIC_URL`, plus an admin setting. Validated at startup, and used for SSO redirects, links in emails, and CORS defaults. If HTTPS is expected and the URL is plain `http`, the server warns.
- **Sessions.** Configurable absolute lifetime and idle timeout. Admins can sign out all sessions and see active sessions.
- **Help.** Extend the `set-up-team-workspace` article with first-run, the setup token, and recommended settings for internet exposure.

## Acceptance criteria

- [ ] AC-1: Over HTTP, first-run setup without the token is refused, and with the token succeeds once; the token cannot be reused (server test)
- [ ] AC-2: Repeated failed sign-ins are delayed, then locked out, and the lockout expires (test). Each failure, lockout and unlock is audited
- [ ] AC-3: The public base URL is validated at start and exposed to the SSO work packages through one core function (test)
- [ ] AC-4: An idle session expires after the configured timeout, and "Sign out all sessions" ends every session (test)
- [ ] AC-5: The help article covers setup on a public address

## Test plan

- New `desktop/server/tests/first_run_and_sign_in_hardening.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy (a lockout is a `policy` error); ADR-0010 migration conventions. ADR-0010 applies to any migration this work package adds. Not behind a feature flag: this is a security fix (ADR-0012 §4).
- WP-0102 and WP-0103 depend on this work package, for the public base URL.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
