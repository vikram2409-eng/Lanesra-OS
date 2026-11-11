# G5 — Platform release

Closes **Phase 5 · Platform and ecosystem** (ADR-0020). Phase 5 runs beside Phase 4 once G3 has passed; this gate does not hold back G4.

Status: **Not started**

## Checks

Each check needs evidence: a CI run link, test names, `programctl` output or a demo note.

| # | Check | Evidence | Result |
|---|---|---|---|
| 1 | Every Phase 5 requirement delivered: `programctl trace --fail-on-gaps` passes | | ☐ |
| 2 | Walkthrough: register an external agent, govern it and see its first run in under 30 minutes (WP-0599, REQ-NFR-006) | | ☐ |
| 3 | Security review of A2A, channels, token vault, MCP gateway and custom tools has no open high findings | | ☐ |
| 4 | Red-team suite runs in the publish gate of every shipped template | | ☐ |
| 5 | High-availability takeover and restore drill meet the documented targets | | ☐ |
| 6 | Help articles, API reference and SDK documentation complete | | ☐ |
| 7 | Owner sign-off | | ☐ |

## Open RAID items at the gate

(list, with the owner's decision on each)

## Throughput and re-forecast

| Measure | Value |
|---|---|
| Work packages completed this phase | |
| Estimate points delivered (S=1, M=2, L=4, XL=8) | |
| Points delivered per agent (Claude Code, Codex, Muse AI) | |
| Forecast for remaining phases | |

## Sign-off

| Role | Name | Date | Decision |
|---|---|---|---|
| Program owner | | | |
