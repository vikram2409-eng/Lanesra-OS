# Regression baseline

The regression suite is everything CI runs. Totals may only go up; record new totals when a work
package completes. Measured on `main` @ `5cfc96c` (2026-10-16), Linux, Rust 1.97.0, Node 22.22.0.

| Suite | Command | Total | Passing | Failing | Notes |
|---|---|---|---|---|---|
| Rust (core, server, cli) | `cargo test --no-fail-fast -p lanesra-core -p lanesra-server -p lanesra-cli` | PENDING | PENDING | ≥ 1 | Full run in progress at commit time; see below |
| Frontend unit | `npx vitest run` (in `desktop/`) | 40 (5 files) | 40 | 0 | Needs `npm install` until WP-0002 fixes `npm ci` |
| TypeScript | `npx tsc --noEmit` | — | clean | — | |
| Online demo syntax | `node --check app.js` | — | clean | — | |
| Program tracker | `python3 -m unittest discover -s program/tools -p 'test_*.py'` | 13 | 13 | 0 | |
| Tauri crate | `cargo check -p lanesra-os` | not measured | | | Needs webkit2gtk system libs; WP-0003 |

Static count: 963 `#[test]`/`#[tokio::test]` functions in `desktop/core/tests` and `desktop/server/tests`.

## Known failures on main (must be fixed by WP-0016 before CI is required)

| Test | Failure |
|---|---|
| `core/tests/auto_service_reference_package.rs::repair_completed_workflow_closes_the_originating_appointment` | `appt_stage` stays "Checked In"; expected "Completed" |

## Known warnings

- `core/src/services/ontology_service.rs:16` unused import `entity_registry` (clippy `-D warnings` will fail; WP-0004).
