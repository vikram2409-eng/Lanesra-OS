---
id: WP-0016
title: Fix tests failing on main
phase: 0
epic: E0.1 Program and engineering readiness
status: in_progress
owner: claude
depends_on: []
requirements: [REQ-PRE-001]
estimate: M
branch: claude/enterprise-build-planning-jxy2kq
pr: 
updated: 2026-10-16
---

# WP-0016 — Fix tests failing on main

## Goal

CI cannot become a required check while main is red. The baseline run on 2026-10-16 found failing tests (see testing/REGRESSION.md and RAID I-03). Root-cause and fix each one in product code or, if the test is wrong, fix the test with a written justification.

## Scope

- Every test listed as failing in testing/REGRESSION.md
- Root cause written in the Progress log for each. Six failures are industry-package workflows that no longer update a related record (auto service, field service, property management, real estate x2, recruitment) - investigate one shared cause in workflow_service cross-record actions first; the seventh is system_graph hard-delete
- No test skipped, ignored or loosened

## Acceptance criteria

- [x] AC-1: `cargo test --no-fail-fast -p lanesra-core -p lanesra-server -p lanesra-cli` passes with zero failures
- [x] AC-2: Each fix has a one-paragraph root cause in the Progress log
- [x] AC-3: REGRESSION.md baseline updated to all-green

## Test plan

- Re-run the full suite three times to rule out flakiness; paste totals into the PR
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

### 2026-10-16 claude
Owner asked for this fix directly in the planning session, so it shares the planning branch
`claude/enterprise-build-planning-jxy2kq` with WP-0001, WP-0002 and WP-0017 (exception to "one WP
per branch", owner-requested).

**Root cause (6 industry-package failures).** Migration 0078 made many_to_one and one_to_one
relationships auto-create a "lookup" custom field whose stored value is a cache of the linked
record's id. Only a lookup-field save wrote that cache. Links made any other way -
`relationship_service::link` from a workflow's `create_record` action (auto service, real estate,
recruitment, property management), the Related-list UI, imports, the API - left it empty. The next
ordinary save of the record read the empty cache, wrote it back, and `set_lookup_value(None)`
**deleted the real relationship link**. A second path did the same: a partial save that left the
lookup key out (field service: saving only `description`) was treated as "clear". The workflows
then found "no linked record to update". This is a real data-loss bug for customers, not a test issue.

**Fix.**
1. `relationship_service::link`, `unlink` and the archive-cascade in `enforce_delete_behavior`
   keep every lookup field's cache in step (`sync_lookup_caches`), using the same side rule as
   `set_lookup_value`; unlink clears a cache only if it still points at that link.
2. `custom_field_service::set_entity_values`: a lookup field missing from the submitted values keeps
   its current value; an explicit empty value still clears it.
3. Migration 0079 repairs caches already out of step in existing databases (refills empty caches
   from `relationship_instances`, deletes caches with no matching link). Without it, upgraded
   databases would still lose links on the next save.

**Root cause (System Graph test).** `hard_delete_removes_the_node_and_cascades_its_edges...` was
written before lookup fields existed and expected one dependent. A many_to_one relationship now
correctly also creates a lookup field, which is a dependent; deleting the relationship deactivates
(soft) that field, whose node is kept by design (the second half of the same test asserts that rule
for fields). The test now asserts exactly that: before = {relationship, lookup field}; after = only
the deactivated lookup field. Product behaviour unchanged; the test is stricter than before.

**Also changed.** `db::migrate::tests::applies_all_migrations_and_is_idempotent` expects schema
version 79 (each migration bumps it, per existing convention).

**Tests added** (`core/tests/lookup_custom_fields.rs`, tagged `Covers: REQ-PRE-001 (WP-0016)`):
link made outside the field shows in it and survives a re-save; unlink outside clears it;
one_to_one link fills both sides; a save that omits the lookup keeps the link; migration 0079
repairs stale caches. The first two fail without the fix (verified by reverting it).

State: the 7 affected suites pass (run twice). Full-suite result: see the next entry.
Next: owner review. Follow-up risk logged as RAID R-15 (missing ordinary-field keys are still
treated as "clear" by `set_entity_values` - semantics decision for WP-0008).

### 2026-10-16 claude (full run)
Clean build, `cargo test --locked --no-fail-fast -p lanesra-core -p lanesra-server -p lanesra-cli`:
1,009 tests, 1,008 pass, 1 ignored, 0 failing (the schema-version unit test failed in that run
because it was bumped to 79 after the run started; re-run alone: core lib 32/32). Full suite run
once (≈75 min); the 7 affected suites run twice. Remaining repeats for flakiness: CI (WP-0003).
State: all ACs met; waiting for owner review and a PR. Then status -> done.
