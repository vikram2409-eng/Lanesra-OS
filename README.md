# Lanesra OS

Modern, open-source sales and business management software for small businesses. Run it privately on Windows (offline, no cloud account, no licence key), share it with a small team over your local network, or try it instantly online with no install at all.

## Features

- **Companies, Contacts & Sales Pipeline** — connected customer records with a Kanban/list opportunity pipeline
- **Products & Services**, **Quotes**, **Orders**, **Invoices**, **Contracts**, **Tasks** — the full flexible sales lifecycle (Company → Opportunity → Quote → Order → Invoice), plus direct-quote/direct-order shortcuts
- **Dashboard & global search** with clickable, filterable KPIs
- **Reports** beyond the dashboard (revenue by month, win rate, AR aging, sales by owner), plus a simple report builder any admin can use to build their own
- **Branding & print customization** — logo, business profile, and PDF/print output for quotes, orders and invoices
- **Admin panel**: user accounts and roles, custom fields, conditional business rules, and workflow automation (auto-created follow-up tasks) — configurable across every major object, not hardcoded to one or two
- **Admin-configurable ID/numbering formats** per object (e.g. `CUS-000001` → `ACC-000001`)
- CSV import/export, whole-workspace backup & restore, self-service password change
- Runs fully offline — no cloud account, licence key, or mandatory internet connection

## Try it online

No install, no registration: **[lanesraos.com/demo](https://lanesraos.com/demo)**

## Download the desktop edition

The Windows desktop edition (Tauri + Rust + SQLite) is in active Early Access. Grab the latest installer from **[GitHub Releases](https://github.com/vikram2409-eng/Lanesra-OS/releases)** (unsigned — Windows will warn on first run) or build it from source.

Full architecture, dev setup, and a detailed "what's here / what's deferred" breakdown live in **[`desktop/README.md`](desktop/README.md)** — read that before assuming a feature exists.

## Running it for a team (on-prem / LAN)

Lanesra OS ships in two operating modes from the same codebase:

- **Personal Workspace** — the desktop app above, single user or a shared PC
- **Team Workspace** — one machine on your network runs a small local server; everyone else opens it in a browser tab and signs in with their own account. No Docker required to try it, though a `Dockerfile`/`docker-compose.yml` are included for the recommended setup.

See **[`desktop/README.md`](desktop/README.md#two-operating-modes)** for exact commands (`cargo run` locally, or `docker compose up -d`) — this targets a local network, not the public internet; put it behind a reverse proxy with TLS if you need that.

## Repository layout

```
/                website + no-registration browser demo (this file's context)
/desktop         the Tauri + Rust + SQLite desktop app and Team Workspace
                  server - see desktop/README.md for the real technical README
```

The root of this repo is the public product website (`lanesraos.com`), a static site with a browser-only demo at `/demo` — it shares product language with the desktop edition but is a separate, simpler codebase (no backend, `localStorage`-based). It auto-deploys from `main` via Netlify.

## Public product pages

- `/principles` — The product decisions and beliefs behind Lanesra OS
- `/compare` — A factual market-positioning comparison
- `/download` — Desktop platform status, what's available today, and what's still planned
- `/roadmap` — Current, building and planned capabilities
- `/changelog` — Release-by-release updates

## Previewing the website locally

The root site is a single-page app with no build step:

```bash
python3 -m http.server 8080
```

Then open `http://localhost:8080`.
