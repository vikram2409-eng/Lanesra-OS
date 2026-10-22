# Regression baseline

The regression suite is everything CI runs. Totals may only go up; record new totals when a work
package completes. Linux, Rust 1.97.0, Node 22.22.0.

## Current baseline (on `main` from merge c85e620 of PR #492, 2026-10-16)

| Suite | Total | Passing | Failing | Notes |
|---|---|---|---|---|
| Rust (core, server, cli), `cargo test --locked --no-fail-fast` | 1,020 (124 binaries) | 1,019 | 0 | +11 feature-flag tests (WP-0010: 6 core integration, 2 core unit, 3 server HTTP). Before WP-0010: 1,009 / 122 binaries. |
| Rust baseline before WP-0010 | 1,009 (122 binaries) | 1,008 | 0 | 1 ignored. Re-run after WP-0004's rustfmt and clippy changes: identical. Clean build + run ≈ 75 min on the agent sandbox |
| Frontend unit (`npm ci` then `npx vitest run`) | 46 (6 files) | 46 | 0 | +6 feature-flag tests (WP-0010) |
| TypeScript / production build | — | clean / ok | — | |
| Program tools (`unittest discover program/tools`) | 52 | 52 | 0 | +7 migration-order, +5 binary-asset self-tests (WP-0003); +5 toolchain-pin self-tests (WP-0020); +7 feature-flag self-tests (WP-0010) |
| Team Workspace server image (`server image` CI job) | 1 smoke test (7 checks) | — | — | health, empty-volume status, first run, sign-in, create a record, restart, sign-in and record still present (WP-0020) |

**Added since this baseline (recorded when each work package completes):**

- WP-0182 (2026-10-21): +3 Rust tests: 1 in `core/tests/integration_webhook.rs` (a failing pending event does not block the next), 2 in `server/tests/job_scheduler.rs` (the scheduler delivers signed record webhooks; paused and other-object webhooks receive nothing). Run locally: `integration_webhook` 9 passed, `job_scheduler` 3 passed. Full totals come from the CI run on the pull request.
- WP-0183 (2026-10-21): +5 Rust tests (4 in `core/tests/integration_webhook.rs`: secret shown once, rotation, admin-only and unknown id, audit without the value; 1 in `server/tests/webhook_secret.rs`: create and rotate over HTTP) and +2 frontend tests (`src/__tests__/webhookSecret.test.tsx`: show-once dialog after create, confirm-then-show after rotate). Run locally: `integration_webhook` 13 passed (needs `--features test-support` when run with `-p lanesra-core` alone), `webhook_secret` 1 passed, Vitest file 2 passed. Full totals come from the CI run on the pull request.
- WP-0184 (2026-10-21): +17 Rust tests: 9 in `core/tests/network_trust.rs` (resolver truth table, list matching, validation), 1 in `core/tests/integration_api_client.rs` (malformed allowed network rejected at creation), 7 in `server/tests/api_network.rs` (allowed and refused on records, MCP and the event stream; forged headers; trusted proxy; 401 unchanged and refusal logged; unknown peer fails closed). Run locally: `network_trust` 9, `integration_api_client` 8, full `lanesra-server` suite green. Full totals come from the CI run on the pull request.
- WP-0109 (2026-10-22): +13 Rust tests (11 in `core/tests/trigger_identities.rs`: tool-call audit names the agent and effective principal from every run entry point, upgrade fixture `0081_triggers.db`, admin-scoped unattended runs, disabled identities, trigger identity validation; 2 in `server/tests/agent_v1.rs`: webhook without an active trigger is refused, identity and credential commands). +3 frontend tests (`agentIdentities.test.tsx`). Full local run: core and server 1,106 passed, 0 failed; vitest 54 passed.
- WP-0186 (2026-10-22): +6 tests: 4 in `server/tests/api_v1.rs` (search across objects with the object filter, 403 without `search.read`, archived records excluded, paging and validation), 1 in `core/tests/integration_api_object.rs` (custom-object records searchable until the object is deactivated), 1 in `cli/tests/cli_parsing.rs` (`lanesra search`).
- WP-0187 (2026-10-22): +9 tests: 7 in the new `server/tests/api_v1_bulk.rs` (streamed export of 3,000 records as NDJSON and CSV, export scope, mixed bulk write in order, per-item failures, 413 above 1000 with nothing applied, `objects.write` and `bulk.write` kept apart, Business Rules block a bulk item and a single write), 2 in `cli/tests/cli_parsing.rs`.
- WP-0188 (2026-10-22): +5 tests: 4 in the new `server/tests/api_v1_webhooks.rs` (lifecycle with signature checks before and after rotation and paged deliveries, 403 on every endpoint without `webhooks.manage`, connection type and outbound-guard refusals, audit with the client id and no secret), 1 in `cli/tests/cli_parsing.rs`.
- WP-0189 (2026-10-22): +6 tests: 3 in the new `server/tests/api_v1_integration_admin.rs` (listings without secrets, pause/reactivate/run audited and logged, read and manage kept apart), 1 in the new `server/tests/api_scope_vocabulary.rs` (every grantable scope enforced, allow and deny), 1 in `core/tests/integration_webhook.rs` (a paused connection gets no delivery), 1 unit test in `integration_admin_service` (URL credentials stripped).
- WP-0185 (2026-10-22): +6 tests: 5 in the new `core/tests/integration_webhook_events.rs` (two changed Company fields give two `field.changed` events and a no-op save none, custom-object fields delivered and `object_scope` honoured, `workflow.completed` and `workflow.failed` with a bounded error and no secret, nothing queued without a subscriber, a failing queue never blocks the write or the run), 1 in `server/tests/job_scheduler.rs` (the background scheduler delivers a signed `field.changed`).

