# ADR-0007: One state machine for runs and steps

- Status: Accepted (program owner, 2026-10-16, PR #628)
- Date: 2026-10-16
- Work package: WP-0009
- Requirements: REQ-PRE-007, REQ-ORCH-016 (`waiting_budget`), REQ-ORCH-025 (`waiting_model`), REQ-ORCH-044, REQ-LOOP-008 (`completed_partial`), REQ-GOV-010, REQ-GOV-023, REQ-API-003
- Constrains: WP-0110, WP-0111, WP-0112, WP-0121, WP-0126, WP-0141, WP-0150, WP-0170, and every Phase 2–4 work package that reads or changes run status

## Context

Today two engines keep run state in two vocabularies:

- **Execution graphs** (`ai_runs`, `ai_run_nodes`, migration 0062) declare 11 run statuses in `desktop/core/src/models/graph_run.rs:7-19`. Three of them, `planning`, `waiting_tool` and `waiting_agent`, are never set by any code.
- **Pipelines** (`ai_agent_runs`, `ai_agent_run_steps`, migration 0040) use their own statuses, including `awaiting_approval`.

The spec adds `waiting_budget` and `completed_partial`, and Phase 1 adds `waiting_model`. The gap audit (RAID I-12) found that a run interrupted while `running` can never be resumed: `resume_run` accepts only waiting statuses, and nothing looks for interrupted runs at startup.

If every work package adds statuses as it goes, the Playground, the approvals inbox, dashboards and the API will each disagree about what a run is doing.

## Decision

### 1. Run statuses

| Status | Meaning | Terminal | Holds a lease |
|---|---|---|---|
| `queued` | Accepted, waiting for a worker | no | no |
| `running` | A worker is executing a step | no | **yes** |
| `paused` | Stopped by a person, or by the interrupted-run sweep (§5) | no | no |
| `waiting_approval` | Blocked on one `ai_approvals` row (`pending_approval_id`) | no | no |
| `waiting_budget` | Run cost or token budget used up; an owner may top up or cancel | no | no |
| `waiting_model` | A long provider call is running in the background (REQ-ORCH-025, Phase 4); the run resumes when the result arrives, even after a restart | no | no |
| `waiting_scheduled` | A `delay` node; `resume_at` is set | no | no |
| `completed` | Every reached end node finished | yes | no |
| `completed_partial` | Finished, but at least one step ended `completed_partial` (a limit was hit, and the best answer so far was kept) | yes | no |
| `failed` | Stopped on an error that cannot be retried (ADR-0009) | yes | no |
| `cancelled` | Stopped by a person or by policy | yes | no |

`planning` is kept, unused, for the Manager-led plan approval in Phase 3; it behaves as a waiting status. `waiting_tool` and `waiting_agent` are removed. Nothing sets them, so the migration only rewrites stray rows to `paused`.

### 2. Step statuses (`ai_run_nodes.status`)

`pending`, `running`, `waiting_approval`, `completed`, `completed_partial`, `failed`, `skipped`, `cancelled`. A step has one row per attempt (`attempt` starts at 1).

### 3. Legal run transitions

| From | To | Trigger |
|---|---|---|
| `queued` | `running` | A worker takes the lease |
| `queued` | `cancelled` | Cancel |
| `running` | `waiting_approval` | An approval node, or a tool needing approval (WP-0150) |
| `running` | `waiting_budget` | A budget limit (WP-0126) |
| `running` | `waiting_model` | A long model call was handed to the background (Phase 4) |
| `running` | `waiting_scheduled` | A delay node |
| `running` | `paused` | Pause, or the interrupted-run sweep |
| `running` | `completed` / `completed_partial` / `failed` / `cancelled` | End of run, error, or cancel |
| any waiting status, or `paused` | `queued` | Resume, approve, top-up, `resume_at` reached, or background model result arrived |
| any waiting status, or `paused` | `cancelled` | Cancel, or reject when the approval has no `rejected` edge |
| `waiting_approval` | `failed` | Reject with fail semantics |

Nothing leaves a terminal status. Replay and fork create a **new** run with `parent_run_id` or `fork_of_step_id` (WP-0110); the old run is never reopened.

### 4. One guard function

Every status change goes through one core function, `run_state::transition(conn, run_id, from, to, stop_reason)`. It:

- checks the table above;
- updates the row only if it still holds the `from` status (`UPDATE ... WHERE id = ? AND status = ?`), so two workers cannot both move a run;
- records `stop_reason` on terminal and waiting statuses;
- emits the matching event (ADR-0008).

Writing `status` directly in SQL anywhere else is a review blocker. Each repository exposes only the transition call.

`stop_reason` values: `end_reached`, `limit_rounds`, `limit_tokens`, `limit_wall_clock`, `budget_exhausted`, `approval_pending`, `approval_rejected`, `guardrail_failed`, `policy_denied`, `model_unavailable` (every tier failed after retries, ADR-0009), `error`, `cancelled_by_user`, `cancelled_by_policy`, `interrupted`.

### 5. Leases and interrupted runs (fixes RAID I-12)

- A worker that moves a run to `running` sets `lease_owner` (a process id) and `lease_expires_at` (now + 2 minutes). It renews the lease every 30 seconds while working.
- At startup, and on every scheduler tick, the server and the desktop shell run one sweep (WP-0141). The desktop needs its own background ticker for this (RAID I-19).
  - Runs `running` with an expired lease become `paused` with `stop_reason = interrupted`, unless the step they were on is marked replay-safe (no side effects, or idempotency key recorded, WP-0141). Replay-safe runs go back to `queued`.
  - Runs `waiting_scheduled` with `resume_at` in the past become `queued`.
  - Runs `waiting_model` whose background result has arrived become `queued`.
- A person resumes an `interrupted` run from the Playground or the approvals inbox, after seeing which step was in flight.
- **Admins choose the policy (owner decision, ADR-0013 #1).** The setting is `interrupted_run_policy`:
  - `resume_if_safe` (default): replay-safe runs are re-queued, and the others pause as described above.
  - `always_pause`: every interrupted run waits for a person.

  It is set per workspace, with an optional override per Team. Only an AI Admin can change it, and every change is audited. The sweep reads it for each run.

### 6. Pipelines converge

Pipeline runs map onto this vocabulary when WP-0150 moves their approvals onto `ai_approvals` (`awaiting_approval` becomes `waiting_approval`). When pipelines become Teams (WP-0306), pipeline runs become ordinary runs. Until then, any API or UI that shows both kinds of run maps pipeline statuses with the table below:

| Pipeline status | Shown as |
|---|---|
| `awaiting_approval` | `waiting_approval` |
| `succeeded`, or any completed status | `completed` |
| anything else | the same name, if it is in §1; otherwise `failed` |

### 7. Online demo

The online demo uses the same status names and transitions on its sample data. A simulated run never invents a status that is not in §1.

## Example

A run that hits the round limit on one step, then needs approval, then finishes:

```
queued → running                         (lease taken)
running → waiting_approval               stop_reason=approval_pending, pending_approval_id=apv_…
waiting_approval → queued                (approved)
queued → running
running → completed_partial              stop_reason=end_reached; step "draft_reply" ended completed_partial (limit_rounds)
```

## Consequences

- One status column means one vocabulary for the Playground, the API (`GET /api/v1/runs/{id}`), events and dashboards.
- The lease and sweep give real crash recovery without double execution. The cost is one renewal write every 30 seconds per running run.
- A migration (WP-0110) adds `stop_reason`, `lease_owner` and `lease_expires_at`, and rewrites any stray `waiting_tool` or `waiting_agent` rows.
- Adding a status later needs a new ADR that amends this one.
