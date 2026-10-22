---
id: WP-0185
title: "Emit the field.changed, workflow.completed and workflow.failed webhook events"
phase: 1
epic: E1.9 Team Workspace platform
status: in_progress
owner: claude
depends_on: [WP-0182]
requirements: [REQ-INT-003, REQ-INT-001]
threats: []
estimate: M
editions: [workspace, desktop]
editions_reason: The REST API exists only on the Team Workspace server; desktop opens no network port and the demo has no real API
delivered: []
docs: [api-access-and-webhooks]
branch: claude/enterprise-build-planning-jxy2kq
pr:
updated: 2026-10-22
---

# WP-0185 — Emit the field.changed, workflow.completed and workflow.failed webhook events

## Goal

Three webhook event types can be subscribed to today and nothing emits them, so an integration that subscribes to them silently gets nothing (RAID I-29, Q-20). The owner decided an enterprise needs them, so build the emitters (ADR-0018 section 3). Record events already work after WP-0182.

## Scope

- `field.changed`: when a record update changes a field value, queue one event per changed field with `data` = `{field, previous, current}`. Applies to built-in and custom objects through the shared write path, so UI, API and workflows are all covered
- `workflow.completed` / `workflow.failed`: when a workflow run ends, queue an event with `data` = `{workflow_id, workflow_name, run_id, status, trigger_record}` and, on failure, a short error message (no secrets)
- Events are queued only when an active webhook subscribes (the existing cheap check in `event_hooks::try_emit`), never block or fail the originating write, and are delivered by the WP-0182 drain
- Honor the webhook's `object_scope` for `field.changed`
- Extend `event_hooks` with the new emitters and keep one helper per event, as the record events do

## Out of scope

- Per-field subscription filters (the stored `filter_json` is still unused)
- New event types beyond the six already offered
- Delivery mechanics (WP-0182) and signing (WP-0183)

## Acceptance criteria

- [ ] AC-1: Updating two fields of a Company queues and delivers two `field.changed` events with the right `field`, `previous` and `current`; updating without changing any value queues none
- [ ] AC-2: `field.changed` is emitted for a custom object's fields as well as built-in ones, and respects `object_scope`
- [ ] AC-3: A workflow run that completes queues and delivers one `workflow.completed`; a run that fails queues one `workflow.failed` with an error message and no secret in it
- [ ] AC-4: With no webhook subscribed to these types, nothing is queued and the write path is unchanged (no extra rows, same result)
- [ ] AC-5: A failure while queueing an event never fails or rolls back the record write or the workflow run
- [ ] AC-6: The `api-access-and-webhooks` help article and the Integrations page list the six event types and their payloads

## Test plan

- `desktop/core/tests/integration_webhook.rs` (or a new `integration_webhook_events.rs`): AC-1 to AC-5 against the real write path and the workflow runner, with the drain delivering to a local receiver
- `desktop/server/tests/job_scheduler.rs`: one end-to-end case proving a `field.changed` reaches a receiver through the scheduler
- Tag every new test with `Covers: REQ-INT-003 (WP-0185 AC-n)`
- Full regression suite green in CI; test count not lower than before

## Notes

- Hook points: the entity services call `event_hooks::record_updated` after commit; the changed-field diff needs the previous values, which the update paths already load. The workflow end is in `workflow_service` run completion.
- Payload size: send values as strings truncated to a sane limit (propose 2 KB per value) and say so in the docs.
- Privacy: `previous` and `current` can contain personal data. Document that webhook receivers should be treated as data recipients, and do not emit values for fields marked secret if such a field class exists.

## Progress log

<!-- Append dated entries, newest last. -->
- 2026-10-21 claude: Written as `ready`. Not started. Starts after WP-0182 merges.
- 2026-10-22 claude: Claimed; WP-0182 is done.