## CI duration (WP-0003 AC-4)

The full suite runs in CI on every pull request and every push to `main` (`.github/workflows/ci.yml`),
with the Rust tests split across 4 `cargo nextest` partitions. Wall clock, push to last check:

| Run | Commit | Result | Duration |
|---|---|---|---|
| [38905651374](https://github.com/vikram2409-eng/Lanesra-OS/actions/runs/38905651374) | PR #624 head 2196270 | green | 12 min 21 s |
| [38918126664](https://github.com/vikram2409-eng/Lanesra-OS/actions/runs/38918126664) | `main` 5f27699 (merge of #624) | green | 12 min 12 s |
| [38918390735](https://github.com/vikram2409-eng/Lanesra-OS/actions/runs/38918390735) | PR #625 b82f577 (deliberate failure, AC-2) | red | 12 min 40 s |

**Median: 12 min 21 s.** The slowest job is a Rust test partition (8.5–11 min; the reference-package
suites take 65–205 s each); lint takes about 1 min, the Tauri shell about 2.5 min, and the web,
demo, migrations and program checks under 1 min each. Compare: a clean local build and run on the
agent sandbox takes about 75 min, which is why the full suite runs in CI and not in the AI's session
(tiered rule, `TEST-STRATEGY.md`).

## Original baseline on `main` @ `5cfc96c` (before fixes)

| Suite | Command | Total | Passing | Failing | Notes |
|---|---|---|---|---|---|
| Rust (core, server, cli) | `cargo test --no-fail-fast -p lanesra-core -p lanesra-server -p lanesra-cli` | 1,004 (122 binaries) | 996 | 7 | 1 ignored. Reference-package suites take 65–205 s each; plan CI around that |
| Frontend unit | `npx vitest run` (in `desktop/`) | 40 (5 files) | 40 | 0 | `npm ci` was broken (fixed by WP-0002) |
| TypeScript | `npx tsc --noEmit` | — | clean | — | |
| Online demo syntax | `node --check app.js` | — | clean | — | |
| Program tracker | `python3 -m unittest discover -s program/tools -p 'test_*.py'` | 13 | 13 | 0 | |
| Tauri crate | `cargo check -p lanesra-os` | not measured | | | Needs webkit2gtk system libs; WP-0003 |

Static count before the run: 963 `#[test]`/`#[tokio::test]` functions in `desktop/core/tests` and `desktop/server/tests`.

## Failures on main at `5cfc96c` (all fixed by WP-0016)

| Test | Failure |
|---|---|
| `core/tests/auto_service_reference_package.rs::repair_completed_workflow_closes_the_originating_appointment` | `appt_stage` stays "Checked In"; expected "Completed" |
| `core/tests/field_service_reference_package.rs::work_completed_workflow_propagates_completion_date_onto_the_linked_asset` | Workflow does not update the linked record |
| `core/tests/property_management_reference_package.rs::lease_activation_and_termination_workflows_flip_the_units_occupancy` | Workflow does not update the linked record |
| `core/tests/real_estate_reference_package.rs::offer_accepted_workflow_moves_the_listing_and_opens_a_transaction` | Workflow does not update the linked record |
| `core/tests/real_estate_reference_package.rs::transaction_closed_workflow_closes_the_listing_and_creates_a_task` | Workflow does not update the linked record |
| `core/tests/recruitment_reference_package.rs::offer_accepted_workflow_places_the_application_and_drafts_a_placement` | Workflow does not update the linked record |
| `core/tests/system_graph.rs::hard_delete_removes_the_node_and_cascades_its_edges_but_soft_deactivate_keeps_it` | Separate area (System Graph) |

Six of the seven are industry-package workflows that should update a *related* record and do not — likely one shared regression in cross-record workflow actions.

## Lint and format

- `cargo fmt --all --check` and `cargo clippy -p lanesra-core -p lanesra-server -p lanesra-cli --all-targets -- -D warnings` are clean (WP-0004); documented allow-list in `desktop/Cargo.toml`.

## Performance baseline (WP-0015)

**Date:** 2026-10-17  
**Method:** `cargo test --test perf_baseline --features lanesra-core/test-support -- --ignored --nocapture`  
**Provider:** Scripted provider (WP-0005) for deterministic numbers.  
**Machine:** CI ubuntu-latest (reported by perf-baseline.yml workflow).

| Metric | Value | Notes |
|---|---|---|
| agent_loop_per_round_us | TBD (CI) | 100 rounds, scripted provider |
| sqlite_bytes_per_run | TBD (CI) | Standard workload |
| checkpoint_write_ms | TBD (CI) | Single checkpoint write |

**Thresholds:** Not yet set (WP-0015 AC-2, pending owner). CI reports numbers but does not fail.
