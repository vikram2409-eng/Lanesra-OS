---
id: WP-0106
title: Team Workspace first-run and sign-in hardening
phase: 1
epic: E1.1 Enterprise sign-in
status: done
owner: claude
depends_on: [WP-0107, WP-0108, WP-0020]
requirements: [REQ-PRE-019]
threats: [T-11]
estimate: M
editions: [workspace, desktop]
editions_reason: Server-side protections; the desktop shares the throttling and session settings, and the online demo has no real sign-in
delivered: [workspace, desktop]
docs: [set-up-team-workspace]
branch: claude/enterprise-build-planning-jxy2kq
pr: "#735 (backend); admin UI follow-up"
updated: 2026-11-03
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

- [x] AC-1: Over HTTP, first-run setup without the token is refused, and with the token succeeds once; the token cannot be reused (server test)
- [x] AC-2: Repeated failed sign-ins are delayed, then locked out, and the lockout expires (test). Each failure, lockout and unlock is audited
- [x] AC-3: The public base URL is validated at start and exposed to the SSO work packages through one core function (test)
- [x] AC-4: An idle session expires after the configured timeout, and "Sign out all sessions" ends every session (test)
- [x] AC-5: The help article covers setup on a public address

## Test plan

- New `desktop/server/tests/first_run_and_sign_in_hardening.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy (a lockout is a `policy` error); ADR-0010 migration conventions. ADR-0010 applies to any migration this work package adds. Not behind a feature flag: this is a security fix (ADR-0012 §4).
- WP-0102 and WP-0103 depend on this work package, for the public base URL.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-03 claude: Refined proposed -> ready and claimed (deps WP-0107, WP-0108, WP-0020 all done). Scope confirmed against the 2026-10-16 Team Workspace review (RAID I-26, I-27) and ADR-0009 (lockout = policy error) / ADR-0012 §4 (security fix, no flag). Building: setup token, sign-in throttling, public base URL, configurable sessions + sign-out-all, help update.
- 2026-11-03 claude: Built the backend and proved every AC in the new `server/tests/first_run_and_sign_in_hardening.rs` (4 tests). Migration 0089 adds `security_settings` (lockout thresholds, session lifetime/idle, public base URL) and `sign_in_lockouts` (per-account and per-address brute-force state). Setup token: `setup_token` module + the HTTP first-run route requires it when a token file exists (startup writes one whenever there is no workspace); desktop and tests that never run startup are not gated. Throttling: `sign_in_throttle_service` with an injected clock, wired into `auth_service::login_at` (soft delays then a hard lockout, audited failure/lockout/unlock); `login` keeps its signature. Public URL: `security_settings_service::effective_public_base_url` is the one function the SSO WPs read, env override, validated at startup in `main.rs`. Sessions: `session_repo` now honours the configured absolute lifetime and idle timeout (`resolve_and_touch_at`), with `web_session_service` listing active sessions and signing out all. AC-5: the `set-up-team-workspace` help article now covers the setup token, `LANESRA_PUBLIC_URL`, and sign-in/session settings. Regressions re-run green (core user_management, migration_upgrade; server approvals_http, admin_actions, api_v1, api_network, mcp, feature_flags_http, api_scope_vocabulary). fmt/clippy/cargo-deny clean.
- 2026-11-03 claude: Remaining before done: the admin-facing surface — a Security settings screen (edit lockout thresholds, session lifetime/idle, public URL) and an active-sessions list with "sign out all" — and wiring the same `get_security_settings`/`update_security_settings`/`list_active_sessions`/`sign_out_all_sessions` commands into the desktop Tauri command layer so the capability is editable in the desktop edition too (the shared-core throttling and session enforcement already apply there). Kept `in_review` until that ships, so editions stay undelivered for now.
- 2026-11-04 claude: Backend merged in #735. Admin surface added here, completing the editions: desktop Tauri commands `get`/`update_security_settings`, `list_active_sessions`, `sign_out_all_sessions` (src-tauri, mirroring the server dispatch); `api.ts` methods + TS types; a `SecurityAdmin` screen under Admin -> Access -> "Security & sign-in" (edit lockout thresholds, session lifetime/idle and public URL; list active sessions and sign out all); 3 Vitest tests in `src/__tests__/securityAdmin.test.tsx`. Verified: `tsc --noEmit`, Vitest 62 passing, `npm run build`, and `cargo check`/`cargo clippy --all-targets -D warnings` on `lanesra-os` (installed the GTK/WebKit dev libs locally), `cargo fmt` clean. Delivered to Team Workspace and desktop; done.
