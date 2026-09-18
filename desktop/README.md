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

This targets the PRD's original scope: a local network. There's still no
HTTPS/TLS termination built into `lanesra-server` itself - that's what the
reverse proxy in the next section is for - but the server now knows how to
behave correctly *behind* one, so an organization can expose Team
Workspace on the open internet on its own domain/infrastructure if it
wants to (still self-hosted, never a Lanesra-run SaaS).

### Exposing it on the open internet (optional)

Everything in this section is opt-in via environment variables; leave
them unset and the server behaves exactly as it always has (LAN-only,
plain HTTP, same-origin frontend). Three things change when you expose
Team Workspace beyond a trusted LAN:

1. **TLS termination.** `lanesra-server` speaks plain HTTP only - put a
   reverse proxy in front of it that terminates TLS and forwards to
   `127.0.0.1:8080` (or wherever the container is bound). Minimal
   examples for the two most common choices:

   **Caddy** (`Caddyfile`, automatic HTTPS via Let's Encrypt):
   ```
   lanesra.example.com {
       reverse_proxy 127.0.0.1:8080
   }
   ```

   **nginx** (assumes a cert already provisioned, e.g. via certbot):
   ```nginx
   server {
       listen 443 ssl;
       server_name lanesra.example.com;
       ssl_certificate     /etc/letsencrypt/live/lanesra.example.com/fullchain.pem;
       ssl_certificate_key /etc/letsencrypt/live/lanesra.example.com/privkey.pem;

       location / {
           proxy_pass http://127.0.0.1:8080;
           proxy_set_header Host $host;
           proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
           proxy_set_header X-Forwarded-Proto $scheme;
       }
   }
   server {
       listen 80;
       server_name lanesra.example.com;
       return 301 https://$host$request_uri;
   }
   ```

2. **`LANESRA_TRUST_PROXY_HTTPS=1`** - tells the server to mark the
   session cookie `Secure` and send `Strict-Transport-Security`. Only set
   this once a proxy like the above is actually terminating TLS in front
   of the server - a `Secure` cookie handed to a browser that only ever
   sees the plain-HTTP hop between it and the proxy is simply never sent
   back, which silently breaks every session rather than failing loudly.
   Leave it unset for a LAN-only, plain-HTTP deployment.

3. **`LANESRA_ALLOWED_ORIGINS`** (optional, comma-separated) - only
   needed if the frontend is served from a different origin than this
   API (e.g. a separate static-hosting deployment). `lanesra-server`
   serves its own frontend same-origin by default, so this is unset in
   the normal case; setting it enables a CORS layer that allows the
   listed origins to call the API with credentials (the session cookie)
   included.

Every response also now carries a conservative, always-on header set
(`X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
`Referrer-Policy: strict-origin-when-cross-origin`) regardless of any of
the above - see `server/src/security.rs`.

With Docker, pass these through `-e` or Compose's `environment:` (see
`docker-compose.yml`), e.g.:
```bash
docker run -p 8080:8080 -v lanesra-data:/data \
  -e LANESRA_TRUST_PROXY_HTTPS=1 \
  lanesra-os-server
```

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
- **CSV export**: every list screen (Companies, Contacts, Products,
  Sales Pipeline, Quotes, Orders, Invoices, Contracts, Tasks) has an
  "Export CSV" button that downloads the currently-loaded rows with a
  UTF-8 BOM and CRLF line endings (so it opens cleanly in Excel), built
  client-side from data already on screen - no new Rust code.
- **CSV import**: Companies and Contacts - the two records a new
  workspace most needs to bulk-load - have an "Import CSV" button that
  parses a chosen file, previews each row's validity (missing required
  fields, an unrecognized status, or for contacts an unmatched company
  name all fail that row with a reason shown before you commit), then
  imports row by row through the exact same `create_company`/
  `create_contact` command the manual "New ..." form uses, so a bulk
  import can never bypass business rules like duplicate-name detection.
  One bad row doesn't block the rest, and the after-import summary
  shows created/failed/skipped per row. See
  `src/components/CsvImportDialog.tsx`, `src/lib/csv.ts`.
- **Branding & print customization (FR-BRD)**: an Administrator can edit
  the workspace profile - business name, legal name, business address,
  currency/locale/timezone/tax defaults - at any time from a new Settings
  screen, not just once at first-run like before. They can also upload a
  logo (resized to 240px and re-encoded as PNG client-side before upload,
  validated again server-side for mime type and a 256KB size cap) or
  remove it. Both the address and the logo now render on the print
  letterhead for quotes/orders/invoices alongside the business name. See
  `core/src/services/workspace_service.rs`,
  `src/features/settings/Settings.tsx`.
- **Reports (FR-RPT)**: a Reports screen beyond the dashboard's fixed KPI
  tiles - Revenue by month, Win rate by owner, Lost reasons, AR aging,
  and Sales by owner, each with a date-range filter (AR aging uses an
  "as of" date instead) and an Export CSV button reusing the existing
  CSV helper. A fixed gallery of named reports, not a query builder -
  see "Reports (Phase 1)" in the product backlog for why. "Win rate by
  stage" from the original brainstorm was adapted to "by owner" plus a
  separate lost-reasons breakdown, since stage and status both
  terminate at Won/Lost for a closed opportunity in this schema, so
  grouping by stage doesn't produce a meaningful split - see the doc
  comment on `models::report::WinRateByOwner`. Invoices/orders have no
  owner of their own, so Sales by owner attributes revenue via the
  invoice's Company owner. See `core/src/services/report_service.rs`,
  `src/features/reports/Reports.tsx`.
- **Custom fields (FR-CFG)**: an Administrator can define custom fields on
  Companies and Contacts from the new Settings screen (Phase 1; see
  "Admin flexibility" below for the later phase that generalized this to
  every major entity) - label, type
  (text/number/date/yes-no/select-with-options), required, and
  active/inactive - without a code change or a schema migration per
  field. An auto-generated key is uniquified rather than rejected on a
  duplicate label. Active fields render on the record's create/edit form
  and are enforced both client-side (HTML5 `required`, immediate
  feedback) and server-side inside `set_custom_field_values`
  (required-field and select-option validation, verified independently
  of the UI with a direct API call in this phase's testing) - a
  client-only check would be trivially bypassed by any direct API
  caller. Custom entity types (letting an admin define a whole new record
  type, not just fields on an existing one) remains out of scope; see
  "Custom entities" in the product backlog. Deferred within this
  phase: showing custom field values as extra columns on list screens
  (`show_in_list` is stored and editable in the admin screen, just not
  yet rendered anywhere), and including custom fields in CSV
  import/export. See
  `core/src/services/custom_field_service.rs`,
  `src/components/CustomFieldsSection.tsx`,
  `src/features/settings/CustomFieldsAdmin.tsx`.
- **Conditional business rules (FR-RUL)**: an Administrator can make a
  custom field required, or hide it entirely, based on the entity's
  built-in status (e.g. "require Lead Source when Status = Prospect") from
  a new Business rules panel in Settings, right below Custom fields. A
  rule's trigger is either the built-in `status` field or another custom
  field, compared with `equals`/`not_equals`; its target is always a
  custom field. When two active rules target the same field, the one with
  the higher `sort_order` wins - implemented as a single map-insert per
  rule in ascending sort order, so a later insert simply overwrites an
  earlier one for that target (`field_rule_service::effects_for`). A
  hidden field is never required even if it's statically flagged
  `required`, since there's nothing to validate on a field the user can't
  see. Only `require` is enforced server-side inside
  `custom_field_service::set_entity_values` - `hide` is UX-only, since a
  hidden field has no value to reject. The client mirrors the same
  evaluation logic (`src/lib/fieldRules.ts`) purely for live form
  feedback (graying out the asterisk, hiding the field as you change
  Status) - the server re-validates independently on save regardless, and
  this phase's testing proves that by calling `set_custom_field_values`
  directly, bypassing the form entirely. See
  `core/src/services/field_rule_service.rs`,
  `src/features/settings/FieldRulesAdmin.tsx`,
  `src/lib/fieldRules.ts`.
- **Workflow automation, Phase 1 (FR-WFL)**: an Administrator can
  auto-create a follow-up Task the moment an Opportunity's stage or an
  Invoice's status transitions into a chosen value - e.g. "when stage
  reaches Won, create task 'Send onboarding kit', due in 3 days, assigned
  to the record's owner" - from a new Workflow automation panel in
  Settings, right below Business rules. Unlike FR-RUL's field rules, this
  is purely additive: there's no "highest wins" conflict, since every
  active rule that matches the new value fires and creates its own task,
  so two rules on the same trigger create two tasks. "The record's owner"
  resolves to `Opportunity.owner_user_id`, or - since an Invoice has no
  owner of its own - its Company's `owner_user_id`, the same attribution
  `report_service::sales_by_owner` already uses; a rule can also name a
  specific assignee instead. Firing is a no-op unless the old and new
  values actually differ, so re-saving a record without changing its
  stage/status can never re-fire an already-fired rule. Wired into the one
  choke point each entity already had for status changes -
  `opportunity_service::update` (comparing stage before/after) and every
  path that changes an invoice's status: `issue`/`void` (via `set_status`),
  `record_payment`'s automatic Paid/Partially Paid transition, and
  `refresh_overdue`. The created Task is linked back via
  `related_type`/`related_id`, so it shows up wherever tasks for that
  record already do (e.g. the Tasks screen's "By Related Record" view).
  Entirely server-side - there is nothing for the client to evaluate live,
  since (unlike FR-RUL) nothing here changes what a form looks like.
  Deliberately scoped to one action (task creation) for Phase 1; see
  "Workflow automation" in the product backlog for the fuller brainstorm
  this slice was cut from. See `core/src/services/workflow_service.rs`,
  `src/features/settings/WorkflowRulesAdmin.tsx`.
- **Admin flexibility - generalized to every major entity**: custom
  fields (FR-CFG) and business rules (FR-RUL), originally Company/Contact
  only, and workflow automation (FR-WFL), originally Opportunity/Invoice
  only, now all work identically across Company, Contact, Opportunity,
  Quote, Order, Invoice, Contract and Task (custom fields also cover
  Product, which has no status/stage field to trigger a rule or workflow
  on). Each entity's one built-in "trigger" field is `status` for all of
  them except Product (`is_active`, compared as `"true"`/`"false"`) -
  `field_rule::builtin_trigger_field_for` in the core, mirrored in the
  frontend as `builtinTriggerFieldFor`. Workflow automation keeps
  Opportunity's existing special case of triggering on `stage` rather
  than `status` (`workflow_rule::transition_field_for`), since that's the
  field that actually flows through the sales pipeline. The entity_type
  CHECK constraints from the original migrations were dropped (SQLite
  can't ALTER one in place) in favor of the application layer, which was
  already the real source of truth. Every entity's create/edit form
  (or, for Quote/Order/Invoice, which have no full edit form, a "Custom
  fields" card on their detail view) now renders `CustomFieldsSection` /
  `CustomFieldsCard`. See `core/src/db/migrations/0007_broaden_entity_types.sql`.
- **Admin panel navigation**: Users moved from its own sidebar item into
  a tabbed "Admin" panel alongside Business profile, Custom fields,
  Business rules, Workflow automation, Numbering and Dashboard KPIs - one
  nav entry for every administrator-facing capability instead of two.
  See `src/features/settings/Settings.tsx` (`AdminPanel`).
- **Business phone number**: alongside the existing address/logo
  branding fields, shown on the print letterhead when set.
- **Configurable ID/numbering format**: an Administrator can change the
  prefix and zero-padded digit width used for any entity's
  auto-generated number (e.g. "CUS-000001" → "ACC-000001", or
  "ACC-ab0001" - the letters are just part of the chosen prefix text,
  there's no separate alpha-segment syntax). Changing the format never
  resets or renumbers already-issued numbers: the underlying sequence in
  `number_sequences` is untouched, only the formatting changes going
  forward - proven in this phase's testing by issuing a number, changing
  the prefix, and confirming the *next* number continues the same
  sequence with the new prefix. `domain::numbering::allocate_number`
  looks up an optional override directly (the same way it already talks
  straight to `number_sequences` without a repository); the admin CRUD
  layer lives in `numbering_service.rs`.
- **Simple report builder**: an Administrator picks an entity, a field to
  group by (the entity's built-in status/stage, or any active custom
  field), and an aggregate - count of records, or sum of a numeric custom
  field - and saves it as a named custom report any user can run from
  Reports → Custom reports. Deliberately not a full drag-and-drop
  builder; see "Report builder" in the product backlog for the fuller
  version this was scoped down from. See
  `core/src/services/custom_report_service.rs`.
- **Dashboard KPI picker**: an Administrator can choose which of the 7
  Dashboard KPI tiles show, from Admin → Dashboard KPIs. Reordering isn't
  exposed yet - the stored preference (`Workspace.dashboard_kpi_prefs`,
  JSON) is already an ordered list, so that's addable later without a
  schema change. See `src/features/dashboard/kpis.tsx`.
- **Custom objects (admin extensibility, spec §20.2)**: an Administrator
  can define a whole new business object at runtime - "Vendors", "Assets",
  "Projects" - from Admin → Custom Objects: singular/plural name, icon,
  and a record-number prefix/digit width. The object immediately gets its
  own sidebar section with generic list/create/edit screens
  (`CustomObjectRecords.tsx`), and - critically - its custom fields,
  business rules and custom reports all work through the *exact same*
  subsystems every built-in entity already uses, unmodified: a custom
  object's `key` (a lowercase slug) is simply one more `entity_type`
  string those subsystems accept. The only new integration point is
  `custom_object_service::is_valid_dynamic_entity_type`, a single shared
  check that `custom_field_service` and `custom_report_service` call
  instead of matching against the fixed `CUSTOM_FIELD_ENTITY_TYPES` list.
  Records are stored generically in a `custom_records` table (one row per
  record of any custom object, `object_key` + `display_number` +
  `primary_name` + a fixed `Active/Inactive/Archived` status) rather than
  a table per object type. Deleting an object definition is blocked while
  any of its records exist (archive them, or deactivate the object
  instead - always non-destructive); deactivating hides it from
  navigation and new-record creation but keeps everything intact.
  Workflow automation on custom objects is deferred to the next phase
  (its task-creation action would need `tasks.related_type`'s CHECK
  constraint broadened, which that phase is doing anyway for its own
  reasons - see below). See `core/src/services/custom_object_service.rs`,
  `custom_record_service.rs`, and `core/tests/custom_objects.rs`, whose
  last test composes a custom field + a business rule + the report
  builder together on a custom object, mirroring how
  `admin_flexibility.rs` proves the same composition on a built-in entity.
- **Custom relationships (admin extensibility, spec §20.3/§21)**: an
  Administrator can connect any two object types - built-in to built-in,
  built-in to custom, or custom to custom - from Admin → Relationships:
  many-to-one, one-to-one or many-to-many, with forward/reverse labels, a
  "show as related list" toggle, and a delete behavior (`restrict` blocks
  archiving a linked record; `archive` silently drops the link instead).
  A record's detail page (Company, and every custom object record) renders
  every relationship it participates in via one generic
  `RelatedRecordsCard` component with zero per-entity wiring - the same
  "compose for free" property custom fields/business rules/reports already
  have on a custom object. `entity_registry` (a lookup-by-entity_type
  resolver, not a physical `record_registry` table as the spec suggests -
  see that module's doc comment for why) is the shared seam this and Phase
  C/D's engines all resolve "any record of any entity_type" through. See
  `core/src/services/relationship_service.rs`, `entity_registry.rs`,
  `src/components/RelatedRecordsCard.tsx`.
- **Richer business rules (spec §22/ADM-BR)**: replaces the original
  single-condition/require-or-hide field_rules engine with a no-code IF
  (AND/OR conditions) / THEN (multiple actions) rule builder - ten
  operators (equals/contains/is_empty/greater_than/on_or_after and their
  complements, not just equals/not_equals), and actions beyond
  require/hide: lock (read-only), set a default or forced value, block
  the whole save with a custom message, or show a non-blocking message.
  Rule priority and an optional effective date window are supported; a
  Test Rule panel evaluates a hypothetical record against every active
  rule without persisting anything. Still enforced at the one integration
  point every entity's save flow already calls unconditionally
  (`custom_field_service::set_entity_values`), so no per-entity wiring was
  needed. See `core/src/services/business_rule_service.rs`,
  `domain/conditions.rs` (the shared AND/OR matcher Phase D's workflow
  engine reuses), `src/features/settings/BusinessRulesAdmin.tsx`.
- **Richer workflow automation (spec §23/ADM-WF)**: replaces the original
  single-trigger (status transition) / single-action (create task) engine
  with seven trigger types (record created/updated, status changed, a
  custom field changed, a date reached, overdue, or a recurring schedule),
  the same AND/OR conditions business rules use, and six actions: create
  task, update a custom field, assign owner, create a related record
  (through a Phase B relationship), send an in-app notification, or create
  a reminder. Every run is logged (trigger, actions, outcome, failure
  reason) to `workflow_runs`, which doubles as the fire-once dedup source
  for date-based triggers. A thread-local recursion depth guard
  (`workflow_service::WORKFLOW_DEPTH`) bounds self-referential workflow
  chains (e.g. a workflow that creates a related record whose own
  creation triggers another workflow) without threading a depth parameter
  through every entity service. date_reached/due_overdue/scheduled
  triggers run via a periodic scan the frontend polls every 5 minutes
  (Personal Workspace has no OS-level background scheduler - see that
  function's doc comment). Workflows can now trigger on and act on custom
  objects; `task_links.related_type` is no longer CHECK-constrained to the
  original seven built-in types for the same reason. Includes a minimal
  in-app notification center (bell icon, unread count, mark read/all). See
  `core/src/services/workflow_service.rs`,
  `src/features/settings/WorkflowAutomationAdmin.tsx`,
  `src/components/NotificationBell.tsx`.
- **Business rules and workflow automation on any built-in field**: both
  engines previously only let you condition/trigger on one hardcoded
  built-in field per entity (`status`, or `stage` for Opportunity) and
  could only ever write to custom fields. A new
  `core::domain::builtin_fields` registry lists every safely-targetable
  built-in field per entity (name, tax number, addresses, opportunity
  value/probability/close date, etc. - excludes relationship/FK fields,
  computed columns like `*_number` and document totals, and
  `owner_user_id`, which has its own `assign_owner` action), and a new
  `core::services::builtin_field_service` reads/writes them by routing
  every write back through the entity's own `*_service::update()` (so
  existing validation, audit, and workflow-firing all still run - no
  bypass). Conditions, workflow `field_changed` triggers, and both
  engines' `update_field`/`set_value` actions now carry a
  `field_source: "builtin" | "custom"` alongside the field key, so a rule
  can require "Tax number" be set, or a workflow can watch an
  Opportunity's `value` field changing and copy it into a related
  record - not just watch/set the status field. Quote/Order/Invoice have
  no actionable built-in fields, since (by original design) those
  entities have no general-purpose `update()` - they're immutable after
  creation aside from status/conversion methods - so their fields are
  conditionable but not writable, same as before. Recursion (a builtin
  field write re-triggering `field_changed`) is bounded by the same
  pre-existing `WORKFLOW_DEPTH` guard used for every other workflow
  recursion path. The 9 built-in entity forms don't yet show a live
  require/hide/lock preview for builtin-targeted rule actions the way
  `CustomFieldsSection` does for custom fields - consistent with
  `block_save`/`set_value`, which were already server-only with no live
  client preview; server-side enforcement is unaffected and tested. See
  `core/src/domain/builtin_fields.rs`,
  `core/src/services/builtin_field_service.rs`,
  `core/src/db/migrations/0016_builtin_field_targeting.sql`,
  `src/components/BuiltinValueInput.tsx`.
- **Richer conditions: more operators + field-to-field comparison (Admin
  Automation & Customization addendum, Phase 1)**: the shared condition
  engine gained four operators - `starts_with`/`ends_with`/`in_list`/
  `not_in_list` (the last two split on `|`, the same convention select
  options use) - and every condition (a business rule's, a workflow's
  `field_changed` trigger, or an "extra condition") can now compare a
  field against another field's live value instead of only a fixed
  literal, via a new `compare_field_source`/`compare_field_key` pair.
  Resolution happens once, in each engine's service layer, right before
  calling `domain::conditions::conditions_match` - the matcher itself
  never has to know whether a value came from a literal or another
  field. `domain::conditions::field_ref_is_valid` is now the single
  validator for both a condition's primary field and its optional
  compare-to field, in both business_rule_service and workflow_service.
  Also consolidated `CONDITION_OPERATORS`, which `models::business_rule`
  and `domain::conditions` used to each maintain as a separately
  hand-copied list of the same 10 operators, into one re-exported source
  of truth. See `core/src/domain/conditions.rs`,
  `core/src/db/migrations/0017_condition_field_comparison.sql`.
- **Custom field validation and capability flags (spec ADM-CF-04/05)**: a
  custom field can now have optional validation - min/max for `number`
  fields, max length and a regex pattern for `text` fields - checked both
  at definition time (sane range, valid regex) and at save time, plus
  searchable/filterable/reportable flags. Only `is_reportable` is wired to
  an actual behavior in this build (an unreportable field can't be used as
  a report group-by/sum target); `is_searchable`/`is_filterable` are
  stored as forward-looking metadata, since this build has no global
  search or list-view filtering to wire them into yet - see "What's
  deferred" below.
- **Windows task reminder notifications (FR-TSK-06)**: uses the standard
  Web Notification API rather than a native Tauri plugin - WebView2 (the
  webview Tauri v2 uses on Windows) surfaces it as a real Windows toast
  notification, so the same code works unmodified in Team Workspace's
  plain browser tab too, with no new native dependency. Polls every
  minute; already-fired reminders are tracked in `localStorage` so a
  reload doesn't repeat one. See `src/components/TaskReminderNotifier.tsx`.
- **Session inactivity auto-lock**: a fixed 15-minute idle timeout (not
  admin-configurable yet) shows a lock screen requiring the current user's
  password before the app is usable again - reuses the existing `login`
  command rather than a separate "unlock" concept. See
  `src/components/SessionLock.tsx`.
- **Global search** (spec §5.3/§9.3): a lightweight, case-insensitive
  substring search across every core entity's natural display fields, plus
  any custom field an admin has flagged `is_searchable` - not a full-text
  engine (no ranking/tokenization beyond which table a match came from);
  see `core/src/services/search_service.rs`.
- **Screen/App Builder**: named layouts (per entity, by role) with tabbed,
  multi-column field sections and Draft → Publish, driving both the
  create/edit form and the read-only detail view; custom-relationship
  related-lists can be placed on a specific tab.
- **Dashboard customization**: named dashboard layouts (KPI, chart and
  record-list widgets) assigned to roles, Draft → Publish, alongside the
  original fixed 7-tile KPI picker.
- **App Builder**: an Administrator groups any set of objects, their
  published screens, and one dashboard into a named, publishable app with
  its own icon, granted to roles or individual users as Viewer or Editor,
  enforced server-side on every write including status-lifecycle actions.
- **App Catalog** (11 industry apps): Field Service, Property Management,
  Construction & Contractors, Professional Services, Practice
  Administration, Recruitment & Staffing, Real Estate Brokerage, Legal
  Practice, Nonprofit & Association Management, Auto Repair & Service
  Garage, and Policy Administration & Claims Management - each a versioned
  package manifest (objects, fields,
  relationships, business rules, workflows, screens, reports, an optional
  dashboard) installed with pre-install validation, an automatic safety
  backup, and a transactional, rollback-safe install that reuses the
  workspace's existing Company/Contact/Task records instead of creating a
  parallel data model.
- **Lanesra Industry Foundation**: a twelfth, different-in-kind package -
  shared cross-industry plumbing (Party, Party Role, Party Relationship,
  Organization Unit, Location, Contact Point, External Identifier,
  Consent/Preference, Asset, Agreement, Service Case, Interaction,
  Document Record, Financial Transaction, Data Quality Issue) other
  packages can optionally declare a real, enforced dependency on
  (`IndustryPackageManifest::dependencies`, checked before install) and
  relate their own objects to, instead of each reinventing its own
  version. Not retrofitted onto the eleven business-vertical packages
  above in this pass - each keeps its own independent fields exactly as
  shipped.
- **Deployment Management**: a Publisher registry with enforced namespace
  validation, component-tagging across every admin-configurable artifact
  (a synthetic "Local Workspace" grouping shows everything built by hand),
  named/versioned Solutions curated one component at a time, real
  export/import between workspaces, and update-with-diff (a real per-key
  Added/Modified/Removed diff for objects/fields, a summarized
  added-count for everything else) before a package update is applied.
- **Integration Hub**: AES-256-GCM-encrypted Connections (REST, SFTP,
  PostgreSQL, OData, SMTP), OpenAPI 3.x Connectors - importable from a
  curated template gallery (OpenAI, Cohere, Slack, GitHub, SendGrid) or
  a hand-pasted spec - usable as a Workflow Automation action or, once an
  admin opts a connector in, as an AI Agent Foundry tool via the Tool
  Bridge - read-only by default, write access
  behind a further explicit per-connector opt-in, and fails closed rather
  than exposing a tool for an action whose request-body schema can't be
  confidently typed - a generic `/api/v1/objects/...` REST API secured by
  hashed/scoped API keys, HMAC-SHA256-signed Webhooks with retry, a
  generalized CSV Data Exchange wizard, read-only External Objects, and
  Integration Jobs run by a real background scheduler on the Team
  Workspace server (mTLS is the one deliberate, named gap).
- **Saved Views & Bulk Actions**: any filterable list screen can save its
  current filter/sort/group-by as a named view (Private/Shared, with an
  admin-settable default), reuse it as a dashboard record-list widget's
  data source, and run bulk update-status/reassign-owner/tag/archive
  operations across a multi-select.
- **AI & Agentic Layer**: a bring-your-own-key Chat Assistant and a native
  Model Context Protocol server & CLI (JSON-RPC 2.0, same scoped API keys
  as the REST API); an AI Agent Foundry for named, admin-defined agents -
  persona, a per-tool Actions checklist, persistent Memory (with real
  change history - every prior value is snapshotted before being
  overwritten, whether by the agent's own `update_memory` tool or an
  admin's direct edit), a shared Skills library, and depth-guarded
  delegation to other agents; Orchestration - deterministic Pipelines,
  fired manually, on a schedule, from an authenticated webhook, or as a
  new Workflow Automation action, every run landing in one unified
  history; a Unified AI Gateway - named AI Providers, per-agent
  primary/fallback/local-fallback model routing with automatic failover,
  real token-usage accounting against System/Agent daily budgets, and
  DLP-driven forced air-gapping the moment a request touches a configured
  sensitive-data class; approval gates on any topology's own well-defined
  resume point (a sequential Pipeline's any step, a consensus Pipeline's
  synthesizer, a peer-review Pipeline's reviewer) pause a run for an
  Administrator to approve, edit, or reject; a `search_records` tool
  giving agents ranked full-text search over Custom Object records via a
  SQLite FTS5 index - `bm25()` lexical relevance; and (opt-in, desktop
  only) Vector Search - a `semantic_search_records` tool ranking the same
  records by real embedding similarity instead, via the workspace's own
  already-configured OpenAI-compatible or Google Gemini provider
  (Anthropic has no embeddings API), stored as plain SQLite BLOBs and
  compared by cosine similarity in Rust rather than a bundled vector
  database. See `core/src/services/chat_service.rs`,
  `ai_agent_service.rs`, `ai_gateway_service.rs`, `search_service.rs`,
  `vector_search_service.rs`, `server/src/mcp.rs`.
- **Engine hardening (platform primitives, not one industry vertical)**:
  five gaps `reference_packages.rs`'s own doc comment had been
  accumulating across the twelve shipped industry packages, closed
  together since each pays off retroactively across all of them -
  self-referential relationships (a parent/child hierarchy: an
  Organization Unit's parent unit, an Asset's parent Asset), a generic
  polymorphic relationship target (a Document attached to a Policy, a
  Claim, or a Matter - any valid type, chosen per link, read paths only;
  writing through the variable side is a stated follow-up), cross-record
  validation (a business rule/workflow condition reading a field off the
  record linked through a `many_to_one`/`one_to_one` relationship, not
  just the triggering record's own), `date_reached`/`due_overdue`
  workflow triggers on a custom object's own date field (previously
  built-in-only), and an "as of" query - `effective_dating_service`
  answers whether a record's *currently stored* `valid_from`/`valid_to`
  window is in effect on a given date (not true point-in-time
  reconstruction of a past value, which the schema has no history table
  for), wired into the custom report builder. None of the twelve existing
  packages have been retrofitted to actually use these yet - see
  `reference_packages.rs`'s own doc comment for the full, honestly-scoped
  breakdown of what each closes and what's still deferred. See
  `core/src/services/relationship_service.rs`, `business_rule_service.rs`,
  `workflow_service.rs`, `effective_dating_service.rs`.
- **Enterprise Access Foundation (Phase 1 of 6)**: an Organization (one per
  workspace) containing Organization Units in a materialized-path hierarchy
  (`/root-id/child-id/.../` + `depth`, so a subtree move is one bounded
  `UPDATE ... WHERE path LIKE ?`, not a second closure-table structure to
  keep in sync) and Work Teams with time-bounded memberships - ending a
  membership never touches any record's ownership, a regression the test
  suite checks directly. Every record across Companies, Contacts,
  Opportunities, Products, Quotes, Orders, Invoices, Contracts, Tasks and
  Custom Objects now carries a structured Owner (a user or a team,
  `record_owner_type`/`record_owner_id`) and an Owning Organization Unit,
  with `ownership_version`/`assigned_at` bumped only on a real reassignment
  and an `audit_events` row per transfer - reusing the existing generic
  audit table rather than a new Security Activity feed. Bulk
  dry-run/commit transfer reuses `bulk_action_service`'s existing
  `BulkActionResult` shape rather than a parallel one. The legacy
  `owner_user_id` column present on five tables is kept, read-only, as a
  rollback safety net rather than dropped (SQLite can't rename/drop a
  column safely without a full table rebuild). The "assign" capability
  check is a deliberate placeholder (`ownership_service::require_assign_capability`,
  requiring the existing Administrator role) - the real capability+scope
  Access Role engine is Phase 2, not built yet. See
  `core/src/services/organization_service.rs`, `org_unit_service.rs`,
  `work_team_service.rs`, `ownership_service.rs`, and
  `core/tests/access_foundation_org_structure.rs` /
  `access_foundation_ownership.rs`. Mirrored in the online demo
  (`app.js`) as real client-side state, same convention as every other
  demo-mirrored feature.
- **Voice-First Mode (PR 1 of a 3-PR rollout)**: a 4-digit Voice PIN,
  separate from the primary login password, unlocks a time-boxed Voice
  session (5/15/30 minutes, admin-capped per Access Role) with lockout
  after repeated failed attempts (`voice_session_service`). A new Voice
  Governance admin panel adds 8 per-role capability toggles (Use Voice,
  Search, Create, Update, Act, Bulk Act, External Act, Use AI Agents), a
  Max Voice Action Level (Ask Only → Capture → Act with Confirmation →
  Act) and a Processing Boundary label (Cloud/Approved Private/Local
  Only - Local Only is a label today, with no real local speech adapter
  behind it) via `voice_policy_service` - a second, narrower gate that
  composes with the workspace's existing object-permission model,
  narrowing what a role can already do and never widening it. A
  metadata-driven planner (`voice_planner_service`) recognizes
  NAVIGATE/QUERY/CREATE_TASK/CAPTURE/UPDATE_STATUS commands against live
  object and status metadata rather than a fixed phrase table, so a new
  Custom Object or status is voice-addressable with no new code;
  `voice_entity_resolver` resolves a named record with an honest exact
  match / clarifying question / not-found outcome, never a silent guess.
  `voice_risk_service` classifies every command (none/low/medium/high/
  critical) against the role's action level - reads and navigation
  execute immediately, a write always shows an explicit Before → After
  Confirm/Reject step, and the critical (delete-class) tier is
  hard-blocked for Voice entirely in this release. `voice_execution_service`
  routes every write through the exact same entity service the ordinary
  UI/API already call, so Business Rules, Workflow Automation and
  status-transition validation apply identically to a manual edit; a
  status change or created task gets real Undo through those same
  services, and `voice_audit_service` makes a full trail (transcript,
  resolved intent/record, risk, outcome) queryable from My Voice
  Activity and a new admin-only Voice Activity screen. Real speech I/O
  via the Web Speech API ships alongside an always-visible "type
  instead" fallback, and raw audio is never persisted in any form. See
  `core/src/services/voice_*.rs`, `core/src/models/voice.rs`,
  `core/src/db/migrations/0057_voice_mode_v1.sql`, and the dedicated
  17-test suite in `core/tests/voice_mode_v1.rs`. Mirrored in the online
  demo (`app.js`).
- **Voice-First Mode, PR 2 (part 1): RUN_AGENT**: a new RUN_AGENT intent
  ("ask the Research Analyst agent to summarize a company", "run
  pipeline Lead Triage with this new lead") resolves a spoken Agent or
  Pipeline name against the real `ai_agent_service`/
  `ai_orchestration_service` catalogs and calls the exact same entry
  points the AI Agent Foundry's own chat panel and Orchestration's "Run
  now" button call (`chat_service::send_agent_message`/
  `ai_orchestration_service::run_manual`) - never a second, Voice-only
  agent runtime. Classified Medium risk under `voice_risk_service`'s
  existing table (this release fills in an arm the classifier already
  anticipated), so it always shows an explicit Confirm/Reject step with
  the agent's or pipeline's real answer spoken back once confirmed,
  gated by the `Use AI Agents` Voice capability on top of whatever
  admin-only/action-name restriction the target Agent already has;
  running a Pipeline manually stays Administrator-gated exactly like the
  existing "Run now" button. `submit_command`/`confirm_plan` are now
  genuinely async (they may call an LLM provider), so both the desktop
  Tauri commands and the Team Workspace server route move to the same
  `run_with_own_connection`/admin-action pattern `send_agent_message`
  already established, instead of the plain-sync path every other Voice
  command still uses; the agent/pipeline's reply has no column in the
  fixed PR 1 schema (no migration in this pass), so it rides back as a
  same-response-only `notes` field on `VoiceExecutionResult`. See
  `core/tests/voice_mode_v2.rs` (5 new tests, exercising a real stubbed
  LLM provider end to end). Mirrored in the online demo. Multi-turn
  conversational context and guided step-by-step record creation - the
  rest of the product spec's v0.3 - remain the next slice; industry
  voice vocabulary packs and real local/on-device recognition are scoped
  to PR 3, not built yet.
- **Voice-First Mode, PR 2 (part 2): multi-turn conversational context**:
  a reference like "it"/"that" now survives across a whole
  back-and-forth, not just the record currently open on screen (spec
  §14). `voice_session_service` reads and appends to each session's own
  bounded `conversation_json` (5 turns max, capped the same way other
  bounded lists already are in this codebase) - a turn is only recorded
  when a command actually resolves to one specific record, so an
  aggregate QUERY has nothing for "it" to mean next; recording happens
  as soon as `voice_execution_service::submit_command` gets a resolved
  plan, before the risk/capability gate runs, since the reference itself
  was real even if the action on it is blocked or needs confirmation.
  `voice_entity_resolver::resolve_by_reference` checks this session
  history as a second, lower-confidence fallback (0.9 vs. the live
  screen context's 1.0) - never a replacement for what's actually open
  on screen. The existing `reset_conversation` action (spec's own
  "Reset voice context") clears it without ending the session. See
  `core/tests/voice_mode_v3.rs` (3 new tests: resolving "it" purely from
  conversation history, screen context still winning when both are
  present, and reset clearing the fallback). Mirrored in the online demo
  (`app.js`), including a small "It currently means…" status line with a
  one-click Reset context action. Guided step-by-step record creation -
  the one remaining item from the product spec's v0.3 - is the next
  slice; industry voice vocabulary packs and real local/on-device
  recognition are scoped to PR 3, not built yet.
- **Voice-First Mode: bug fixes, Custom Object parity, fuzzy matching**:
  a follow-up round from real user reports on the online demo. Fixed a
  genuine mic-blocking bug - `netlify.toml`'s `Permissions-Policy` header
  set `microphone=()` (an empty allowlist, denying every origin including
  the site's own) instead of `microphone=(self)`, so the browser refused
  every `SpeechRecognition` request before a permission prompt could even
  appear; `voiceRecognition.onerror` in the demo also now surfaces an
  honest reason (blocked mic, no hardware, no network) instead of
  silently reverting to idle. Fixed `voiceSpeak()`'s speech synthesis for
  real: it now only defers to a new tick when it actually needs to cancel
  a still-speaking utterance (WebKit's same-tick cancel+speak bug), and
  speaks synchronously, inside the original click handler, in the common
  case - a *second*, separate iOS/WebKit quirk silently drops `speak()`
  calls made outside a direct user gesture, which an earlier blanket
  `setTimeout` fix had been doing unintentionally. Custom Object records
  (including ones from an installed Industry App) are now genuinely
  voice-addressable in the online demo - `voiceSearchableObjects()`
  widens the demo's previously-hardcoded 6-object list to include every
  active Custom Object (plus three built-ins, Invoice/Contract/Product,
  the old list had silently dropped), matching what the desktop/server
  edition already guaranteed via `entity_registry`/`global_search`
  (VOICE-AC-06). Added a bounded Levenshtein edit-distance **fuzzy match**
  tier (spec §8.1's own named-but-previously-unbuilt tier) on both
  platforms - `search_service::fuzzy_search` (desktop/server) and an
  identical algorithm in the demo - that only runs once an exact/substring
  search already came back empty, and still only auto-resolves a single
  close-enough candidate; more than one plausible typo still asks rather
  than guessing (VOICE-AC-05). See `core/tests/voice_mode_v4.rs` (4 new
  tests: a Custom Object record resolves like a built-in one, a single
  close typo resolves, two plausible typos ask instead of guessing, and a
  genuinely unrelated name still gets an honest "not found").

## What's deferred to a later phase

- Custom fields as extra columns on list screens, and in CSV import/export
- A full drag-and-drop report/dashboard builder beyond the simple
  group-by-and-aggregate report builder, and reordering Dashboard KPI tiles
- The Approval Framework, Data Quality Center, and Form Builder sections
  of the v1.3 spec - each is its own substantial subsystem, out of scope
  for the phases done so far
- Access Foundation Phases 2-6 from the Enterprise Requirements
  Specification (Organization/Org Units/Work Teams/ownership shipped
  above is Phase 1 only): Access Control v1 (capability+scope Access
  Roles, Record Scopes, an Access Inspector), v1.1 (Field Access
  Policies, Capability Packs/Bundles, Access Groups, Record Grants), v1.2
  (Record Access Policies, stronger bulk-reassignment audit), AI Access
  v1 (AI Action Levels, Agent/Integration Identities, data boundaries),
  and Enterprise Identity v2 (OIDC/SAML/SCIM, external users) - tracked
  as GitHub issues, not built yet
- Making session auto-lock's 15-minute timeout admin-configurable
- A nicer inline banner for non-blocking business-rule `show_message`
  actions - currently a plain `alert()` (see `src/lib/ruleMessages.ts`)
- Windows installer signing/packaging (the Tauri bundle config targets
  `nsis`/`msi`, which need a Windows build host - see below; a GitHub
  Actions workflow at `.github/workflows/desktop-release.yml` now builds
  and drafts a release automatically on a `desktop-v*` tag push - the
  desktop-v0.13.0 Early Access build is already published, just unsigned)

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

To build/test the Rust backend on its own (from the `desktop/` workspace root):

```bash
cargo test --workspace   # every crate: lanesra-core's 60+ integration test
                         # files (one per shipped feature area - money,
                         # numbering, the full sales lifecycle, every
                         # admin-extensibility phase, App Catalog, Deployment
                         # Management, Integration Hub, the AI & Agentic
                         # Layer, ...), plus lanesra-server's HTTP tests
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

**CSV import/export phase:**

- `npm run build`: `tsc` + `vite build` both clean (no new Rust code
  this phase either - it's all client-side, reusing the existing
  `create_company`/`create_contact` commands for import).
- Built the real release `lanesra-server` binary against a fresh
  workspace and drove the whole feature end to end with a real Chromium
  browser (Playwright): with Companies empty, uploaded a 2-row CSV
  (one row with a quoted field containing a comma, to prove RFC 4180
  parsing), confirmed the preview showed both rows "Ready", clicked
  Import, and confirmed the "Done: 2 created" summary and the resulting
  company list (correct customer numbers, names, statuses, tax number)
  matched the CSV exactly.
- Exported that same list back to CSV and confirmed the downloaded file
  has a UTF-8 BOM, CRLF line endings, and correctly re-quotes the field
  containing a comma - a clean round trip.
- Did the same for Contacts with a 3-row CSV where the third row named
  a company that doesn't exist ("Ghost Inc"): the preview correctly
  flagged that one row with `No company named "Ghost Inc" - create it
  first` while leaving the other two "Ready", and after import the
  summary read "2 created, 1 skipped (invalid)" with only the two valid
  contacts actually created - confirming a bad row can't block or
  corrupt the rest of the batch, and importing a still-unresolvable row
  in Contacts fails safely rather than creating a dangling reference.

**Branding & print customization phase (FR-BRD):**

- `cargo test --workspace`: 54/54 passing - 6 new core tests
  (`core/tests/workspace_branding.rs`: profile update, empty-name
  rejection, non-administrator rejection, logo set/clear,
  disallowed-mime rejection, oversized-logo rejection) on top of
  everything from earlier phases.
- `npm run build`: `tsc` + `vite build` both clean.
- Built the real release `lanesra-server` binary and drove the whole
  feature end to end with a real Chromium browser (Playwright) and curl:
  logged in as the seeded Administrator, opened the new Settings screen,
  edited the business name/legal name/address/tax rate and confirmed
  `workspace_status` reflected every field over HTTP. Uploaded a 1x1
  test logo first, which confirmed the resize-then-cap pipeline stores
  even a degenerate image correctly (though at 1px it's too small to
  see, as expected from CSS `max-width`/`max-height` never *enlarging*
  a naturally-tiny image); re-tested with a realistic 160x160 PNG and
  confirmed it renders correctly both in the Settings screen's preview
  and on the actual quote print letterhead, alongside the new business
  address, next to the business/legal name.

**Reports phase (FR-RPT):**

- `cargo test --workspace`: 59/59 passing - 5 new core tests
  (`core/tests/reports.rs`: revenue-by-month excludes Draft invoices,
  win-rate/lost-reasons reflect real opportunity outcomes, AR aging
  buckets correctly and excludes fully-paid invoices, sales-by-owner
  attributes correctly via the company owner).
- `npm run build`: `tsc` + `vite build` both clean.
- Built the real release `lanesra-server` binary, seeded it via curl
  with a company, a Won and a Lost opportunity (with a lost reason), an
  invoice issued this period, and an invoice issued ~60 days ago and
  now 31-60 days overdue - then drove all five report tabs end to end
  with a real Chromium browser. Every number on screen was independently
  cross-checked against the seeded data: Win rate by owner showed
  exactly 1 won / 1 lost / 50% / $5,000 won value; Lost reasons showed
  "Went with a competitor" for $1,000; AR aging correctly bucketed the
  not-yet-due invoice separately from the 31-60-days-overdue one; and
  both Revenue by month and Sales by owner correctly *excluded* the
  invoice issued mid-month when the default date range's "To" was still
  earlier in the month - confirming the date-range filter, not just the
  aggregation, actually works. Exported Sales by owner to CSV and
  confirmed the downloaded file matched the on-screen table exactly.

**Custom fields phase (FR-CFG):**

- `cargo test --workspace`: 66/66 passing - 7 new core tests
  (`core/tests/custom_fields.rs`: auto-generated key, duplicate-label
  uniquification, non-administrator rejection, required-field
  enforcement, select-option validation, clearing a value deletes its
  row, deactivating a field stops enforcing it but keeps existing
  values).
- `npm run build`: `tsc` + `vite build` both clean.
- Built the real release `lanesra-server` binary and drove the whole
  feature end to end with a real Chromium browser (Playwright): from
  Settings, defined a required "Industry" select field (Retail /
  Manufacturing / Services) for Companies; opened New Company and
  confirmed it rendered as a required dropdown; confirmed the browser's
  native required-field validation blocked submission while it was
  unset; selected "Retail" and saved successfully; reopened the company
  for editing and confirmed "Retail" was still selected - a full
  create/persist/reload round trip.
- Separately, with curl (bypassing the UI entirely): created a company,
  then called `set_custom_field_values` directly with the Industry
  field left empty and got `"Industry is required"` back; called it
  again with `"Not A Real Option"` and got `"'Not A Real Option' is not
  a valid option for Industry"` back - confirming the validation lives
  in the server, not just the form's HTML5 `required` attribute.

**Conditional business rules phase (FR-RUL):**

- `cargo test --workspace`: 71/71 passing - 5 new core tests
  (`core/tests/field_rules.rs`: a rule only requires its target field
  when the trigger condition actually matches, a non-Administrator
  cannot define a rule, a rule's target must be an active custom field,
  a statically-`required` field is never enforced once a rule hides it,
  and when two active rules target the same field the one with the
  higher `sort_order` wins).
- `npm run build`: `tsc` + `vite build` both clean.
- With curl (bypassing the UI entirely): defined a "Lead Source" custom
  field on Company and a rule requiring it when Status = Prospect, then
  called `set_custom_field_values` directly on a Prospect company with
  no values and got `"Lead Source is required"` back; supplying
  `lead_source` then succeeded. A second company created as "Active
  Customer" (the rule's condition doesn't match) saved successfully with
  no custom values at all - confirming the trigger condition, not just
  the target field's existence, is what the server actually evaluates.
- Built the real release `lanesra-server` binary and drove the whole
  feature end to end with a real Chromium browser (Playwright): the
  Settings → Business rules panel showed the seeded rule as the plain-English
  sentence `When Status is "Prospect", require Lead Source.`; created a
  second rule from the UI (`hide` Lead Source when Status = Active
  Customer) and confirmed its sentence rendered correctly too. On the New
  Company form: with the default Status of Prospect, Lead Source
  rendered with a required `*`; switching Status to Active Customer made
  the field disappear entirely; switching to Inactive (a status neither
  rule mentions) brought it back, optional. Filled in Status = Prospect,
  a name, and a Lead Source value, and saved successfully end to end -
  confirming the client-side evaluator and the server's independent
  enforcement agree at every step.

**Workflow automation phase (FR-WFL Phase 1):**

- `cargo test --workspace`: 78/78 passing - 7 new core tests
  (`core/tests/workflow_rules.rs`: a rule creates its task only once the
  Opportunity's stage actually reaches the trigger value, not on an
  intermediate stage; re-saving with an unchanged stage never re-fires it;
  the created task is assigned to the Opportunity's owner when the rule
  names no explicit assignee; a non-Administrator cannot define a rule; a
  trigger status invalid for the entity type is rejected; two active
  rules matching the same transition each create their own task
  (additive, not conflict-resolved); and an Invoice's automatic Overdue
  transition, via `refresh_overdue`, fires a rule assigned through the
  invoice's Company owner).
- `npm run build`: `tsc` + `vite build` both clean.
- With curl (bypassing the UI entirely): defined a rule "when Opportunity
  stage reaches Won, create task 'Send onboarding kit', due in 3 days";
  created an Opportunity at stage New and confirmed zero tasks existed;
  called `update_opportunity` directly to move it to stage Won and
  confirmed exactly one task was created, correctly linked
  (`related_type`/`related_id`) and due-dated 3 days out; re-saved the
  same Opportunity still at stage Won and confirmed the task count stayed
  at one, not two - proving the "no-op unless the value actually changed"
  guard is real, not just something the UI happens to avoid triggering.
  Separately, logged in as a non-Administrator and confirmed
  `create_workflow_rule` was rejected with `"Only an Administrator can
  manage workflow automation"`.
- Built the real release `lanesra-server` binary and drove the admin
  screen end to end with a real Chromium browser (Playwright): Settings →
  Workflow automation showed the curl-seeded rule as the plain-English
  sentence `When stage reaches "Won", create task "Send onboarding kit"
  (due in 3 days, assigned to the record's owner).`; created a second rule
  from the UI (Invoice → Overdue, assigned to a specific user by name
  rather than "the record's owner") and confirmed its sentence rendered
  correctly, including the resolved display name. Navigated to the Tasks
  screen's "By Related Record" view and confirmed the task auto-created
  by the earlier curl-driven Won transition was there, titled correctly
  and linked back to its Opportunity by name - the same task the direct
  API call had produced, now visible through the ordinary UI a user would
  actually use.

**Admin flexibility phase (generalized fields/rules/workflows, phone,
numbering, report builder, KPI picker, admin nav):**

- `cargo test --workspace`: 88/88 passing - 10 new core tests
  (`core/tests/admin_flexibility.rs`: custom fields + a business rule +
  workflow automation all working together on Opportunity, an entity
  outside FR-CFG/FR-RUL/FR-WFL Phase 1's original scope; a numbering
  override changes the format without resetting the sequence, and
  resets back to default; only an Administrator can change a number
  format; a custom report counts records grouped by built-in status; a
  custom report sums a numeric custom field grouped by another custom
  field; a sum report is rejected if its target isn't an active numeric
  custom field; an Administrator can set and reset Dashboard KPI
  preferences; only an Administrator can change them; the workspace
  profile stores a phone number - plus one existing workspace-branding
  test updated for the new `phone` field; `domain::numbering` gained its
  own test proving an override reformats without resetting the sequence).
- `npm run build`: `tsc` + `vite build` both clean.
- With curl (bypassing the UI entirely): created a custom field and a
  business rule on Opportunity (previously Company/Contact only),
  confirmed the rule's `"Lead Source is required"` rejection fires
  correctly once status is Won; overrode Company's number format to
  `ACC`/4 digits and confirmed the *next* company issued got
  `ACC-0002` (continuing the sequence, not restarting it, since the
  first company had already taken `CUS-000001` under the old format);
  built a custom report counting Companies grouped by status and got
  back the correct `{Active Customer: 1, Prospect: 1}`; confirmed a
  non-Administrator gets rejected from `set_numbering_format`,
  `create_custom_report`, and `set_dashboard_kpis` with the expected
  "Only an Administrator" messages.
- Built the real release `lanesra-server` binary and drove the whole
  phase end to end with a real Chromium browser (Playwright): confirmed
  the sidebar shows a single "Admin" item and no standalone "Users" item;
  the Admin panel's Users tab showed the sales rep created via curl; the
  Business profile tab showed the phone number set via curl; the
  Numbering tab showed Company's ACC override labeled "Custom"; the
  Business rules and Custom fields tabs, switched to the Opportunities
  sub-tab, showed the rule and field created via curl; the Dashboard
  KPIs tab reflected a curl-set 2-KPI preference, and the Dashboard
  itself rendered exactly those 2 tiles (not the default 7); the
  Opportunity form showed Lead Source as required once Status was set to
  Won; and Reports → Custom reports ran the curl-created report and
  displayed the correct grouped counts - the same report the direct API
  call had produced, now visible through the ordinary UI.

**Custom Objects phase (admin extensibility §20.2 - Phase A of the
extensibility platform):**

- `cargo test --workspace`: 90/90 passing - 7 new core tests
  (`core/tests/custom_objects.rs`: an Administrator can define a custom
  object and a non-admin cannot; two objects with the same name get
  auto-uniquified keys ("vendor", "vendor_2"); records are numbered from
  the object's own definition, independent of the built-in entities'
  numbering; creating a record for an unknown or deactivated object type
  is rejected; deleting an object definition is blocked while records
  exist but deactivating it is always allowed and non-destructive; a
  custom object can't be named the same as a built-in entity; and the
  composition test, custom fields + a business rule + the custom report
  builder all working on a genuinely new object type with zero
  custom-object-specific code in any of those three subsystems, mirroring
  how `admin_flexibility.rs` proves the same composition on a built-in
  entity).
- `npm run build`: `tsc` + `vite build` both clean.
- Built the real release `lanesra-server` binary and drove the whole
  phase end to end with a real Chromium browser (Playwright), through a
  full first-run setup rather than curl-seeded data: created a "Vendor"
  custom object (prefix `VEN`, 4 digits) from Admin → Custom Objects;
  confirmed the sidebar immediately grew a new "Vendors" section below a
  divider; added a "Contact Email" custom field to Vendors from the
  *existing* Custom fields admin screen, which now lists Vendors as an
  extra tab alongside the nine built-in entity types with no dedicated
  UI of its own; confirmed the Business rules admin screen lists Vendors
  as a tab too, for free; navigated to the new Vendors screen, created a
  record with the custom field filled in, and confirmed it was numbered
  `VEN-0001`; reopened it in edit mode and confirmed the custom field
  value round-tripped correctly. Zero JS console errors throughout.

**Custom Relationships, richer Business Rules/Workflow Automation, and
polish phase (admin extensibility Phases B-E, driven by the v1.3
Windows requirements doc):**

- `cargo test --workspace`: 116/116 passing - 26 new core tests across
  three new integration test files plus 5 more added to an existing one:
  `core/tests/relationships.rs` (7: many-to-one/one-to-one/many-to-many
  cardinality enforcement, `restrict` vs `archive` delete behavior, a
  non-Administrator cannot define a relationship, linking rejects an
  invalid entity type), `core/tests/business_rules.rs` (11: AND vs OR
  multi-condition matching, each of the 10 operators, `lock`/`set_value`/
  `block_save`/`show_message` actions, rule priority ordering, an
  `effective_start_date`/`effective_end_date` outside today's date is
  skipped), `core/tests/workflow_automation.rs` (10: `record_created`,
  `field_changed`, `date_reached`/`due_overdue` firing exactly once via
  `workflow_runs` dedup, `assign_owner` resolving through a Quote's
  Company when the Quote itself has no owner column, `add_notification`
  targeting `all_admins`, the recursion guard stopping a
  `create_related_record` chain at depth 5 instead of hanging), and
  `core/tests/custom_fields.rs` gained 5 (`number_field_enforces_min_and_max`,
  `min_cannot_exceed_max_when_defining_a_number_field`,
  `text_field_enforces_max_length_and_pattern`,
  `an_invalid_regex_pattern_is_rejected_at_definition_time`,
  `a_field_flagged_not_reportable_cannot_be_used_as_a_report_group_by`).
  Every business-rule and workflow-automation test from the two older,
  now-deleted `field_rules.rs`/`workflow_rules.rs` files has an equivalent
  (or stronger) case in the new files - nothing was dropped, the engines
  were replaced wholesale, not patched.
- `npm run build`: `tsc` + `vite build` both clean at v0.4.0.
- Built the real release `lanesra-server` binary and drove the new admin
  screens end to end with a real Chromium browser (Playwright): created a
  many-to-one "Primary Contact" relationship between Opportunity and
  Contact from Settings → Relationships, linked a record from the new
  related-list card on an Opportunity's detail view, and confirmed
  attempting a second link from the same Opportunity was rejected with a
  cardinality error while the reverse label rendered correctly on the
  Contact side; built a 2-condition OR business rule (`hide` a field when
  Status is either of two values) in the new Business Rules admin screen
  and confirmed its "Test rules" panel matched the server's own
  evaluation; built a `field_changed`-triggered workflow that assigns the
  record's owner and posts an `all_admins` notification, then confirmed
  the bell icon in the topbar picked up the new notification with an
  unread badge and "mark all read" cleared it.
- With curl (bypassing the UI entirely): defined a numeric custom field
  with `min_value`/`max_value` and confirmed a value outside the range
  was rejected with the exact bound in the error message; confirmed a
  field flagged `is_reportable: false` was silently excluded from
  `create_custom_report`'s valid group-by targets rather than erroring
  opaquely; drove a `many_to_many` relationship to its DB-level unique
  constraint and confirmed the raw SQLite constraint violation surfaced
  as the same friendly "already linked" message the service layer
  produces for the other two cardinalities, not a raw SQL error.
- Manually exercised session auto-lock and task-reminder notifications in
  the running Tauri binary under Xvfb: left the app idle past the
  (temporarily shortened for testing) timeout and confirmed the lock
  screen appeared and correctly rejected a wrong password before
  accepting the real one; set a task's reminder to a past timestamp and
  confirmed a Windows-style toast (via WebView2's `Notification` API
  surface) fired exactly once per task across a reload, per the
  `localStorage` dedup set.
