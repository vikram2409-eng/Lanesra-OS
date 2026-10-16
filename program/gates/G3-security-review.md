# G3 — Security review signed off

Closes **Phase 3 · Teams and modes**. Until this gate passes, **Phase 4 · Enterprise scale** work packages stay `proposed`.

Status: **Not started**

## Checks

Each check needs evidence: a CI run link, test names, `programctl` output or a demo note.

| # | Check | Evidence | Result |
|---|---|---|---|
| 1 | Security review WP-0331 complete; findings fixed or accepted by the owner in writing | | ☐ |
| 2 | Prompt-injection corpus run against every mode; results recorded | | ☐ |
| 3 | Authorisation matrix (roles × endpoints × identities) fully tested | | ☐ |
| 4 | Agent Identities never exceed their scope; interactive runs never exceed the user (tests) | | ☐ |
| 5 | Pipelines converted to Teams on upgrade fixtures without loss (WP-0306) | | ☐ |
| 6 | `programctl trace` shows every Phase 3 requirement with a tagged test | | ☐ |
| 7 | Phase 4 WPs refined and `ready`; re-forecast written below | | ☐ |

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
