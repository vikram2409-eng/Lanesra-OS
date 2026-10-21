---
id: WP-0189
title: "API: integration administration and the scope-vocabulary test"
phase: 1
epic: E1.9 Team Workspace platform
status: ready
owner: unassigned
depends_on: [WP-0186, WP-0187, WP-0188]
requirements: [REQ-INT-006, REQ-INT-005]
threats: []
estimate: M
editions: [workspace]
editions_reason: The REST API exists only on the Team Workspace server; desktop opens no network port and the demo has no real API
delivered: []
docs: [api-access-and-webhooks]
branch:
pr:
updated: 2026-10-21
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

- [ ] AC-1: With `admin.integration.read` a client lists connections, connectors, jobs and API clients, and no response contains a secret, ciphertext, token or hash (checked by a test that scans the JSON for every secret column's name and a known planted secret value)
- [ ] AC-2: With `admin.integration.manage` a client can pause and reactivate a connection or job and run a job now; the job run appears in the integration log
- [ ] AC-3: `admin.integration.manage` alone cannot read the listings, and `admin.integration.read` alone cannot pause or run (deny tests)
- [ ] AC-4: The scope-vocabulary test passes: every grantable scope has an enforcing route with an allow and a deny test, and fails when a scope is added with none
- [ ] AC-5: Documented on the APIs page and the help article with the final scope table; the Developers pages' "grant least privilege" guidance lists all eleven scopes

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
