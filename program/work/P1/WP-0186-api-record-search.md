---
id: WP-0186
title: "API: record search behind the search.read scope"
phase: 1
epic: E1.9 Team Workspace platform
status: ready
owner: unassigned
depends_on: [WP-0184]
requirements: [REQ-INT-006, REQ-INT-005]
threats: []
estimate: S
editions: [workspace]
editions_reason: The REST API exists only on the Team Workspace server; desktop opens no network port and the demo has no real API
delivered: []
docs: [api-access-and-webhooks]
branch:
pr:
updated: 2026-10-21
---

# WP-0186 — API: record search behind the search.read scope

## Goal

`search.read` can be granted to an API client and does nothing. An enterprise integration needs to find records by text without knowing which object holds them. Build `GET /api/v1/search` (ADR-0018 section 3).

## Scope

- `GET /api/v1/search?q=<text>&object=<key>&page=&page_size=` searching the objects the client may read (the readable built-ins and active custom objects), reusing the product's existing global-search service, not a second implementation
- Response: `{"ok":true,"data":{"results":[{"object_key","id","title","snippet"}],"total","page","page_size"}}`, same paging limits as the records endpoints (default 50, maximum 500)
- Requires `search.read`; the network rule from WP-0184 applies through `authorize()`; every call logged
- A CLI command `lanesra search <text>`; MCP tool is out of scope

## Out of scope

- Field-level or boolean query syntax; ranking changes
- MCP tool for search

## Acceptance criteria

- [ ] AC-1: A client with `search.read` finds a record by a word in its name across more than one object type; an `object` filter narrows it
- [ ] AC-2: A client without `search.read` gets 403, even if it holds `objects.read`
- [ ] AC-3: Archived records and objects the client cannot read are not returned
- [ ] AC-4: Paging works and `page_size` is clamped to 1 to 500; an empty `q` returns a validation error (400)
- [ ] AC-5: The endpoint is listed in the APIs page, the help article and the CLI help

## Test plan

- `desktop/server/tests/api_v1.rs`: allow and deny tests for AC-1 to AC-4
- `desktop/cli/` test for `lanesra search`
- Tag every new test with `Covers: REQ-INT-006 (WP-0186 AC-n)` and `Covers: REQ-INT-005` on the allow/deny pair
- Full regression suite green in CI; test count not lower than before

## Progress log

<!-- Append dated entries, newest last. -->
- 2026-10-21 claude: Written as `ready`. Not started.
