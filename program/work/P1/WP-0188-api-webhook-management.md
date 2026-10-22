---
id: WP-0188
title: "API: webhook management behind the webhooks.manage scope"
phase: 1
epic: E1.9 Team Workspace platform
status: in_review
owner: claude
depends_on: [WP-0183, WP-0184]
requirements: [REQ-INT-006, REQ-INT-005]
threats: []
estimate: M
editions: [workspace]
editions_reason: The REST API exists only on the Team Workspace server; desktop opens no network port and the demo has no real API
delivered: [workspace]
docs: [api-access-and-webhooks]
branch: claude/enterprise-build-planning-jxy2kq
pr: TBD
updated: 2026-10-22
---

# WP-0188 — API: webhook management behind the webhooks.manage scope

## Goal

`webhooks.manage` can be granted and does nothing, so webhooks can only be managed in the admin UI. Teams that provision integrations as code need to create, pause, rotate and inspect webhooks over the API (ADR-0018 section 3). Reuses the secret show-once and rotate operation from WP-0183.

## Scope

- `GET/POST /api/v1/webhooks`, `GET/PATCH/DELETE /api/v1/webhooks/:id`, `POST /api/v1/webhooks/:id/pause`, `reactivate`, `rotate-secret`, `test`, and `GET /api/v1/webhooks/:id/deliveries`; scope `webhooks.manage`
- Create returns the signing secret once (WP-0183 AC-1); `rotate-secret` returns the new one once; no read ever returns the value, only `has_secret`
- The endpoint URL still comes from a webhook-type connection id and passes the SSRF guard; the API cannot point a webhook at an arbitrary URL
- Calls are made as the API client, not as an administrator: the same service functions with the admin check satisfied by the scope, and rotation and creation audited with the client id

## Out of scope

- Creating or editing the connection itself (secrets): not available over the API (ADR-0018)
- Per-webhook filters

## Acceptance criteria

- [x] AC-1: With `webhooks.manage` a client can create, list, read, update, pause, reactivate, test, rotate and delete a webhook; the secret is returned only by create and rotate
- [x] AC-2: Without `webhooks.manage`, every webhook endpoint returns 403, even with `objects.write` or `admin.integration.manage`
- [x] AC-3: A webhook cannot be created against a connection that is not webhook-typed or whose URL the SSRF guard refuses
- [x] AC-4: Rotation and creation through the API are written to the audit log with the API client id and no secret value
- [x] AC-5: The delivery history endpoint pages through attempts and never includes the signing secret
- [x] AC-6: Documented on the APIs page, the help article and the CLI (`lanesra webhooks ...`)

## Test plan

- `desktop/server/tests/api_v1_webhooks.rs` (new): AC-1 to AC-5 over real HTTP, with a local receiver proving a rotated secret verifies and the old one does not
- `desktop/cli/` tests for the commands
- Tag every new test with `Covers: REQ-INT-006 (WP-0188 AC-n)` and `Covers: REQ-INT-005` on the allow/deny pair
- Full regression suite green in CI; test count not lower than before

## Progress log

<!-- Append dated entries, newest last. -->
- 2026-10-21 claude: Written as `ready`. Not started. Starts after WP-0183 and WP-0184.
- 2026-10-22 claude: Claimed. WP-0183 (dependency) is merged but still `in_review` pending WP-0182 AC-5; its show-once and rotate code is on main, which is all this needs.
- 2026-10-22 claude: Built.
  - `webhook_service`: a `WebhookManager` (a signed-in Administrator or an API client) decides who may act and who is audited; the existing admin functions delegate to new `*_by` functions (`create_with_secret_by`, `update_by`, `set_status_by`, `rotate_secret_by`, `delete_by`, `test_delivery_by`), plus `get` and `list_deliveries_page`. An API client is authorised by its `webhooks.manage` scope (checked by the server), never by borrowing an Administrator; its changes are audited with `api_client_id` in `details_json` and no user id. Creation, update and deletion are now audited for everyone (only rotation was before).
  - `validate_input` (create and update): the connection must be `webhook`-typed and its URL must pass `ssrf_guard::check_url`. This also applies to the admin screen: a webhook whose URL the server may not reach is refused when saved instead of failing every delivery (Releases entry).
  - Repo: `update` and `list_deliveries_page` (with total).
  - Server `api_webhooks.rs`: `GET/POST /api/v1/webhooks`, `GET/PATCH/DELETE /api/v1/webhooks/:id`, `POST .../pause`, `reactivate`, `rotate-secret`, `test` (own connection, as it calls out), `GET .../deliveries` (paged); all through `api_v1::authorize` with `webhooks.manage` and logged.
  - CLI `lanesra webhooks list|get|create|update|delete|pause|reactivate|rotate-secret|test|deliveries`.
  - Docs: APIs page rows, help section "Managing webhooks over the API", llms files, Releases entry.
  - Tests: `server/tests/api_v1_webhooks.rs` 4 (full lifecycle with a local receiver verifying the created secret, then only the rotated one; paged deliveries without secrets; 403 on all ten endpoints with `objects.write` and `admin.integration.manage`; non-webhook connection 400 and metadata-address URL 403; audit rows carry the client id and no secret); `cli/tests/cli_parsing.rs` +1. All webhook suites pass unchanged (integration_webhook, webhook_secret, job_scheduler, integration_connection, integration_api_client, agent_v1, trigger_identities, principals, ai_agent_orchestration).
