---
id: WP-0189
title: "API: integration administration and the scope-vocabulary test"
phase: 1
epic: E1.9 Team Workspace platform
status: in_review
owner: claude
depends_on: [WP-0186, WP-0187, WP-0188]
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

# WP-0189 — API: integration administration and the scope-vocabulary test

## Goal

`admin.integration.read` and `admin.integration.manage` can be granted and do nothing. Operations teams need to see what is connected and to pause or run integrations from automation, without ever being able to read a secret. Build them, then lock the whole scope vocabulary with a test so a scope can never again exist without an endpoint behind it (REQ-INT-005, ADR-0018).

## Scope

- `admin.integration.read`: `GET /api/v1/integration/connections`, `connectors`, `jobs`, `api-clients` (the last without secrets or hashes). Metadata only: names, types, status, last run, counts; never a secret, ciphertext, token or base-URL credentials
- `admin.integration.manage`: pause and reactivate a connection or job, and run a job now (`POST .../jobs/:id/run`); it cannot create a connection, read a secret, change a scope or touch an API client
- Scope-vocabulary test: a test lists every scope the server will accept when creating an API client and, for each, names the route(s) that enforce it and proves one allow and one deny; the test fails if a scope is added without an enforcing route
- Remove nothing that clients already hold; all eleven scopes are now real

## Out of scope

- Creating or editing connections, connectors or API clients over the API
- A UI change beyond showing the scope descriptions

## Acceptance criteria

- [x] AC-1: With `admin.integration.read` a client lists connections, connectors, jobs and API clients, and no response contains a secret, ciphertext, token or hash (checked by a test that scans the JSON for every secret column's name and a known planted secret value)
- [x] AC-2: With `admin.integration.manage` a client can pause and reactivate a connection or job and run a job now; the job run appears in the integration log
- [x] AC-3: `admin.integration.manage` alone cannot read the listings, and `admin.integration.read` alone cannot pause or run (deny tests)
- [x] AC-4: The scope-vocabulary test passes: every grantable scope has an enforcing route with an allow and a deny test, and fails when a scope is added with none
- [x] AC-5: Documented on the APIs page and the help article with the final scope table; the Developers pages' "grant least privilege" guidance lists all eleven scopes

## Test plan

- `desktop/server/tests/api_v1_integration_admin.rs` (new): AC-1 to AC-3
- `desktop/server/tests/api_scope_vocabulary.rs` (new): AC-4
- Tag every new test with `Covers: REQ-INT-005 (WP-0189 AC-n)` and `Covers: REQ-INT-006`
- Full regression suite green in CI; test count not lower than before

## Notes

- Closes the loop on RAID I-31. When this merges, remove the "not enforced" caveats from the Developers pages and the help article.

## Progress log

<!-- Append dated entries, newest last. -->
- 2026-10-21 claude: Written as `ready`. Not started. Starts after WP-0186 to WP-0188.
- 2026-10-22 claude: Claimed.
- 2026-10-22 claude: Built.
  - Core `integration_admin_service`: metadata-only listings (`ApiConnectionSummary` without `config_json`, whose headers can hold tokens, and with the base URL stripped of user info, query and fragment; `ApiConnectorSummary`; jobs; API clients, whose model holds no key or hash), and `set_connection_paused`, `set_job_paused`, `run_job_now`, each audited under the API client (`api_client_id` in `details_json`, no user id).
  - **Pausing a connection is enforced**, so it is not another setting that does nothing: a new `paused` status is refused in `connection_service::resolve_secret`, which every outbound use passes through (connector actions, external objects and their jobs, MCP servers), and webhook fan-out skips webhooks on a paused connection. `active` and `disabled` keep their earlier meaning (a new connection defaults to `disabled` and works, so enforcing "must be active" would have broken existing setups). Jobs: `paused` was already honoured by the scheduler.
  - Server `api_integration_admin.rs`: four listings behind `admin.integration.read`; pause and reactivate for connections and jobs and `jobs/:id/run` (own connection, as it calls out) behind `admin.integration.manage`.
  - `server/tests/api_scope_vocabulary.rs` maps every entry of `VALID_SCOPES` to an enforcing route and proves allow (the scope alone is not refused) and deny (every other scope together is refused with 403). Verified it fails when a scope is added without a route: adding `reports.read` made it fail with "scopes with no enforcing route: [\"reports.read\"]" (reverted).
  - Docs: the APIs page scope table now lists all eleven scopes with the least-privilege callout updated; endpoint rows; help section "Integration administration and the scope table"; llms files; Releases entry. No "not enforced" caveat remained in the pages to remove.
  - Tests: `server/tests/api_v1_integration_admin.rs` 3 (listings scanned for planted secrets and secret column names; pause, reactivate and run with the run in the integration log and five audit rows; read and manage do not grant each other), `server/tests/api_scope_vocabulary.rs` 1, `core/tests/integration_webhook.rs` +1 (paused connection gets no delivery), `integration_admin_service` unit test +1 (URL credentials stripped). Connection-using and API suites pass unchanged.
