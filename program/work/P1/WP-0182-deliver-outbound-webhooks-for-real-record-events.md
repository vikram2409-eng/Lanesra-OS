---
id: WP-0182
title: "Deliver outbound webhooks for real record events"
phase: 1
epic: E1.9 Team Workspace platform
status: in_progress
owner: claude
depends_on: []
requirements: [REQ-PRE-008, REQ-PRE-015]
threats: []
estimate: S
editions: [workspace, desktop]
editions_reason: Outbound webhooks are an Integration Hub feature of the Team Workspace server and the desktop app; the online demo has no real network egress or receivers
delivered: []
docs: [api-access-and-webhooks]
branch: wp/WP-0182-deliver-outbound-webhooks
pr:
updated: 2026-10-21
---

# WP-0182 — Deliver outbound webhooks for real record events

## Goal

A webhook an admin creates in the Integration Hub must actually fire when a subscribed record is created, updated or archived. Today it does not: `event_hooks::emit` queues each event in `integration_pending_events`, and `webhook_service::drain_pending_events` (the only code that signs, sends and clears them) has no caller outside tests. Only the admin **Test** button delivers, because it bypasses the queue. The help article and the Developers pages describe webhooks as live, so this is a gap between what is promised and what runs (RAID I-29).

## Scope

- Call `webhook_service::drain_pending_events` from the Team Workspace scheduler tick (`server/src/job_scheduler.rs`), next to the other `drain_*` calls, so each queued event is delivered within one tick (60 seconds in production).
- Call it from desktop's `drain_pending_async_work` (`src-tauri/src/commands/ai_agent_pipeline_commands.rs`), so a desktop workspace that has webhooks does not accumulate queued events forever.
- Make `drain_pending_events` robust: an event whose fan-out returns an error must not stop later events in the same pass. A failed event stays queued for the next pass.
- Update the webhook help article and the Integrations page so they state exactly when events are delivered and which event types are emitted.

## Out of scope

- Event types that are accepted as subscriptions but never emitted (`field.changed`, `workflow.completed`, `workflow.failed`): emit them or stop offering them is an owner decision (RAID Q-19).
- Showing or rotating the signing secret so a receiver can verify signatures: WP-0183 (RAID I-30).
- Per-event attempt counts and dead-lettering of an event that can never be delivered (a possible follow-up; see Notes).
- Honoring the stored but unused `retry_policy_json` and `filter_json` columns.

## Acceptance criteria

- [x] AC-1: On a running Team Workspace scheduler, creating, updating and archiving a record that an active webhook subscribes to each results in one delivery to the receiver within a tick, with nobody calling Test or `drain_pending_events` by hand
- [x] AC-2: The delivery carries the documented payload fields (`event_id`, `event_type`, `occurred_at`, `workspace_id`, `object_key`, `record_id`, `correlation_id`, `data`) and a valid `X-Lanesra-Signature: sha256=<hex>` over the raw body, and appears in the webhook's delivery history
- [x] AC-3: Deny and limit paths: a paused webhook, a deleted webhook and a webhook scoped to a different object each receive nothing, and the queued event row is cleared in every case
- [x] AC-4: One pending event whose fan-out errors does not stop the events behind it in the same pass; the failed event stays queued and the others are delivered
- [ ] AC-5: Desktop's `drain_pending_async_work` also drains pending webhook events (a one-line mirror of the server call)
- [ ] AC-6: The `api-access-and-webhooks` help article, the Integrations page, `llms-full.txt` and the Releases page state that record events (`record.created`, `record.updated`, `record.archived`) are delivered by the Team Workspace scheduler and the desktop poll

## Test plan

