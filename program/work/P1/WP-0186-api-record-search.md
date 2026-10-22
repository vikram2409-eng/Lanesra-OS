---
id: WP-0186
title: "API: record search behind the search.read scope"
phase: 1
epic: E1.9 Team Workspace platform
status: done
owner: claude
depends_on: [WP-0184]
requirements: [REQ-INT-006, REQ-INT-005]
threats: []
estimate: S
editions: [workspace]
editions_reason: The REST API exists only on the Team Workspace server; desktop opens no network port and the demo has no real API
delivered: [workspace]
docs: [api-access-and-webhooks]
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/700
updated: 2026-10-22
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

- [x] AC-1: A client with `search.read` finds a record by a word in its name across more than one object type; an `object` filter narrows it
- [x] AC-2: A client without `search.read` gets 403, even if it holds `objects.read`
- [x] AC-3: Archived records and objects the client cannot read are not returned
- [x] AC-4: Paging works and `page_size` is clamped to 1 to 500; an empty `q` returns a validation error (400)
- [x] AC-5: The endpoint is listed in the APIs page, the help article and the CLI help

## Test plan

- `desktop/server/tests/api_v1.rs`: allow and deny tests for AC-1 to AC-4
- `desktop/cli/` test for `lanesra search`
- Tag every new test with `Covers: REQ-INT-006 (WP-0186 AC-n)` and `Covers: REQ-INT-005` on the allow/deny pair
- Full regression suite green in CI; test count not lower than before

## Progress log

<!-- Append dated entries, newest last. -->
- 2026-10-21 claude: Written as `ready`. Not started.
- 2026-10-22 claude: Built.
  - `search_service`: the existing global search gains a shared core with a result cap and an optional object filter (`search_records`); `global_search` keeps its 25-result behaviour, unchanged for the desktop and the admin search.
  - `api_object_service::search_records`: searches every object the REST API can read (readable built-ins and active Custom Objects, the same list as `GET /api/v1/objects`), drops anything else (for example a searchable custom field on a non-API object), removes duplicates, pages like the records endpoint (default 50, clamp 1 to 500, overflow-safe offset). `total` counts at most 10,000 matches (`SEARCH_RESULT_CAP`). Empty `q` is 400; an unknown or inactive `object` is 404.
  - `GET /api/v1/search` behind `search.read` through `authorize()` (so the WP-0184 network rule applies) and `logged()`; CLI `lanesra search <text> [--object] [--page] [--page-size]`.
  - Docs: APIs page table row, help article `api-access-and-webhooks` section "Searching records" (`wps` names WP-0186), llms files, Releases v0.97.0 entry.
  - Tests: `server/tests/api_v1.rs` +4 (AC-1 across Company and Contact with the object filter and an unknown object, AC-2 403 with `objects.read`, AC-3 archived, AC-4 paging, clamping, huge page and empty `q`); `core/tests/integration_api_object.rs` +1 (AC-1 and AC-3: a Custom Object's records are found until the object is deactivated); `cli/tests/cli_parsing.rs` +1 (AC-5). The existing search suites (`search`, `admin_search`, `ai_context_layer`, `ai_vector_search`, `voice_mode_v4`) pass unchanged; clippy `-D warnings` clean.
  - Shipped in PR #700 together with WP-0109, at the owner's choice (2026-10-22).
- 2026-10-22 claude: PR #700 merged by the owner (cfc4a38). Delivered on Team Workspace, its only edition; help article updated. Done.
