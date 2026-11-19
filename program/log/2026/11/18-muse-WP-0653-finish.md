# Session log — 2026-11-18 — WP-0653 finish (Muse)

## Scope
WP-0653 AC-4 (Playwright spec), AC-5 (help + docs), AC-6 (polish + parity).
AC-1..AC-3 shipped in #790 (Claude). Demo-only WP: no Rust, no REGRESSION.md
totals touched. WP-0710's done-status folds into this PR (merged as #796).

## Claim
Branch `wp/WP-0653-foundry-demo-finish` from main `1164060` (the #796 merge).
Claim commit `7ba7afb` (WP file in_progress + owner muse + `delivered: [demo]`,
WP-0710 set done, regenerated STATUS.md). Claim push pending on an approval
card at time of writing — see the push note below.

## What changed
- `desktop/e2e/tests/foundry-demo.spec.ts` (new, port 4179): 7 tests.
  AC-4: bands + all thirteen items, reused screen (Agents) + native panels
  (Traces, Monitor), Optimization hero 0.71 → 0.82, sales + warehouse demos
  still load, zero page errors on load. AC-6: Orchestration view (real
  Pipelines + Sales Ops Team), theme toggle flips `data-theme`, phone width
  390px no horizontal scroll + mobile drawer flow.
- `app.js`: FOUNDRY_NAV gains Orchestration (Create band, render
  `orchestration`); new `foundryOrchestration()`; home Orchestration card
  links to it; Monitor gains Runs by agent + approvals inbox state; Traces
  gains per-span duration bars + per-trace tool/guardrail/delegation counts;
  help article `agent-foundry-demo` (5 sections); Releases v0.99.12; Roadmap
  item under "AI Agent Foundry & Orchestration".
- `sitemap.xml`: `/help/agent-foundry-demo` entry (`/demo/foundry` already
  present). `llms.txt` + `llms-full.txt`: Foundry demo lines.
- WP file: AC-4/AC-5/AC-6 ticked; AC-1 + Goal + Scope updated to thirteen
  items (Orchestration is the thirteenth, authorized by AC-6 itself).

## Local proof
- `foundry-demo.spec.ts`: 7/7 passed (headless shell, static server).
- `node --check app.js` clean (run after every app.js edit round).
- `programctl validate`: 0 errors, 0 warnings.
- Full demo E2E regression batch: see the batch result recorded below.

## Gotchas
- Phone-width nav: at 390px the lower rail items sit below the fold, so the
  spec must open the drawer first. The first phone test clicked the rail
  item directly and hung on Playwright's actionability wait for the full
  30s test timeout; the trace showed the Click never landed. That is the
  designed mobile flow (drawer restores labels), not a product bug.
- The headless shell lives at `~/.cache/ms-playwright/chromium_headless_shell-1254/`
  (installed 2026-11-18 via curl-with-resume; the Playwright downloader kept
  failing). `npm ci` needed in a fresh worktree before running specs.

## Push note
The claim push (`7ba7afb`) sat at `pending_user_confirmation` for 15+ minutes
on 2026-11-18 ~16:33–16:49 PST — the user was away from his phone. Per the
standing approval pattern it was NOT re-fired blindly. The build is done
locally and committed (`87179ef`, 7 files; spec 7/7, demo regression batch
58/58, node --check clean, validate 0/0). The push session then died in a
runtime restart (~17:10 PST) with no confirmed result; verified via
`git ls-remote` that the branch was never created. The user said "send
approval card" at 19:18 PST — re-firing the claim push now, then the build
push, then the draft PR.
