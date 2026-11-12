# G7 — Process Lanes

Closes **Phase 7 · Process Lanes** (ADR-0023). Phase 7 is a parallel track and holds back no other gate.

Status: **Not started**

## Checks

Each check needs evidence: a CI run link, test names, `programctl` output or a demo note.

| # | Check | Evidence | Result |
|---|---|---|---|
| 1 | Every Phase 7 requirement delivered: `programctl trace --fail-on-gaps` passes | | ☐ |
| 2 | A first-time user completes Receive Shipment on the lane screen alone; records match the worked example (REQ-NFR-008) | | ☐ |
| 3 | The same walkthrough gives the same result in desktop, Team Workspace and the online demo | | ☐ |
| 4 | No step write is repeated on retry, double click or restart; a step's writes are all-or-nothing | | ☐ |
| 5 | No lane with a dead-end branch can be published; workflow replacement is visible in both editors | | ☐ |
| 6 | Help article covers features, setup and where to find lanes | | ☐ |
| 7 | Owner sign-off | | ☐ |

## Open RAID items at the gate

(list, with the owner's decision on each)

## Throughput and re-forecast

| Measure | Value |
|---|---|
| Work packages completed this phase | |
| Estimate points delivered (S=1, M=2, L=4, XL=8) | |
| Points delivered per agent (Claude Code, Codex, Muse AI) | |

## Sign-off

| Role | Name | Date | Decision |
|---|---|---|---|
| Program owner | | | |
