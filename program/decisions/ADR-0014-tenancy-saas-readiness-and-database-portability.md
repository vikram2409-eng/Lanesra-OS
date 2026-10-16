# ADR-0014: Tenancy, SaaS readiness and database portability

- Status: Accepted (program owner, 2026-10-16, PR #628)
- Date: 2026-10-16
- Work package: WP-0009 (decided with the owner, ADR-0013 #4)
- Requirements: REQ-PRE-022, REQ-PRE-023
- Constrains: WP-0100, WP-0106, WP-0180, WP-0181, WP-0420, WP-0421, WP-0422, and every new table and repository (through ADR-0010)

## Context

The owner wants three things:

- several workspaces on one Team Workspace server;
- a SaaS edition in the future;
- the option of PostgreSQL (or another open-source database) for Team Workspace.

Today, measured on 2026-10-16, a Team Workspace server is one SQLite file holding **one** workspace:

- `require_workspace_id` (`server/src/dispatch.rs:117`) reads "the current workspace" from the database, not from the user's session. Around 290 call sites use it.
- 107 of 153 tables carry `workspace_id`. The other 46 are child rows reached through a parent (order lines, graph nodes, run nodes and so on), plus a few global tables (`roles`, `workspaces`).
- One `secret.key` encrypts every secret.
- SQL sits mostly in 85 repository files. 12 services also run SQL directly. About 45 statements use SQLite-only syntax.

## Decision

### 1. The workspace is the tenant boundary, everywhere

A workspace is the unit of isolation, billing (later), backup, encryption and data residency. No row, key, file, event or cache entry may be shared between workspaces, except account identity (§3).

### 2. Team Workspace: one database per workspace (WP-0181)

- A server keeps a small **control database** (`/data/control.sqlite3`). It holds:
  - accounts (people who can sign in);
  - their memberships in workspaces;
  - sessions;
  - the workspace registry (id, name, status, database location, key reference).
- Each workspace has **its own database** (`/data/workspaces/<id>/workspace.sqlite3`) and **its own data key**.
- The request's session selects the workspace. `require_workspace_id` becomes "the workspace this session is signed in to", and the repository connection comes from that workspace's database.
- Reasons for this design:
  - **Isolation by construction.** A query cannot read another workspace's rows.
  - **Backup, restore, export and deletion are per file.** Deleting a workspace is deleting a directory.
  - **The existing single-workspace code keeps working inside each database,** so this is far less risky than adding `workspace_id` filters to every query.
- The desktop edition is unchanged: one local workspace.

### 3. Accounts and principals

- An **account** (an email, a password or SSO link, MFA later) lives in the control database. One account can be a member of several workspaces, with a different role in each.
- Inside a workspace, the WP-0100 principal (human, Agent Identity or API client) refers to the account by id for humans. Agent Identities and API clients belong to exactly one workspace and never cross it.

### 4. Keys

- The server master key, held outside the data (`secret.key` today; a KMS later), **wraps one data key per workspace**. That data key encrypts the workspace's secrets.
- Rotating the master key rewraps the data keys only.
- Key recovery (WP-0180) exports one workspace's data key, wrapped with a passphrase.

### 5. Database portability: rules from now on

PostgreSQL support itself comes later (§6). From the moment this ADR is accepted, new code must not make it harder:

1. **SQL lives in `repositories/` only.** Services call repository functions. New raw SQL in a service is a review blocker. The 12 services that run SQL today move it out when a work package next touches them.
2. **New SQL uses the portable subset:**
   - timestamps come from Rust (`now_iso()`), never `datetime('now')`;
   - upserts use `INSERT ... ON CONFLICT (...) DO ...`, never `INSERT OR REPLACE`/`IGNORE`;
   - no `rowid`;
   - no `GROUP_CONCAT` (build lists in Rust);
   - `PRAGMA` appears only in `db/connection.rs`.
3. **Types stay portable:**
   - ids are UUID text;
   - money uses integer micros (ADR-0010);
   - booleans are `INTEGER 0/1` behind typed Rust fields;
   - JSON is text in `_json` columns, parsed in Rust.
4. **Every new table** carries `workspace_id` when it is workspace-owned, as ADR-0010 already requires. Even with one database per workspace, this keeps a shared-database SaaS deployment (§7) possible without another migration.

### 6. PostgreSQL for Team Workspace (Phase 4: WP-0420, then WP-0421)

- **WP-0420, storage layer:** a database handle type at the repository boundary that can be backed by SQLite or PostgreSQL, used by all 85 repositories. Migrations are kept per dialect, with an equivalence test proving both schemas match.
- **WP-0421, PostgreSQL adapter:**
  - the server chooses its database with `LANESRA_DATABASE_URL`, and SQLite remains the default;
  - CI runs the full regression suite against both databases;
  - a migration tool copies an existing SQLite workspace into PostgreSQL.
- Other open-source databases (MySQL or MariaDB) are not planned. The storage layer must not rule them out, but each would be its own work package.

### 7. SaaS readiness (WP-0422 writes the plan; it is not built in this program)

The SaaS edition is expected to run on PostgreSQL. It can use either database-per-tenant (as in §2) or a shared database with row-level security on `workspace_id`; WP-0422 compares the two. Either way, the foundations above make it possible:

- the tenant boundary;
- the control plane separated from tenant data;
- a key per tenant;
- portable SQL;
- `workspace_id` on every table;
- no reliance on the local file system except through the storage and backup layers.

## Example

```
/data/control.sqlite3                      accounts, memberships, sessions, workspace registry
/data/workspaces/7f3c…/workspace.sqlite3   everything belonging to workspace 7f3c…
/data/workspaces/7f3c…/backups/            that workspace's scheduled backups (WP-0180)
/data/secret.key                           server master key; wraps each workspace's data key
```

## Consequences

- WP-0100 (principals) and WP-0181 (several workspaces) must be designed together. Accounts move to the control database, and principals stay per workspace.
- WP-0180's backups and key recovery work per workspace.
- PostgreSQL support costs about two extra-large work packages after general availability. The portability rules keep that cost from growing in the meantime.
- Existing single-workspace servers are migrated by moving their file into `/data/workspaces/<id>/` and creating the control database. WP-0181 includes that upgrade, tested by the WP-0007 harness.
