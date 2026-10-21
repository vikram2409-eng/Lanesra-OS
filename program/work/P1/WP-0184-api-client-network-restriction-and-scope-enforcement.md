---
id: WP-0184
title: "API clients: enforce the allowed network with a trusted-proxy rule"
phase: 1
epic: E1.9 Team Workspace platform
status: in_progress
owner: claude
depends_on: []
requirements: [REQ-INT-004, REQ-PRE-019]
threats: [T-20]
estimate: M
editions: [workspace]
editions_reason: The REST API and MCP endpoint exist only on the Team Workspace server; desktop opens no network port and the demo has no real API
delivered: []
docs: [api-access-and-webhooks, set-up-team-workspace]
branch: wp/WP-0184-api-client-network-restriction
pr:
updated: 2026-10-21
---

# WP-0184 — API clients: enforce the allowed network with a trusted-proxy rule

## Goal

An API client can be created with an **Allowed CIDR**, and the Help article tells admins this restricts where the client can call from. It does not: `allowed_cidr` is stored (`api_client_service.rs:76`) and never read, so a leaked key works from anywhere (RAID I-31). Enforce it, using the caller-address rule the owner approved in ADR-0018 (the connection address, or the forwarded address only through an operator-configured trusted proxy).

## Scope

- Parse `LANESRA_TRUSTED_PROXIES` (comma-separated IPs and CIDRs, default empty) at startup; a malformed entry stops the server naming the entry
- One resolver for the caller address, exactly as ADR-0018 section 1: peer address; `X-Forwarded-For` read only when the peer is a trusted proxy, walked right to left skipping trusted proxies; no other header is read; IPv4-mapped IPv6 normalised
- `allowed_cidr` becomes a comma- or whitespace-separated list of CIDRs and single addresses; empty means unrestricted; no migration
- Enforce it in `authorize()` (`server/src/api_v1.rs`), after the key verifies and before the scope check, so REST, MCP, the agent trigger routes and the event stream are all covered. Refusal: 403 `This API client is not allowed from this network address`, logged to the integration log with the resolved address
- Validate the list when a client is created or edited (Tauri command, server dispatch, UI form); show the field with an example and the proxy note
- Add `LANESRA_TRUSTED_PROXIES` to the Team Workspace settings in `desktop/README.md`, `docker-compose.yml` comments and the website settings table

## Out of scope

- Which scopes are enforced and the new endpoints: WP-0186 to WP-0189
- Access Control for API-client record writes (RAID I-20): WP-0100 / WP-0109
- Per-client rate limits

## Acceptance criteria

- [x] AC-1: A client with `allowed_cidr` is accepted from an address inside the range and refused (403) from one outside it, on `/api/v1` records, `/mcp` and `/api/v1/events/stream`
- [x] AC-2: A client with an empty `allowed_cidr` behaves exactly as today
- [x] AC-3: With no trusted proxy configured, a forged `X-Forwarded-For` (and `Forwarded`, `X-Real-IP`) is ignored: a restricted client cannot be admitted by spoofing a header
- [x] AC-4: With the peer inside `LANESRA_TRUSTED_PROXIES`, the caller is the right-most address that is not itself a trusted proxy; extra left-hand entries an attacker prepended are ignored; a missing, malformed or all-trusted header falls back to the peer address
- [x] AC-5: A list with several entries, single addresses, IPv6 and IPv4-mapped IPv6 all match correctly
- [x] AC-6: A malformed `allowed_cidr` is rejected on create (API clients have no edit operation today, so create is the only write); a malformed `LANESRA_TRUSTED_PROXIES` entry stops startup naming it
- [x] AC-7: A refusal does not reveal whether a key is valid to an unauthenticated caller (a bad key returns the same 401 whatever the address), and is recorded in the integration log with the resolved address
- [x] AC-8: The `api-access-and-webhooks` and `set-up-team-workspace` help articles, the Developers pages and `desktop/README.md` describe the behavior and the new setting

## Test plan

- `desktop/core/tests/network_trust.rs` (new, pure unit-level): resolver truth table for AC-3, AC-4, AC-5; list parsing for AC-5, AC-6
- `desktop/server/tests/api_v1.rs` and `mcp.rs`: real HTTP requests with a fixed peer address and chosen headers, allowed and refused, for REST, MCP and the stream (AC-1, AC-2, AC-7)
- `desktop/core/tests/integration_api_client.rs`: validation on create and update (AC-6)
- Tag every new test with `Covers: REQ-INT-004 (WP-0184 AC-n)`
- Full regression suite green in CI; test count not lower than before

## Notes

- Design is ADR-0018 (accepted). Follow it exactly; two agents must produce the same resolver.
- Server tests need control of the peer address. The server already runs on a real local socket, so peer is 127.0.0.1; add the loopback to `LANESRA_TRUSTED_PROXIES` in the test to exercise the forwarded path, and use an allowed list that excludes 127.0.0.1 for the refusal path.
- Security: this changes authentication behavior. Link the threat "leaked API key used from outside the intended network" at refinement; extend `security/THREAT-MODEL.md` if no entry fits.

## Progress log

<!-- Append dated entries, newest last. -->
- 2026-10-21 claude: Refined to `ready` per ADR-0018 (owner answered Q-19). The scope work that was originally part of this WP moved to WP-0186 to WP-0189. Not started.
- 2026-10-21 claude: Claimed. Branched from the WP-0182 branch (for the planning files); no code dependency on WP-0182 or WP-0183. Design: a pure `network_trust` module in core (resolver and list matching, reusing `ssrf_guard::AllowEntry`), `SecurityConfig.trusted_proxies`, a `Caller` extractor in the server that resolves the address once per request, and the check in `authorize()`. No API-client edit operation exists, so AC-6 validation is on create. Next: failing tests first.
- 2026-10-21 claude: Built. Core `network_trust` (resolver, list parsing, `is_restricted`, `client_admits`, `validate_allowed_cidr`), `SecurityConfig.trusted_proxies` read from `LANESRA_TRUSTED_PROXIES` (a malformed entry panics at startup naming it), `Caller` extractor, check in `authorize()` between key verification and the scope check, refusal logged as `network_blocked` with the resolved address. If the server is not given the peer address (served without connect info) a restricted client is refused and unrestricted ones still work; `main.rs` serves with connect info. Create validates the list; there is no edit operation for API clients. Added threat T-20. Tests: 9 core `network_trust`, 1 `integration_api_client`, 7 server `api_network` (loopback is the peer; loopback is listed as a trusted proxy to drive the forwarded path). The create test was seen failing with validation removed. Docs: help articles, Developers pages, README, docker-compose, llms, Releases. Server suite (11 binaries) green with `--features lanesra-core/test-support`.
