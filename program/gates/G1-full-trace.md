# G1 — Every run produces a full trace

Closes **Phase 1 · Foundations**. Until this gate passes, **Phase 2 · Playground** work packages stay `proposed`.

Status: **Not started**

## Checks

Each check needs evidence: a CI run link, test names, `programctl` output or a demo note.

| # | Check | Evidence | Result |
|---|---|---|---|
| 1 | Full-trace conformance suite green for every run entry point (WP-0113) | | ☐ |
| 2 | 100% of tool calls have a complete audit trail (REQ-NFR-003) | | ☐ |
| 3 | Run events validate against published JSON Schemas (WP-0112) | | ☐ |
| 4 | Task contracts, output schemas, limits and partial results proven by tests (WP-0121, WP-0131, WP-0132) | | ☐ |
| 5 | All approvals on ai_approvals; legacy paths removed (WP-0150) | | ☐ |
| 6 | Prompt catalog in place; no hard-coded system prompts (WP-0130) | | ☐ |
| 7 | OIDC and SAML sign-in working with negative tests (WP-0102, WP-0103) | | ☐ |
| 8 | `programctl trace` shows every Phase 1 requirement with a tagged test | | ☐ |
| 9 | Threat model updated for Phase 1; no open high-severity findings | | ☐ |
| 10 | Phase 2 WPs refined and `ready`; re-forecast written below | | ☐ |

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