- `desktop/server/tests/job_scheduler.rs`: a new test starts the real scheduler thread against an on-disk database and a local HTTP receiver, creates, updates and archives a Company, and asserts three signed deliveries arrive with no manual drain (AC-1, AC-2); a paused webhook and an other-object scope receive nothing (AC-3)
- `desktop/core/tests/integration_webhook.rs`: a new test queues two events, makes the first one's fan-out error, and asserts the second is still delivered and the first stays queued (AC-4)
- AC-5 has no automated test: `src-tauri` has no test harness and needs a window system to build. The `tauri (desktop shell)` CI job compiles the change, and the code is a line-for-line mirror of the server call. Recorded as a known limit, not a skipped test
- AC-6: `node --check app.js`, `python3 program/tools/check_binary_assets.py`, the website render check for the Integrations page
- Tag every new test with `Covers: REQ-PRE-008 (WP-0182 AC-n)` so `programctl trace` finds it
- Full regression suite green in CI; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Found 2026-10-21 while writing the Developers pages; `rg drain_pending_events` shows no production caller. See RAID I-29.
- No migration: `integration_pending_events` already exists (migration 0032).
- Follow ADR-0009 (error taxonomy) for any new error. The Team Workspace SSRF guard (WP-0143 AC-4) already applies to webhook endpoint URLs.
- Known limit to record, not fix here: `list_batch` returns the oldest 100 rows, so more than 100 permanently failing rows could starve newer events. Dead-lettering needs an attempts column and so a migration; propose it as a follow-up WP if it ever matters.
- Requirement IDs: no requirement names outbound webhook delivery. `REQ-PRE-008` (discrepancies between documented and actual behavior are resolved) and `REQ-PRE-015` (edition parity) are the closest. A proper requirement is proposed in RAID Q-18.

## Progress log

<!-- Append dated entries, newest last. This is the handoff point for the next agent. -->
- 2026-10-21 claude: Claimed. Researched the gap: `event_hooks::emit` queues, `drain_pending_events` has no production caller, the server tick drains five other queues and desktop's `drain_pending_async_work` drains four. Next: write the failing scheduler test first (AC-1), then add the call.
- 2026-10-21 claude: Built AC-1 to AC-4, tests first.
  Done: AC-1, AC-2, AC-3 in `desktop/server/tests/job_scheduler.rs` (two new tests: a real scheduler thread delivers signed `record.created/updated/archived` webhooks with nobody calling the drain; paused, other-object and deleted cases receive nothing and the queue clears). AC-4 in `desktop/core/tests/integration_webhook.rs` (`one_pending_event_that_errors_does_not_block_the_events_behind_it`). Both server tests and the core test were seen failing for the right reason before the change (timed out waiting for delivery; `drain_pending_events` aborted on the first error).
  Code: `webhook_service::drain_pending_events_report` (new, reports `delivered` and `failed`, leaves failed events queued); `drain_pending_events` is now a thin wrapper returning the delivered count, so no existing test changed. Server tick logs failures. Desktop `drain_pending_async_work` calls the drain (AC-5 code is in, no automated test is possible for it; the `tauri (desktop shell)` CI job compiles it; left unticked until CI is green and the owner accepts that limit). `desktop/server/Cargo.toml` gains dev-dependencies `hmac`, `sha2`, `hex` (already in `Cargo.lock` via core; lockfile updated).
  State: green locally: `cargo test -p lanesra-core --test integration_webhook` (9 passed), `cargo test -p lanesra-server --test job_scheduler` (3 passed), `cargo clippy --locked ... -D warnings` clean, `cargo fmt` clean. Full suite not run locally (tiered testing, PROTOCOL §5); CI runs it.
  Next: (1) AC-6 docs once PR #688 (Developers pages) merges, because #688 edits the same one-line Help article section: update the `api-access-and-webhooks` article, the Integrations page (`site-modern-pages.js`), `llms-full.txt`; add the Releases entry for whichever version ships it; (2) record test totals in `program/testing/REGRESSION.md`; (3) `programctl new-log`; (4) mark ready for review after CI is green.
  Gotchas: `COMPANY_STATUSES` is `Prospect`, `Active Customer`, `Inactive`, `Archived`. Server tests need a webhook created with the key the scheduler will resolve (`secret_service::resolve_master_key(&key_file_path)`). `list_batch` returns the oldest 100 rows, so more than 100 permanently failing events could starve newer ones (recorded in Notes).
  Commands: `cargo test -p lanesra-core --features lanesra-core/test-support --test integration_webhook`; `cargo test -p lanesra-server --features lanesra-core/test-support --test job_scheduler`
