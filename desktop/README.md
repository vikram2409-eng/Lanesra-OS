# Lanesra OS Desktop

Local-first Windows desktop edition of Lanesra OS, built from
`Lanesra_OS_Desktop_Windows_Requirements_v1.1.docx`. This now covers both
of the PRD's operating modes - **Personal Workspace** (the Tauri desktop
app, single user) and **Team Workspace** (one shared instance on a
network host, multiple authenticated users in their own browsers) - built
from the same domain logic. See "What's here" and "What's deferred" below
before assuming a feature exists.

## Stack

- **UI**: React + TypeScript + Vite - the same frontend runs unmodified in
  both operating modes (see "Two operating modes" below)
- **Personal Workspace shell**: Tauri v2 (Rust)
- **Team Workspace shell**: axum (Rust), cookie-based sessions, Docker
- **Database**: SQLite via `rusqlite` (bundled, no system SQLite required),
  foreign keys enforced on every connection, WAL journaling
- **State/query**: TanStack Query
- **Auth**: local users, Argon2 password hashing

## Architecture

This is a Cargo workspace of three crates, so the two operating modes
share one implementation of every business rule and can never drift apart:

```
desktop/
  Cargo.toml            workspace manifest (core, src-tauri, server)
  src/                  React frontend - unchanged between operating modes
    lib/                api.ts (dual transport: Tauri invoke or HTTP fetch,
                         see "Two operating modes"), types.ts, money.ts
    components/          AppShell, LineItemsEditor, StatusBadge
    features/            firstRun, auth, dashboard, companies, contacts,
                          products, opportunities, quotes, orders, invoices,
                          contracts, tasks, users
  core/                 lanesra-core: all business logic, no Tauri
                         dependency, shared by both shells below
    src/
      db/                connection + versioned migration runner
      domain/            money (integer-cents arithmetic), numbering
                          (Appendix B sequences), errors, ids
      models/             serde structs mirrored by src/lib/types.ts
      repositories/        parameterized SQL, one module per entity
      services/             business rules, relationship validation,
                             conversions, audit logging
    tests/                 lifecycle.rs, contracts_and_tasks.rs,
                            user_management.rs
  src-tauri/            Personal Workspace shell (desktop app)
    src/
      commands/            thin #[tauri::command] wrappers over
                            lanesra_core::services, single in-process
                            session (state.rs)
  server/               Team Workspace shell (HTTP server)
    src/
      dispatch.rs          mirrors every Tauri command 1:1 against the
                            same lanesra_core::services calls
      routes.rs            axum router: cookie sessions (web_sessions
                            table), auth gate, static frontend serving
      session.rs           HttpOnly session cookie helpers
    tests/http.rs           login/session/authorization tests over real HTTP
```

Layering follows the PRD's component boundaries (11.2): presentation ->
application services -> domain -> repositories -> SQLite. Neither shell's
commands/routes touch the database directly; they call `lanesra_core`
services, which call its repositories.

## Two operating modes

**Personal Workspace** (`src-tauri`): the Tauri desktop app described in
the rest of this README below "Running it". One process, one implicit
session held in memory (`state.rs`) - matches the PRD's "single user /
shared PC" model.

**Team Workspace** (`server`): an axum HTTP server that serves the same
built frontend and exposes every business operation at
`POST /api/invoke/<command>`, matching each Tauri command's name and
argument shape exactly (`server/src/dispatch.rs` is a line-for-line mirror
of `src-tauri/src/commands/*.rs`). Multiple team members open the same
`http://<host>:<port>` in their own browser and authenticate independently
via an `HttpOnly` session cookie backed by a `web_sessions` table - logging
one session out never affects another (see `server/tests/http.rs`). Every
command except `workspace_status`/`first_run_setup`/`login`/`logout`/
`current_user` requires a valid session; unlike the desktop app (where the
OS process boundary is the trust boundary), this is enforced by the server
itself, since anyone on the network can reach the port.

