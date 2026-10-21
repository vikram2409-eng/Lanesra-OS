---
id: WP-0184
title: "API clients: enforce the allowed network and make every scope mean something"
phase: 1
epic: E1.9 Team Workspace platform
status: proposed
owner: unassigned
depends_on: []
requirements: [REQ-PRE-019, REQ-PRE-008]
threats: []
estimate: M
editions: [workspace]
editions_reason: The REST API and MCP endpoint exist only on the Team Workspace server; desktop opens no network port and the demo has no real API
delivered: []
docs: [api-access-and-webhooks]
branch:
pr:
updated: 2026-10-21
---

# WP-0184 — API clients: enforce the allowed network and make every scope mean something

## Goal

An API client can be created with an **Allowed CIDR** (for example `10.0.0.0/8`) and a set of scopes, and the Help article tells admins this restricts where the client can call from. Neither is fully true today (RAID I-31):

- `allowed_cidr` is stored (`api_client_service.rs:76`) but never read; `authorize()` in `server/src/api_v1.rs` ignores it, so a stolen key works from anywhere.
- Of the 11 scopes offered at creation, only `metadata.read`, `objects.read`, `objects.write`, `agents.trigger` and `events.read` are checked by any route. `search.read`, `bulk.read`, `bulk.write`, `webhooks.manage`, `admin.integration.read` and `admin.integration.manage` can be granted but do nothing.
- Code comments (`api_v1.rs:9-13`, `mcp.rs:4-6`) and the MCP settings copy say API and MCP calls get the "identical permission checks" as the UI. They do not: API-client record writes run with `actor = None` and skip Access Control (RAID I-20, owned by WP-0100 and WP-0109).

This is a proposed WP because two design questions belong to the owner (RAID Q-19, Q-20).

## Scope (proposed)

- Enforce `allowed_cidr` on every `/api/v1` and `/mcp` request. An empty value means unrestricted (today's behavior). A request from outside the range is refused with 403 and logged
- Decide the trusted client address source (direct peer address, or the forwarded header when `LANESRA_TRUST_PROXY_HTTPS` says a proxy is in front) and implement it once for both surfaces
- Make the scope vocabulary honest: each scope offered at client creation is enforced by at least one route, or it is removed from what can be granted. Existing clients holding a removed scope keep working and are shown a notice
- Correct the code comments and the MCP settings copy so they describe scope-based authorization accurately

## Out of scope

- Applying Access Control (roles, record scopes) to API-client writes: RAID I-20, WP-0100 / WP-0109
- New API routes for search, bulk or webhook management (only if the owner chooses "implement" in Q-20, as a separate WP)
- Per-client rate-limit configuration changes

## Acceptance criteria

- [ ] AC-1: A client with `allowed_cidr` set is refused (403) from an address outside the range on REST and MCP, and accepted from inside it
- [ ] AC-2: A client with no `allowed_cidr` behaves exactly as today
- [ ] AC-3: The client address is taken only from the source Q-19 decides; a spoofed forwarded header from an untrusted peer does not bypass the restriction (negative test)
- [ ] AC-4: Every scope that can be granted is enforced by a route, or is no longer grantable; a test lists the scope vocabulary and the enforcing route for each
- [ ] AC-5: A malformed CIDR is rejected when the client is created or edited, not silently ignored
- [ ] AC-6: The `api-access-and-webhooks` help article, the Developers APIs page and the MCP settings copy match the behavior

## Test plan

- `desktop/server/tests/api_v1.rs` and `desktop/server/tests/mcp.rs`: allowed and refused addresses, spoofed header, empty CIDR (AC-1 to AC-3)
- `desktop/core/tests/integration_api_client.rs`: CIDR validation on create and update (AC-5); scope vocabulary table test (AC-4)
- Tag every new test with `Covers: REQ-PRE-019 (WP-0184 AC-n)`
- Full regression suite green in CI; test count not lower than before

## Notes

- Blocked for refinement on RAID Q-19 (client address source behind a proxy) and Q-20 (remove or implement the unenforced scopes). Once answered, set `ready`.
- Threat to link when refined: unauthorized use of a leaked API key (extend THREAT-MODEL.md if no entry fits).
- The Developers pages shipped with PR #688 deliberately do not claim network restriction or the unenforced scopes.

## Progress log

<!-- Append dated entries, newest last. -->
- 2026-10-21 claude: Proposed from the Developers-pages code review. Not started; waiting for Q-19 and Q-20.
