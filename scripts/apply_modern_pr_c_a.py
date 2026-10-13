#!/usr/bin/env python3
"""Modern UI Revamp PR-C - Worker A: Admin Center reskin (issue #385).

app.js: reskin adminLanding() (setup-progress / needs-attention / platform-
health panels, landing grids, category cards) onto catalog components
(.card, .badge) with token-driven helper classes; adminToolView() already
uses catalog classes (.page-head/.breadcrumbs) so its markup is unchanged.
adminRecentNavHtml() is intentionally byte-identical (it is one of the
listed data functions); its card picks up the reskin purely through the
token-driven .admin-cat-card CSS.

styles.css: migrate the admin chrome (.admin-landing-grid, .admin-cat-card
+ head/icon/items, .admin-cat-item, .admin-cat-arrow, .admin-body, .kpi)
to 100% token-driven CSS in the concept artifact's settings-card language.

Visual reskin ONLY: every data value, route, handler, filter, and behavior
is unchanged. Data functions adminSetupProgressItems,
adminNeedsAttentionItems, adminRecentChanges, adminPlatformHealthHtml and
ADMIN_CATEGORIES are byte-identical (verified separately). Every
replacement asserts its anchor matches EXACTLY the expected count; any
mismatch fails loudly.

Usage: run from the repo root. The workflow must stage both files
(`git add app.js styles.css`) for the changes to be committed.
"""
import sys

APP = 'app.js'
CSS = 'styles.css'

JS_REPLACEMENTS = [
    # -- adminLanding: setup-progress panel -> catalog .card + .badge --
    ('a1:adminLanding: setup progress panel',
     ' ${setupDone?\'\':`<div class="panel" style="margin-bottom:18px"><h3 style="margin-top:0">Setup progress</h3><div class="admin-cat-items">${setup.map(s=>`<div class="admin-cat-item" style="cursor:default"><span>${s.done?\'✓\':\'○\'} ${s.label}</span></div>`).join(\'\')}</div></div>`}\n',
     ' ${setupDone?\'\':`<section class="card prc-admin-panel"><div class="prc-panel-head"><h3>Setup progress</h3><span class="badge badge-secondary">${setup.filter(s=>s.done).length} of ${setup.length} done</span></div><div class="admin-cat-items">${setup.map(s=>`<div class="admin-cat-item" style="cursor:default"><span class="prc-row-label"><span class="prc-status${s.done?\' prc-status-done\':\'\'}">${s.done?\'✓\':\'○\'}</span>${s.label}</span>${s.done?\'<span class="badge badge-success">Done</span>\':\'<span class="badge badge-outline">To do</span>\'}</div>`).join(\'\')}</div></section>`}\n',
     1),
    # -- adminLanding: needs-attention panel -> catalog .card + .badge --
    ('a2:adminLanding: needs attention panel',
     ' ${attention.length?`<div class="panel" style="margin-bottom:18px"><h3 style="margin-top:0">Needs attention</h3><div class="admin-cat-items">${attention.map(a=>`<button type="button" class="admin-cat-item" data-admin-open="${a.tab}">${a.label}<span class="admin-cat-arrow">${a.count} →</span></button>`).join(\'\')}</div></div>`:\'\'}\n',
     ' ${attention.length?`<section class="card prc-admin-panel"><div class="prc-panel-head"><h3>Needs attention</h3><span class="badge badge-warning">${attention.length}</span></div><div class="admin-cat-items">${attention.map(a=>`<button type="button" class="admin-cat-item" data-admin-open="${a.tab}"><span>${a.label}</span><span class="prc-row-meta"><span class="badge badge-warning">${a.count}</span><span class="admin-cat-arrow">›</span></span></button>`).join(\'\')}</div></section>`:\'\'}\n',
     1),
    # -- adminLanding: platform-health panel -> catalog .card --
    ('a3:adminLanding: platform health panel',
     ' <div class="panel" style="margin-bottom:18px"><h3 style="margin-top:0">Platform health</h3><div class="kpi-grid">${adminPlatformHealthHtml()}</div></div>\n',
     ' <section class="card prc-admin-panel"><div class="prc-panel-head"><h3>Platform health</h3></div><div class="kpi-grid">${adminPlatformHealthHtml()}</div></section>\n',
     1),
    # -- adminLanding: landing grid stack spacing -> token class --
    ('a4:adminLanding: grid stack token class',
     '<div class="admin-landing-grid" style="margin-bottom:18px">',
     '<div class="admin-landing-grid prc-grid-stack">',
     1),
    # -- adminLanding: category cards keep .admin-cat-card (now token-driven
    #    card chrome); item rows get label span + artifact chevron --
    ('a5:adminLanding: category card rows',
     '`<button class="admin-cat-item" data-admin-open="${k}">${tabLabel(k)}<span class="admin-cat-arrow">→</span></button>`',
     '`<button class="admin-cat-item" data-admin-open="${k}"><span>${tabLabel(k)}</span><span class="admin-cat-arrow">›</span></button>`',
     1),
]

