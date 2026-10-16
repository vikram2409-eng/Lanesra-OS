# Regression baseline

The regression suite is everything CI runs. Totals may only go up; record new totals when a work
package completes. Measured on `main` @ `5cfc96c` (2026-10-16), Linux, Rust 1.97.0, Node 22.22.0.

| Suite | Command | Total | Passing | Failing | Notes |
|---|---|---|---|---|---|
| Rust (core, server, cli) | `cargo test --no-fail-fast -p lanesra-core -p lanesra-server -p lanesra-cli` | 1,004 (122 binaries) | 996 | 7 | 1 ignored. Reference-package suites take 65–205 s each; plan CI around that |
| Frontend unit | `npx vitest run` (in `desktop/`) | 40 (5 files) | 40 | 0 | Needs `npm install` until WP-0002 fixes `npm ci` |
| TypeScript | `npx tsc --noEmit` | — | clean | — | |
| Online demo syntax | `node --check app.js` | — | clean | — | |
| Program tracker | `python3 -m unittest discover -s program/tools -p 'test_*.py'` | 13 | 13 | 0 | |
| Tauri crate | `cargo check -p lanesra-os` | not measured | | | Needs webkit2gtk system libs; WP-0003 |

Static count before the run: 963 `#[test]`/`#[tokio::test]` functions in `desktop/core/tests` and `desktop/server/tests`.

## Known failures on main (must be fixed by WP-0016 before CI is required)

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

## Known warnings

- `core/src/services/ontology_service.rs:16` unused import `entity_registry` (clippy `-D warnings` will fail; WP-0004).
