# G0 — Engineering readiness

Closes **Phase 0 · Precursors**. Until this gate passes, **Phase 1 · Foundations** work packages stay `proposed`.

Status: **Not started**

## Checks

Each check needs evidence: a CI run link, test names, `programctl` output or a demo note.

| # | Check | Evidence | Result |
|---|---|---|---|
| 1 | CI runs on every PR and is green on `main` (WP-0003); owner has enabled required checks (D-01) | | ☐ |
| 2 | Zero failing tests on `main`; baseline recorded in testing/REGRESSION.md (WP-0016) | | ☐ |
| 3 | `npm ci` and `cargo build --locked` succeed from a clean clone (WP-0002) | | ☐ |
| 4 | Scripted model provider available and documented (WP-0005) | | ☐ |
| 5 | E2E harness and migration upgrade harness run in CI (WP-0006, WP-0007) | | ☐ |
| 6 | Gap audit merged; discrepancies in RAID (WP-0008) | | ☐ |
| 7 | Six foundational ADRs Accepted (WP-0009); feature flags available (WP-0010) | | ☐ |
| 8 | Threat model reviewed; dependency and secret scanning in CI (WP-0011) | | ☐ |
| 9 | Clean-room controls in place (WP-0012); demo ADR decided (WP-0013) | | ☐ |
| 10 | Design tokens inventoried and axe checks running (WP-0014); performance baseline recorded (WP-0015) | | ☐ |
| 11 | Team Workspace server image builds from the lockfile with pinned toolchains, and is smoke-tested in CI on every PR (WP-0020) | | ☐ |
| 12 | Phase 1 WPs refined and moved to `ready`; re-forecast written below | | ☐ |

## Open RAID items at the gate

(list, with the owner's decision on each)

## Throughput and re-forecast

| Measure | Value |
|---|---|
| Work packages completed this phase | |
| Estimate points delivered (S=1, M=2, L=4, XL=8) | |
| Calendar days | |
| Forecast for remaining phases | |

## Sign-off

| Role | Name | Date | Decision |
|---|---|---|---|
| Program owner | | | |
