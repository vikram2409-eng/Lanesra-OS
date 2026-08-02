# Lanesra OS Desktop

Local-first Windows desktop edition of Lanesra OS, built from
`Lanesra_OS_Desktop_Windows_Requirements_v1.1.docx`. This is the initial
foundation plus the full sales lifecycle vertical slice — see "What's here"
and "What's deferred" below before assuming a feature exists.

## Stack

- **UI**: React + TypeScript + Vite
- **Shell**: Tauri v2 (Rust)
- **Database**: SQLite via `rusqlite` (bundled, no system SQLite required),
  foreign keys enforced on every connection, WAL journaling
- **State/query**: TanStack Query
- **Auth**: local users, Argon2 password hashing

## Architecture

```
desktop/
  src/                  React frontend
    lib/                api.ts (typed invoke wrapper), types.ts, money.ts
    components/          AppShell, LineItemsEditor, StatusBadge
    features/            firstRun, auth, dashboard, companies, contacts,
                          products, opportunities, quotes, orders, invoices,
                          contracts, tasks
  src-tauri/
    src/
      db/                connection + versioned migration runner
      domain/            money (integer-cents arithmetic), numbering
                          (Appendix B sequences), errors, ids
      models/             serde structs mirrored by src/lib/types.ts
      repositories/        parameterized SQL, one module per entity
      services/             business rules, relationship validation,
                             conversions, audit logging
      commands/             thin #[tauri::command] wrappers over services
```

Layering follows the PRD's component boundaries (11.2): presentation ->
application services -> domain -> repositories -> SQLite. Commands never
touch the database directly; they call services, which call repositories.

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
  actually exist. Today / Upcoming / Overdue / Completed / My Tasks / By
  Related Record views (FR-TSK-05; "By Owner" is scoped to the current user
  since there is no user-directory UI yet - see deferred items below).

## What's deferred to a later phase

The database schema already has tables for these so the migration doesn't
need to change shape later, but there is no service/command/UI layer yet:

- PDF generation and printing for quotes/orders/invoices
- CSV import/export
- Backup/restore (`.lanesra` package format)
- Reports beyond the dashboard's built-in KPIs
- Windows notifications for task reminders (FR-TSK-06)
- User management UI (inviting/listing other local users and roles beyond
  the first-run administrator - `users`/`roles`/`user_roles` tables and the
  repository layer exist, but there's no command/UI to manage them yet,
  which is also why the Tasks "By Owner" view is just "My Tasks" for now)
- Team Workspace / multi-user LAN mode (the schema's `workspace_id` and
  `operating_mode` columns are ready for it; Personal mode only for now)
- Windows installer signing/packaging (the Tauri bundle config targets
  `nsis`/`msi`, which need a Windows build host - see below; a GitHub
  Actions workflow at `.github/workflows/desktop-release.yml` now builds
  and drafts a release automatically on a `desktop-v*` tag push)

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
               # validation) and tests/contracts_and_tasks.rs (contract
               # numbering/renewal alerts, task relationship validation
               # and open/overdue counts)
```

Producing the actual signed Windows `.exe`/`.msi` installer requires a
Windows build host. `.github/workflows/desktop-release.yml` builds it on a
`windows-latest` GitHub Actions runner and attaches it to a draft GitHub
Release - push a `desktop-v*` tag, or run the workflow manually, to produce
one. It isn't code-signed yet.

## Verification performed this session

- `cargo test` (src-tauri): 23/23 passing, including a full lifecycle
  integration test asserting exact money math through every conversion step,
  plus dedicated Contracts/Tasks tests (renewal alert windows, relationship
  validation, open/overdue counts).
- `npm run build` (frontend): `tsc` + `vite build` both clean.
- Ran the actual compiled Tauri binary under Xvfb end to end: drove the
  first-run wizard with sample data through the real UI, and confirmed the
  dashboard (including the new renewal-alert and task-count KPIs),
  Contracts list (with the "Renewing soon" badge), and Tasks screens
  (Overdue tab correctly resolving "Opportunity: Acme Q3 Advisory
  Engagement", and the New Task form's relationship-type-filtered picker)
  all render real data from the SQLite backend with correct numbering
  (`Q-2026-000001` -> `SO-2026-000001` -> `INV-2026-000001`,
  `CTR-2026-000001`, `TSK-000001`...`TSK-000004`) and audit trail.
