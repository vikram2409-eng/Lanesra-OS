---
id: WP-0181
title: Several workspaces on one Team Workspace server
phase: 1
epic: E1.9 Team Workspace platform
status: proposed
owner: unassigned
depends_on: [WP-0100, WP-0020, WP-0007]
requirements: [REQ-PRE-022]
threats: [T-12]
estimate: XL
editions: [workspace]
editions_reason: The desktop edition stays a single local workspace (ADR-0014 §2), and the online demo has one sample workspace
delivered: []
docs: [set-up-team-workspace]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0181 — Several workspaces on one Team Workspace server

## Goal

The owner decided on 2026-10-16 (ADR-0013 #4) that one Team Workspace server should host several workspaces, for example one per department or per customer, and that the design must keep a future SaaS edition possible.

Today a server holds exactly one workspace. "The current workspace" is read from the database (`server/src/dispatch.rs:117`, about 290 call sites), not from the user's session. This package implements ADR-0014 §2–§4: a control database, one database per workspace, and a data key per workspace.

## Scope

- **Control database** (`/data/control.sqlite3`) with these tables:
  - accounts (people who can sign in);
  - memberships (account, workspace, role);
  - sessions;
  - workspace registry (id, name, status, database path, wrapped data key).
- **Per-workspace databases** (`/data/workspaces/<id>/workspace.sqlite3`), each with its own pool and connection. The session picks the workspace, so `require_workspace_id` returns the session's workspace.
- **Sign-in and switching:** after sign-in, a person with several memberships picks a workspace, and can switch later without signing in again. The URL carries the workspace (`/w/<id>/…`) so links are unambiguous.
- **Server administration** (a new server-admin role, distinct from a workspace Administrator):
  - create, rename, suspend and delete workspaces;
  - invite the first Administrator of each workspace.
- **Data keys:** a key per workspace, wrapped by the server master key (ADR-0014 §4). Secrets are re-encrypted during the upgrade.
- **Per-workspace operation:** the background scheduler and the interrupted-run sweep loop over active workspaces. Backups and key recovery (WP-0180) work per workspace.
- **Upgrade of an existing single-workspace server:**
  - move its file into `/data/workspaces/<id>/`;
  - create the control database;
  - move users into accounts and memberships.

  This is tested by the WP-0007 upgrade harness.
- **Isolation tests:** with two workspaces, no route, event stream, search, export, backup or scheduled job can read or affect the other.

## Acceptance criteria

- [ ] AC-1: Two workspaces on one server are fully isolated. A cross-workspace test suite covers every `/api/invoke` command family, the run event stream, search, export and backup
- [ ] AC-2: One account belongs to two workspaces with different roles; switching changes what it can see and do (test)
- [ ] AC-3: An existing single-workspace server upgrades with no data loss, users can sign in afterwards, and stored secrets still decrypt (upgrade-harness test)
- [ ] AC-4: Deleting a workspace removes its database, backups and key, and is audited in the control database
- [ ] AC-5: The help article explains workspaces, the server-admin role and switching

## Test plan

- New `desktop/server/tests/multi_workspace_isolation.rs`; extend the WP-0007 harness
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0008 trace and events; ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0012 feature flags; ADR-0014 tenancy. ADR-0010 applies to any migration this work package adds.
- Design it together with WP-0100: accounts live in the control database, while principals stay per workspace.
- Split at refinement (XL): (a) control database, accounts and sessions; (b) per-workspace databases and routing; (c) data keys and secret re-encryption; (d) server admin UI and switching; (e) the upgrade path.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
