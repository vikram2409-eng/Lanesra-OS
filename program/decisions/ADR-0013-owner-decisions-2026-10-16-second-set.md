# ADR-0013: Owner decisions, 2026-10-16 (second set)

- Status: Accepted (decided by the program owner, Vikram Grover, 2026-10-16)
- Answers the review questions on PR #628 (ADR-0007, ADR-0008, ADR-0012) and RAID Q-14 and Q-15
- Amends ADR-0007 §5, ADR-0008 §4 and ADR-0012 §1 and §5; adds ADR-0014
- Requirements added: REQ-PRE-021, REQ-PRE-022, REQ-PRE-023
- Work packages added or changed: WP-0013, WP-0100, WP-0111, WP-0141, WP-0180, WP-0223, new WP-0181, WP-0420, WP-0421, WP-0422

## Decisions

| # | Question | Decision | Consequence in the plan |
|---|---|---|---|
| 1 | Should a crashed run restart on its own only when that is safe, and otherwise wait for a person? | **Yes, and an admin can configure it.** | ADR-0007 §5 gains `interrupted_run_policy`: `resume_if_safe` (default) or `always_pause`, set per workspace with an optional per-Team override. AI Admin only; every change is audited. Built in WP-0141. |
| 2 | Leave prompt and output text out of traces by default? | **Yes by default, and configurable.** | ADR-0008 §4 gains `trace_bodies`: `off` (default), `redacted` or `full`, set per workspace with an optional per-Team override, and applied to storage, live events and OTLP export alike. AI Admin only; every change is audited; the trace says which setting was in force. Built in WP-0111. |
| 3 | May the online demo show features before the desktop app releases them? | **No.** A feature is built and released in desktop and Team Workspace, and appears in the online demo **in the same release, never earlier.** | ADR-0012: the demo has no separate early default. A flag's demo state follows the release in which the feature reaches `ga` for desktop and Team Workspace. New REQ-PRE-021. WP-0013 and WP-0223 follow this rule. |
| 4 | Several workspaces on one Team Workspace server? Database choice? | **Yes, several workspaces per server.** A **SaaS version is planned for the future**, so design decisions must keep it possible. **Organisations should be able to choose PostgreSQL** (or another open-source database) for Team Workspace. | New ADR-0014 (tenancy, SaaS readiness and database portability). New WP-0181 (several workspaces on one server, Phase 1). WP-0420 (storage layer) and WP-0421 (PostgreSQL), both Phase 4. WP-0422 (SaaS architecture plan, Phase 4). New REQ-PRE-022 and REQ-PRE-023. RAID Q-14 answered. |
| 5 | Should the desktop app connect to a Team Workspace server? | **No. The browser is the only way a team uses Team Workspace.** | The desktop stays a standalone single-user app. Team features are delivered through the browser only. ADR-0008's Tauri channel serves the desktop edition's own runs only. RAID Q-15 answered. |

## Is PostgreSQL support too complicated?

No, provided it is planned as its own stage rather than squeezed into the Playground build. Measured on 2026-10-16:

- SQL is already almost entirely behind a repository layer: 85 repository files. Only 12 of 111 services run SQL themselves.
- SQLite-only syntax is rare: about 45 places (`PRAGMA`, `datetime('now')`, `GROUP_CONCAT`, `INSERT OR IGNORE`, `rowid`).
- The real work is that every service receives a synchronous `rusqlite::Connection`, and the 79 migrations (about 4,800 lines) are written in SQLite's dialect.

The estimate is two extra-large work packages (WP-0420 and WP-0421, about 16 points), done after the Playground reaches general availability, with a rule from now on that new code must not make the move harder (ADR-0014). Doing it earlier would stall the Playground build for little user value, because a single-file database serves small and medium teams well.

## Consequences

- None of these decisions removes scope. Each one adds a setting or a later work package.
- The SaaS direction mostly changes *how* foundation work is built (tenant boundaries, keys, portable SQL), not *when* the Playground ships.
