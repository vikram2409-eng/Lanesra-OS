# Design token inventory

Maps the Agent Playground spec palette to desktop CSS variables in
`desktop/src/styles/tokens.css`. Gaps are marked **GAP** with the token to add.
All colour pairs checked for WCAG AA (4.5:1 text, 3:1 UI) in light and dark themes.

## Brand / primary

| Spec palette | Desktop token | Light | Dark | AA |
|---|---|---|---|---|
| Brand purple | `--primary` | `#635bff` on `#ffffff` ✅ | `#635bff` on `#09090b` ✅ | pass |
| Brand deep | `--primary-foreground` | `#ffffff` on `#635bff` ✅ | — | pass |

## Semantic

| Spec use | Desktop token | Light | Dark | AA |
|---|---|---|---|---|
| Success | `--success` | `#16a34a` | `#4ade80` | pass |
| Warning | `--warning` | `#d97706` | `#f59e0b` | pass |
| Danger | `--destructive` | `#ef4444` | `#ef4444` | pass |
| Info | `--info` | `#3b82f6` | `#3b82f6` | pass |

## Surfaces

| Spec use | Desktop token | Light | Dark |
|---|---|---|---|
| Background | `--background` | `#ffffff` | `#09090b` |
| Foreground | `--foreground` | `#09090b` | `#f4f4f5` |
| Card | `--card` | `#ffffff` | `#111827` |
| Muted | `--muted` | `#f4f4f5` | `#1e2a4a` |
| Border | `--border` | `#e4e4e7` | `#26314f` |

## Playground-specific (gaps)

| Spec component | Token needed | Status |
|---|---|---|
| Risk badge (low/med/high/critical) | `--risk-low`, `--risk-medium`, `--risk-high`, `--risk-critical` | **GAP** — add to tokens.css |
| Lifecycle pill (draft/active/paused/archived) | `--lifecycle-*` | **GAP** — add to tokens.css |
| Trace item severity | reuse `--info/--warning/--destructive` | ✅ covered |
| Budget meter | `--budget-ok`, `--budget-warn`, `--budget-over` | **GAP** — add to tokens.css |
| Setup rail / progress | `--progress-track`, `--progress-fill` | **GAP** — add to tokens.css |

## Typography / spacing / radius

All mapped: `--font-sans`, `--font-mono`, `--text-xs`…`--text-2xl`,
`--space-1`…`--space-8`, `--radius-sm`…`--radius-full`, `--shadow-sm`…`--shadow-lg`.

## Component inventory (Phases 1–3)

| Component | Exists in `desktop/src/components/ui` | Action |
|---|---|---|
| RiskBadge | No | Build in WP-0160 |
| LifecyclePill | No | Build in WP-0160 |
| StatusDot | No | Build in WP-0203 |
| EffectNote | No | Build in WP-0203 |
| Toast | No | Build in WP-0203 |
| BudgetMeter | No | Build in WP-0203 |
| TraceItem | No | Build in WP-0203 |
| ProgressRail | No | Build in WP-0203 |
| badge, button, card, dialog, input, select, tabs, tooltip | Yes | Reuse |

Covers: REQ-PRE-011 (WP-0014 AC-1)
