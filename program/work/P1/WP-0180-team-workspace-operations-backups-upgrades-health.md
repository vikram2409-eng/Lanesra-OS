---
id: WP-0180
title: "Team Workspace operations: backups, key recovery, upgrades and health"
phase: 1
epic: E1.9 Team Workspace platform
status: proposed
owner: unassigned
depends_on: [WP-0020, WP-0007]
requirements: [REQ-PRE-020]
threats: [T-17]
estimate: L
editions: [workspace, desktop]
editions_reason: Operating real data on a server or a desktop; the online demo keeps its sample data in the browser
delivered: []
docs: [set-up-team-workspace, workspace-essentials]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0180 — Team Workspace operations: backups, key recovery, upgrades and health

## Goal

Once agents run unattended on a Team Workspace (schedules, webhooks, long runs), the server has to survive upgrades, restarts and hardware loss. The 2026-10-16 review found these gaps:

- **Backups are manual only.** `backup_service::create_backup` exists, but nothing runs it on a schedule or before an upgrade.
- **A restored backup loses every stored secret (RAID I-28).** The `.lanesra` backup holds the database but not `secret.key`, the master key that encrypts every LLM provider key and connector credential (AES-256-GCM, `secret_service`). Restored on new hardware, or after losing the volume, all of those secrets become unreadable, and nothing tells the admin beforehand.
- **Upgrades run migrations on start with no safety copy.** Migrations also run without a transaction today (ADR-0010 §2, which WP-0007 fixes).
- **Health checks are shallow.** `/api/health` returns `ok` without checking the database, the scheduler or free disk space.

## Scope

- **Key recovery.**
  - An admin action to export the master key as a passphrase-protected recovery file.
  - An import step during restore.
  - The backup screen warns plainly when no key export exists.
  - The key is never included in a plain backup.
  - Restore on a machine whose key does not match explains what is unreadable and offers the import.
- **Scheduled backups.**
  - Daily by default to `/data/backups` (desktop: the app data folder).
  - Admin-set retention, defaulting to 14 days.
  - An optional copy to an admin-configured folder.
  - Each run is audited and shown on the backup screen.
- **Backup before upgrade.** On start, if migrations are pending, take a backup first. If a migration fails, the server refuses to start and names the backup to restore. Uses the transactional runner from WP-0007.
- **Health and readiness.**
  - `/api/health` (live): the process is up.
  - `/api/ready`: the database opens and the schema is current, the scheduler ticked within 3 intervals, free disk is above a threshold, and the master key loads.
  - The Docker image uses `/api/ready` in its `HEALTHCHECK`.
- **Logs.** Structured JSON log option (`LANESRA_LOG_FORMAT=json`). A request id on every log line and API error, so an admin can match a user's error to the log.
- **Upgrade guide.** Steps for Docker, Compose and the desktop app, covering rollback by restoring the pre-upgrade backup. Added to the `set-up-team-workspace` and `workspace-essentials` help articles.
- Owner decisions (ADR-0013, ADR-0014, 2026-10-16):
  - Backups, key recovery and health work per workspace (ADR-0014 §2, §4): each workspace has its own data key wrapped by the server master key

## Acceptance criteria

- [ ] AC-1: Backup, then key export, then restore on a fresh data directory with key import: provider and connector secrets decrypt and work (test). Restore without the key reports exactly which secrets are unreadable
- [ ] AC-2: Scheduled backups run, follow retention, and are audited (test with a short interval)
- [ ] AC-3: With a pending migration, a backup is taken before migrating. A migration forced to fail leaves the old schema intact and the server refuses to start with a clear message (upgrade-harness test, WP-0007)
- [ ] AC-4: `/api/ready` fails when the database is unreadable, when the scheduler has stalled, or when disk is low (test). The image `HEALTHCHECK` uses it
- [ ] AC-5: Desktop and Team Workspace behave the same for backup, key export and restore
- [ ] AC-6: The help articles cover backups, key recovery and upgrades

## Test plan

- New `desktop/core/tests/backup_key_recovery.rs`, `desktop/server/tests/readiness.rs`; extend the WP-0007 upgrade harness
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy; ADR-0010 migration conventions. ADR-0010 applies to any migration this work package adds. Not behind a feature flag: this is data safety (ADR-0012 §4).
- Split at refinement if needed: (a) key recovery and restore, (b) scheduled and pre-upgrade backups, (c) health, logs and the guide.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
