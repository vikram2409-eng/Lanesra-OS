---
id: WP-0187
title: "API: bulk export and bulk write behind bulk.read and bulk.write"
phase: 1
epic: E1.9 Team Workspace platform
status: in_review
owner: claude
depends_on: [WP-0184]
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

# WP-0187 — API: bulk export and bulk write behind bulk.read and bulk.write

## Goal

`bulk.read` and `bulk.write` can be granted and do nothing. Moving thousands of records one request at a time is the first thing an enterprise integration cannot do. Build bulk export and bulk write (ADR-0018 section 3).

## Scope

- `GET /api/v1/objects/:key/export?format=ndjson|csv`: stream every non-archived record of an object; `bulk.read`; CSV follows the existing RFC 4180 exporter. Streamed, so memory does not grow with the table
- `POST /api/v1/objects/:key/bulk`: body `{"operations":[{"op":"create|update|archive","id"?,"data"?}...]}`, at most 1000 operations; `bulk.write`; each operation goes through the same validated write path as single writes (validation, Business Rules, writable-object rules)
- Per-item results `{"results":[{"index","ok","id"?,"error"?}],"succeeded","failed"}`; a bad item does not stop the others. HTTP 200 when the request was processed, with failures listed in the body; 413 above 1000 operations
- Writable objects are the same four built-ins plus custom objects as the single-record API; other built-ins fail per item with the same validation error
- Rate limiting counts a bulk call as one request but the operation cap bounds the work

## Out of scope

- All-or-nothing transactions (a future opt-in flag)
- Reads of read-only built-ins beyond what `objects.read` already allows (export is allowed for every readable object)
- Async job-style bulk with polling

## Acceptance criteria

- [x] AC-1: Export streams all records of an object as NDJSON and as CSV; archived records are excluded; a client without `bulk.read` gets 403
- [x] AC-2: A bulk write of mixed create, update and archive operations applies each through the validated path and returns a result per item in order
- [x] AC-3: An invalid item (validation error, unknown id, read-only object) fails alone; the others still apply
- [x] AC-4: More than 1000 operations is refused with 413 and nothing is applied
- [x] AC-5: A client with `objects.write` but not `bulk.write` cannot call the bulk endpoint, and the reverse
- [x] AC-6: Business Rules that block a save also block that item in a bulk write
- [x] AC-7: Documented on the APIs page, the help article and in the CLI (`lanesra records export`, `lanesra records bulk`)

## Test plan

- `desktop/server/tests/api_v1.rs` (or a new `api_v1_bulk.rs`): AC-1 to AC-6 over real HTTP, including a streamed export of a few thousand records
- `desktop/cli/` tests for the two commands
- Tag every new test with `Covers: REQ-INT-006 (WP-0187 AC-n)` and `Covers: REQ-INT-005` on the allow/deny pairs
- Full regression suite green in CI; test count not lower than before

## Progress log

<!-- Append dated entries, newest last. -->
- 2026-10-21 claude: Written as `ready`. Not started.
- 2026-10-22 claude: Claimed. Building on `claude/enterprise-build-planning-jxy2kq`.
- 2026-10-22 claude: Built.
  - Core (`api_object_service`): `all_records` (unpaged, non-archived; `list_records` now pages over it), `bulk_write` with `MAX_BULK_OPERATIONS = 1000`, each operation in its own savepoint through the single-record create, update or archive path. Per-item results in request order. Unknown object is 404 for the whole call; a read-only built-in fails per item with the single-record error. `data_exchange_service`: `ExportFormat`, `export_columns`, `encode_export_chunk` (same RFC 4180 writer and column rule as `export_csv`).
  - Server: `GET /api/v1/objects/:key/export` (`bulk.read`) streams 500-record chunks (NDJSON default, CSV with a header row); `POST /api/v1/objects/:key/bulk` (`bulk.write`) with a 32 MB body limit, 413 before any write above 1000 operations.
  - **Found and fixed RAID I-32:** Business Rules (`block_save`) never ran on API writes. Every API create and update now runs in a savepoint and is refused and rolled back when a rule blocks the saved record. This is also what REST, MCP and CSV import now do (behaviour change, in the Releases entry).
  - Memory: the export's encoded output is streamed in chunks, but the record list itself is loaded from the entity services, as the records endpoint already does. A cursor-level stream needs per-object paging in every service; not done here.
  - CLI: `lanesra records export <object> [--format]` and `lanesra records bulk <object> --data <json|@file>`.
  - Docs: APIs page rows, help section "Bulk export and bulk write" (and the Business Rules note), llms files, Releases entry.
  - Tests: `server/tests/api_v1_bulk.rs` 7 (3,000-record export as NDJSON and CSV without the archived record, export scope, mixed write in order, failing items alone, 413 with nothing applied, scope separation both ways, Business Rule blocks the item and the single write); `cli/tests/cli_parsing.rs` +2. All suites on the API write path pass unchanged.