CSS_REPLACEMENTS = [
    # -- platform-health KPIs -> token-driven card chrome --
    ('c1:kpi token-driven',
     '.kpi{background:#fff;border:1px solid var(--line);border-radius:16px;padding:18px}.kpi-label{font-size:13px;color:#6b7280}.kpi-value{font-size:28px;font-weight:850;margin-top:8px}',
     '.kpi{background:var(--card);border:1px solid var(--border);border-radius:var(--radius-lg);padding:var(--space-5)}.kpi-label{font-size:var(--text-sm);color:var(--muted-foreground)}.kpi-value{font-size:var(--text-2xl);font-weight:800;margin-top:var(--space-2);letter-spacing:-0.02em;color:var(--foreground)}',
     1),
    # -- admin landing chrome -> token-driven, artifact settings-card look --
    ('c2:admin chrome token-driven',
     '.admin-landing-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:18px}\n'
     '.admin-cat-card{background:#fff;border:1px solid var(--line);border-radius:16px;padding:20px}\n'
     '.admin-cat-head{display:flex;gap:12px;align-items:flex-start;margin-bottom:14px}\n'
     '.admin-cat-icon{font-size:22px;width:42px;height:42px;border-radius:12px;background:#eef2ff;display:grid;place-items:center;flex-shrink:0}\n'
     '.admin-cat-head h3{margin:0 0 3px;font-size:16px}\n'
     '.admin-cat-head p{margin:0;font-size:12.5px}\n'
     '.admin-cat-items{display:flex;flex-direction:column;gap:2px}\n'
     '.admin-cat-item{display:flex;justify-content:space-between;align-items:center;width:100%;text-align:left;background:none;border:0;border-radius:9px;padding:9px 10px;font-size:13.5px;font-weight:600;color:#1f2937;cursor:pointer}\n'
     '.admin-cat-item:hover{background:#f5f6fa;color:#4338ca}\n'
     '.admin-cat-arrow{color:#9ca3af;font-weight:400}\n'
     '.admin-cat-item:hover .admin-cat-arrow{color:#4338ca}',
     '/* Modern UI Revamp - PR-C Worker A (issue 385): Admin Center chrome.\n'
     '   Settings-style grouped cards matching the concept artifact. Every value\n'
     '   is a var(--token) or color-mix() of tokens: no hex, no px radii, no\n'
     '   literal shadows. Muted text uses --muted-foreground; muted surfaces\n'
     '   use --secondary. */\n'
     '.admin-landing-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:var(--space-4)}\n'
     '.prc-grid-stack{margin-bottom:var(--space-5)}\n'
     '.admin-cat-card{background:var(--card);color:var(--card-foreground);border:1px solid var(--border);border-radius:var(--radius-lg);box-shadow:var(--shadow-sm);padding:var(--space-5);display:flex;flex-direction:column;gap:var(--space-4)}\n'
     '.admin-cat-head{display:flex;gap:var(--space-3);align-items:flex-start}\n'
     '.admin-cat-icon{font-size:var(--text-xl);width:42px;height:42px;border-radius:var(--radius-md);background:var(--accent);color:var(--accent-foreground);display:grid;place-items:center;flex-shrink:0}\n'
     '.admin-cat-head h3{margin:0 0 var(--space-1);font-size:var(--text-base);font-weight:700;letter-spacing:-0.01em;color:var(--card-foreground)}\n'
     '.admin-cat-head p{margin:0;font-size:var(--text-xs);color:var(--muted-foreground)}\n'
     '.admin-cat-items{display:flex;flex-direction:column}\n'
     '.admin-cat-item{appearance:none;display:flex;justify-content:space-between;align-items:center;gap:var(--space-3);width:100%;text-align:left;background:transparent;border:0;border-top:1px solid var(--border);border-radius:var(--radius-sm);padding:var(--space-3) var(--space-2);font-size:var(--text-sm);font-weight:600;color:var(--foreground);cursor:pointer;transition:background-color var(--motion-fast) ease,color var(--motion-fast) ease}\n'
     '.admin-cat-items>.admin-cat-item:first-child{border-top:0}\n'
     '.admin-cat-item:hover{background:color-mix(in srgb,var(--accent) 55%,transparent);color:var(--accent-foreground)}\n'
     '.admin-cat-arrow{color:var(--muted-foreground);font-weight:400;flex-shrink:0}\n'
     '.admin-cat-item:hover .admin-cat-arrow{color:var(--accent-foreground)}\n'
     '.prc-row-label{display:flex;align-items:center;gap:var(--space-2);min-width:0;flex:1}\n'
     '.prc-status{display:inline-grid;place-items:center;width:var(--space-5);height:var(--space-5);border-radius:var(--radius-full);background:var(--secondary);color:var(--muted-foreground);font-size:var(--text-xs);flex-shrink:0}\n'
     '.prc-status-done{background:color-mix(in srgb,var(--success) 15%,transparent);color:var(--success)}\n'
     '.prc-row-meta{display:inline-flex;align-items:center;gap:var(--space-2);flex-shrink:0}\n'
     '.prc-admin-panel{padding:var(--space-5);margin-bottom:var(--space-5)}\n'
     '.prc-panel-head{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);margin-bottom:var(--space-3)}\n'
     '.prc-panel-head h3{margin:0;font-size:var(--text-base);font-weight:700;letter-spacing:-0.01em;color:var(--card-foreground)}',
     1),
    # -- #adminBody stack spacing -> token --
    ('c3:admin-body token gap',
     '.admin-body{display:flex;flex-direction:column;gap:18px}',
     '.admin-body{display:flex;flex-direction:column;gap:var(--space-5)}',
     1),
]


def apply(path, replacements):
    with open(path, encoding='utf-8') as f:
        text = f.read()
    for name, old, new, expected in replacements:
        n = text.count(old)
        if n != expected:
            sys.exit('ANCHOR FAIL [%s] (%s): expected %d, found %d'
                     % (name, path, expected, n))
        text = text.replace(old, new)
    with open(path, 'w', encoding='utf-8') as f:
        f.write(text)
    print('OK %s: %d replacements' % (path, len(replacements)))


def main():
    apply(APP, JS_REPLACEMENTS)
    apply(CSS, CSS_REPLACEMENTS)


if __name__ == '__main__':
    main()