The frontend doesn't know which mode it's running in beyond one runtime
check: `src/lib/api.ts` calls Tauri's `invoke()` when `window.__TAURI_INTERNALS__`
exists, and otherwise `fetch()`s the equivalent `/api/invoke/...` endpoint
with `credentials: "include"`. Every feature screen is unchanged between
the two modes.

### Running Team Workspace locally

```bash
cd server
LANESRA_DATA_DIR=./data LANESRA_FRONTEND_DIR=../dist cargo run
# then open http://localhost:8080 - first-run wizard creates the workspace
```

(`../dist` must exist - run `npm run build` in `desktop/` first.)

### Running it with Docker (recommended for a small team)

```bash
cd desktop
docker build -t lanesra-os-server .
docker run -p 8080:8080 -v lanesra-data:/data lanesra-os-server
# team members open http://<this-machine's-LAN-IP>:8080
```

Or with Compose: `docker compose up -d`. The named volume
(`lanesra-data`) is where the SQLite database lives - back that volume up
the same way you'd back up the desktop app's database file. This has been
built and run end-to-end in this session (first-run, sample data seeding,
login, two independent concurrent sessions, and data persisting across a
container restart all verified against the real image - see "Verification
performed this session").

This targets the PRD's explicit scope: a local network, not the public
internet. There's no HTTPS/TLS termination built in - put it behind a
reverse proxy with TLS if you need that, or keep it LAN-only as intended.

## What's here

- Full SQLite schema (Appendix C, all 20 tables) with foreign keys, check
  constraints and indexes.
- Complete CRUD + business rules for **Companies, Contacts, Products,
  Opportunities, Quotes, Orders, Invoices, Contracts, Tasks**.
- The full flexible sales lifecycle (6.1): Company -> Opportunity -> Quote
  -> Order -> Invoice, *and* the direct-quote / direct-order / direct-invoice
  shortcuts, all in one app.
- Quote -> Order and Order -> Invoice conversion that copies line items
  without mutating or deleting the source document (FR-QUO-06/FR-ORD-07).
- Atomic, gap-free business document numbering (Appendix B) via a single
  `INSERT ... ON CONFLICT ... RETURNING` statement per allocation.
- Decimal-safe money: every persisted amount is integer cents, quantities
  are integers scaled by 1000, rates are basis points (BR-014). No floats
  touch persisted totals.
- Local user auth (Argon2), roles table seeded with the five MVP roles,
  audit trail on create/update/archive/status-change/conversion/payment/login.
- First-run wizard (business profile + local administrator + optional
  sample data covering both the managed-opportunity and direct-quote paths).
- Dashboard with real KPI queries (open pipeline, won revenue, outstanding
  and overdue invoices, quotes awaiting response, 30/60/90-day contract
  renewal alerts, open/overdue task counts, pipeline by stage, recent audit
  activity) and overdue-invoice auto-reclassification on load.
- Company duplicate-name and contact duplicate-email warnings (FR-COM-04 /
  FR-CON-05) surfaced before save, not blocking it.
- **Contracts**: company required, contact and source quote optional,
  deliberately has no opportunity relationship at the type level
  (FR-CTR-03/BR-009 - `ContractInput` simply has no such field), 30/60/90-day
  renewal alert counts (FR-CTR-05).
- **Tasks**: General or linked to exactly one of Company / Contact /
  Opportunity / Quote / Order / Invoice / Contract (FR-TSK-02), with the
  related-record picker filtered to that type (FR-TSK-03) and validated to
  actually exist. Today / Upcoming / Overdue / Completed / By Owner (grouped
  using the real user directory) / By Related Record views (FR-TSK-05).
- **User management**: an Administrator can list, create, edit
  roles/display name/active status, and reset the password for other local
  users from the Users screen (nav item is admin-only). A safety guard
  blocks demoting or deactivating the last active Administrator so a
  workspace can never lock itself out. Any authenticated user can list the
  directory (needed to assign task owners) but only an Administrator can
  create/edit/deactivate accounts or reset passwords.
- **Team Workspace**: a `lanesra-server` axum binary (crate `server/`,
  Dockerfile + `docker-compose.yml` at the `desktop/` root) that serves the
  same frontend and business logic over plain HTTP for a small team on one
  network host, with per-user `HttpOnly` session cookies, a server-side
  auth gate on every business command, and a persistent SQLite volume. See
  "Two operating modes" above.
- **Backup & restore**: an Administrator can export the entire workspace as
  a single `.lanesra` file (a zip of a live SQLite snapshot, taken via
  SQLite's online backup API, plus a manifest with the schema/app version)
  from the Users screen, and restore one to wholesale-replace the current
  workspace. Restore stages and fully validates the uploaded file on disk
  - including rejecting one made by a newer app version than the one
    running - before it ever touches the live database, and works
  identically in Personal Workspace and Team Workspace (where restoring
  safely swaps the shared connection out from under any requests in
  flight; see `core/src/services/backup_service.rs`).
- **Self-service password change**: any authenticated user can change
  their own password (proving they know the current one) from a "My
  account" screen reachable by clicking their name in the top bar -
  previously only an Administrator could reset a password, from the Users
  screen, which is still true for resetting *someone else's*.
- **PDF generation and printing**: quotes, orders and invoices each have a
  "Print / Save as PDF" button that opens a full-page print preview
  (business letterhead, Bill To company/contact, dates, line items,
  totals - plus paid/balance-due for invoices) built from the same data
  already on screen, then hands off to the browser's native print dialog
  (`window.print()`) so "Save as PDF" is just the OS/browser print target.
  No PDF-rendering crate or server-side work - a `@media print` rule hides
  everything else on the page, so it works identically in the Tauri
  webview and a Team Workspace browser tab. See
  `src/components/PrintableDocument.tsx` and `PrintOverlay.tsx`.

## What's deferred to a later phase

The database schema already has tables for these so the migration doesn't
need to change shape later, but there is no service/command/UI layer yet:

- CSV import/export
- Reports beyond the dashboard's built-in KPIs
- Windows notifications for task reminders (FR-TSK-06)
- Windows installer signing/packaging (the Tauri bundle config targets
  `nsis`/`msi`, which need a Windows build host - see below; a GitHub
  Actions workflow at `.github/workflows/desktop-release.yml` now builds
  and drafts a release automatically on a `desktop-v*` tag push - the
  v0.1.0 Early Access build is already published, just unsigned)

## Running it

Requires Node.js and a Rust toolchain (`rustup`). On Linux, Tauri also
needs `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`,
and `librsvg2-dev` (see the [Tauri prerequisites docs](https://v2.tauri.app/start/prerequisites/)
for your platform).

```bash
npm install
npm run tauri dev     # launches the app with hot reload
```

To type-check and build the frontend only:

```bash
npm run build
```

To build/test the Rust backend on its own:

```bash
cd src-tauri
cargo test     # unit tests (money, numbering, migrations, auth) +
               # integration tests: tests/lifecycle.rs (full company ->
               # opportunity -> quote -> order -> invoice -> payment flow,
               # foreign key enforcement, cross-company relationship
               # validation), tests/contracts_and_tasks.rs (contract
               # numbering/renewal alerts, task relationship validation
               # and open/overdue counts), and tests/user_management.rs
               # (admin-only authorization, last-administrator guard,
               # password reset)
```

Producing the actual signed Windows `.exe`/`.msi` installer requires a
Windows build host. `.github/workflows/desktop-release.yml` builds it on a
`windows-latest` GitHub Actions runner and attaches it to a draft GitHub
Release - push a `desktop-v*` tag, or run the workflow manually, to produce
one. It isn't code-signed yet.

## Verification performed this session

- `cargo test` (src-tauri): 30/30 passing, including a full lifecycle
  integration test asserting exact money math through every conversion step,
  dedicated Contracts/Tasks tests (renewal alert windows, relationship
  validation, open/overdue counts), and user-management tests (non-admin
  rejection, last-administrator guard, password reset).
- `npm run build` (frontend): `tsc` + `vite build` both clean.
- Ran the actual compiled Tauri binary under Xvfb end to end twice this
  project: confirmed the dashboard (including renewal-alert and task-count
  KPIs), Contracts list (with the "Renewing soon" badge), Tasks screens
  (Overdue tab resolving related-record labels, the relationship-filtered
  picker, and the By Owner view grouping real tasks under "CoTest Admi" and
  the seeded "Morgan Reyes" sample user), and the Users screen (listing both
  seeded accounts with correct roles/status) all render real data from the
  SQLite backend with correct numbering and audit trail.

**Team Workspace phase:**

- `cargo test` across the whole workspace (`core`, `src-tauri`, `server`):
  39/39 passing after splitting the business logic into its own
  `lanesra-core` crate - the original 30 (now living in `core/`) plus 5 new
  HTTP integration tests in `server/tests/http.rs`
  (`health_check_responds`, `unauthenticated_requests_are_rejected`,
  `login_grants_a_session_that_can_read_data`, `two_sessions_are_independent`,
  `only_an_administrator_can_create_users`) and 4 new session-repository
  unit tests.
- Manual curl testing against the running `lanesra-server` binary caught a
  real security gap before it shipped: the first HTTP routing draft let
  unauthenticated requests call business commands directly. Fixed by adding
  a server-side auth gate in `routes.rs` that every command except
  `workspace_status`/`first_run_setup`/`login`/`logout`/`current_user` must
  pass; re-verified with curl that anonymous requests are now rejected
  while logged-in requests still work.
- Built the real Docker image (multi-stage: Node frontend build, Rust
  server build, slim Debian runtime) and ran it end to end: first-run
  wizard over plain HTTP, sample data seeding, login, two independent
  concurrent browser sessions (confirmed via cookies that logging one out
  doesn't affect the other), and data surviving a full container
  stop/restart against the same named volume.
- Re-ran the Tauri desktop app under Xvfb after the `core` crate
  extraction to confirm the Personal Workspace shell still builds and
  renders correctly (FirstRun screen) with zero behavior change from
  moving its business logic into a shared crate.

**Backup/Restore + self-service password phase:**

- `cargo test` across the whole workspace: 53/53 passing - the prior 39
  plus 6 new core tests (`core/tests/backup_and_restore.rs`: round-trip
  backup/restore, non-administrator rejection, garbage-input rejection,
  future-schema-version rejection, wrong-current-password rejection,
  too-short-new-password rejection) and 8 HTTP integration tests (2
  pre-existing plus `backup_then_restore_reverts_data_over_http`,
  `only_an_administrator_can_restore_a_backup`,
  `self_service_password_change_over_http`).
- `npm run build`: `tsc` + `vite build` both clean.
- Built the real release `lanesra-server` binary and drove the whole
  feature by hand with curl against a live, file-backed SQLite database
  (not `:memory:`): first-run, created a company, exported a backup,
  created a second company, restored the backup, and confirmed via
  `list_companies` that only the pre-backup company remained - and that
  the admin's own session (captured inside the backup snapshot) still
  resolved correctly afterward, since the connection swap happens without
  touching `web_sessions`. Also exercised wrong-current-password rejection
  and a successful self-service password change, then logged in with the
  new password. Restarted the server process entirely afterward and
  confirmed the restored data was still there and the on-disk directory
  held one clean `lanesra.sqlite3` (+ WAL/SHM) with no leftover
  `.restoring` temp file from the swap.

**PDF printing phase:**

- `npm run build`: `tsc` + `vite build` both clean (104 modules, no new
  Rust code this phase - printing is entirely client-side).
- Built the real release `lanesra-server` binary, seeded it via curl with
  a workspace ("Northstar Digital Solutions"), a company with a billing
  address, a contact, and a quote with a taxed line item, then drove it
  end to end with a real Chromium browser (Playwright): logged in, opened
  the seeded quote, and screenshotted the detail view showing the new
  "Print / Save as PDF" button next to the existing status actions.
  Clicked it and screenshotted the resulting print preview, confirming it
  renders the business name and legal name, "Bill to" with the company's
  address and the contact's name/email, issue date and valid-until date,
  the line item table matching the on-screen figures exactly, the
  subtotal/discount/tax/total breakdown, and the quote's notes - all from
  one `PrintableDocument` component shared by quotes, orders and
  invoices.
