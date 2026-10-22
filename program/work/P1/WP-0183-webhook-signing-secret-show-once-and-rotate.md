---
id: WP-0183
title: "Webhook signing secret: show once and rotate"
phase: 1
epic: E1.9 Team Workspace platform
status: ready
owner: unassigned
depends_on: [WP-0182]
requirements: [REQ-INT-002]
threats: []
estimate: M
editions: [workspace, desktop, demo]
delivered: []
docs: [api-access-and-webhooks]
branch:
pr:
updated: 2026-10-21
---

# WP-0183 — Webhook signing secret: show once and rotate

## Goal

Every webhook delivery is signed with HMAC-SHA256, but the signing secret is generated at creation and never shown. The `Webhook` model exposes only `has_secret`, and no command or screen reveals the value. A receiver therefore has no way to verify a signature, which defeats the point of signing (RAID I-30). Give admins the secret exactly once, at creation, and a way to replace it.

## Scope

- `webhook_service::create` returns the generated signing secret in its result, once. Every later read (`get`, `list`) still returns `has_secret` only
- A new admin-only `rotate_secret` operation generates a new secret, stores it encrypted (same path as today), returns it once, and makes later deliveries use it
- Tauri command, server dispatch entry and frontend API wrapper for rotate; the Webhooks screen shows the secret once after create and after rotate, with a Copy button and a plain warning that it cannot be shown again, plus a **Rotate secret** action
- Rotation is written to the audit log (who, which webhook, when; never the value)
- Online demo: the same flow, simulated on sample data
- Help article, Integrations page and the Node.js verification snippet describe the real flow

## Out of scope

- Delivering record events at all: WP-0182. Managing webhooks over the REST API (including rotate): WP-0188, which reuses this WP's rotate operation
- A grace period where both old and new secrets verify
- Showing the secret again to an admin after creation (by design it is not retrievable)
- Secret rotation for API client secrets (already exists) or connection secrets

## Acceptance criteria

- [ ] AC-1: Creating a webhook returns its signing secret once; reading or listing it afterwards never returns the value, only `has_secret`
- [ ] AC-2: Rotating returns a new secret once; deliveries after rotation verify with the new secret and no longer verify with the old one
- [ ] AC-3: A non-admin cannot rotate (rejected, nothing changes); an unknown webhook id returns not-found
- [ ] AC-4: Rotation is recorded in the audit log without the secret value
- [ ] AC-5: The Webhooks screen (desktop UI and online demo) shows the secret once with Copy and a cannot-be-shown-again warning, and offers Rotate secret
- [ ] AC-6: The `api-access-and-webhooks` help article and the Integrations page show the verification steps against this flow

## Test plan

- `desktop/core/tests/integration_webhook.rs`: create returns the secret and a receiver verifies a delivery with it (AC-1); rotation invalidates the old secret (AC-2); non-admin and unknown-id negatives (AC-3); audit row exists and does not contain the value (AC-4)
- `desktop/server/tests/admin_actions.rs` or the dispatch test file: the rotate command is reachable and admin-gated over HTTP
- `desktop/src/` Vitest component test for the show-once panel (AC-5); online demo checked in the browser
- Tag every new test with `Covers: REQ-INT-002 (WP-0183 AC-n)`
- Full regression suite green in CI; test count not lower than before

## Notes

- Current code: `webhook_service.rs` generates the secret at create and stores it with `secret_service` (AES-256-GCM); `resolve_webhook_secret` decrypts it only to sign. The test helper `read_webhook_secret` in `integration_webhook.rs` shows no public path exists.
- No migration is expected (the secret is already stored encrypted). If one is needed, follow ADR-0010.
- Security: do not log the secret; do not put it in the delivery history or the audit row.
- Sequencing: WP-0182 first, so a receiver that verifies signatures also gets real events.

## Progress log

<!-- Append dated entries, newest last. -->
- 2026-10-21 claude: Written as `ready` (goal, scope, testable ACs, test files, dependency and requirements listed). Not started.
