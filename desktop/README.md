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
                          products, opportunities, quotes, orders, invoices
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
  Opportunities, Quotes, Orders, Invoices**.
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
  and overdue invoices, quotes awaiting response, pipeline by stage, recent
  audit activity) and overdue-invoice auto-reclassification on load.
- Company duplicate-name and contact duplicate-email warnings (FR-COM-04 /
  FR-CON-05) surfaced before save, not blocking it.

## What's deferred to a later phase

The database schema already has tables for these so the migration doesn't
need to change shape later, but there is no service/command/UI layer yet:

- **Contracts** and **Tasks** (schema present, `contracts`/`tasks`/
  `task_links` tables exist and are FK-clean, no business logic yet)
- PDF generation and printing for quotes/orders/invoices
- CSV import/export
- Backup/restore (`.lanesra` package format)
- Reports beyond the dashboard's built-in KPIs
- Team Workspace / multi-user LAN mode (the schema's `workspace_id` and
  `operating_mode` columns are ready for it; Personal mode only for now)
- Windows installer signing/packaging (the Tauri bundle config targets
  `nsis`/`msi`, which need a Windows build host — see below)

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
               # integration tests (tests/lifecycle.rs: full company ->
               # opportunity -> quote -> order -> invoice -> payment flow,
               # foreign key enforcement, cross-company relationship
               # validation)
```

Producing the actual signed Windows `.exe`/`.msi` installer requires a
Windows build host (e.g. a `windows-latest` GitHub Actions runner) - that
CI workflow isn't set up yet.

## Verification performed this session

- `cargo test` (src-tauri): 14/14 passing, including a full lifecycle
  integration test asserting exact money math through every conversion step.
- `npm run build` (frontend): `tsc` + `vite build` both clean.
- Ran the actual compiled Tauri binary under Xvfb, drove the first-run
  wizard with sample data through the UI, and confirmed the dashboard,
  quotes list, and quote detail screens render real data from the SQLite
  backend with correct numbering (`Q-2026-000001` -> `SO-2026-000001` ->
  `INV-2026-000001`) and audit trail.
