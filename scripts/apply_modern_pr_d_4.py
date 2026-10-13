#!/usr/bin/env python3
"""
Modern UI PR-D Worker 4 - Admin Center + Theme Studio artifact fidelity.

Structural rebuild (not a styling pass) of three regions in app.js, plus an
append-only token-mapped artifact CSS block in styles.css:

  1. adminLanding()      -> artifact #admin-home: .settings-page > .page-heading
                            (.eyebrow + h2 + description + action buttons) +
                            .settings-grid of .card.settings-card cards
                            (.settings-icon, .settings-copy [h3 + .settings-meta
                            of per-tab link chips], chevron where clickable).
                            Every data-admin-open / data-pin-admin /
                            data-clear-filter link and its target is kept.
  2. adminToolView()      -> artifact settings-page language: .settings-page >
                            .admin-panel > back buttons + .page-heading with a
                            real "Administration / <Category>" eyebrow and the
                            category note as the description. Tab bodies are
                            untouched - every control and its wiring is intact.
  3. themeStudioTab() + sub-views (gate panel, presets, versions,
     themeAuditLogHtml, themeDiffHtml, themeDraftPanel) -> artifact patterns:
     .admin-panel, .page-heading, .token-table for token rows,
     .swatches/.swatch for preset previews, .form-grid + .control-row for
     editor controls. Every id / data-* hook / handler is preserved, so all
     PR-C features keep working: token editor, presets, component overrides,
     custom CSS, brand assets, diff, audit log, scheduled publish,
     manage_themes guards, publish/rollback.

Rules honored: exact ordinal string anchors with `assert count==1` (loud
failure, no regex replaces); styles.css is APPEND ONLY.

Byte-reproducible: same pristine inputs -> byte-identical outputs.

Usage: apply_modern_pr_d_4.py [app.js] [styles.css]
       (defaults: /tmp/prd-w4/app.js /tmp/prd-w4/styles.css)
"""
import sys

APP_PATH = sys.argv[1] if len(sys.argv) > 1 else '/tmp/prd-w4/app.js'
CSS_PATH = sys.argv[2] if len(sys.argv) > 2 else '/tmp/prd-w4/styles.css'


def swap(src, old, new, label):
    """Exact-string, single-occurrence replacement. Fails loudly otherwise."""
    n = src.count(old)
    if n != 1:
        raise SystemExit('ANCHOR FAIL [%s]: expected 1 occurrence, found %d' % (label, n))
    print('  ok [%s]' % label)
    return src.replace(old, new, 1)


# ---------------------------------------------------------------------------
# 1. adminLanding()
# ---------------------------------------------------------------------------
def transform_admin_landing(app):
    print('region: adminLanding()')

    # 1a. Page shell: .page-head + breadcrumbs + h1 -> artifact .settings-page >
    # .admin-panel > .page-heading (.eyebrow + h2 + description + actions).
    # The artifact's action button is "Search settings" -> wired to the app's
    # own command palette (openCmdPalette); the dashboard breadcrumb becomes a
    # "Back to dashboard" ghost button so the data-clear-filter link survives.
    app = swap(
        app,
        ''' $('#view').innerHTML=`<div class="page-head"><div><div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><span>Admin</span></div><h1>Admin</h1><p class="muted">Configure your workspace, users and automation. Changes save immediately in this browser.</p></div></div>''',
        ''' $('#view').innerHTML=`<div class="settings-page"><div class="admin-panel active"><div class="page-heading"><div><div class="eyebrow">Administration</div><h2 style="margin-top:5px">Settings</h2><p>Configure your workspace, users and automation. Changes save immediately in this browser.</p></div><div style="display:flex;gap:8px;flex-wrap:wrap;align-items:center"><button class="btn btn-outline" id="adminSearchSettings" type="button">Search settings</button><button class="btn btn-ghost btn-sm" data-clear-filter type="button">Back to dashboard</button></div></div>''',
        'landing-heading',
    )

    # 1b. Category cards: .admin-landing-grid of .admin-cat-card ->
    # .settings-grid of artifact .card.settings-card (article variant, like the
    # artifact's own Data Model / AI & Automation / Workspace cards). Each tab
    # link survives as a .settings-count chip button carrying the original
    # data-admin-open target; the shared querySelectorAll('[data-admin-open]')
    # wiring below is untouched.
    app = swap(
        app,
        ''' <div class="admin-landing-grid">${ADMIN_CATEGORIES.map(c=>`<div class="admin-cat-card"><div class="admin-cat-head"><span class="admin-cat-icon">${c.icon}</span><div><h3>${c.label}</h3><p class="muted">${c.note}</p></div></div><div class="admin-cat-items">${c.items.map(k=>`<button class="admin-cat-item" data-admin-open="${k}"><span>${tabLabel(k)}</span><span class="admin-cat-arrow">›</span></button>`).join('')}</div></div>`).join('')}</div>`;''',
        ''' <div class="settings-grid">${ADMIN_CATEGORIES.map(c=>`<article class="card settings-card"><span class="settings-icon" aria-hidden="true">${c.icon}</span><div class="settings-copy"><h3>${c.label}</h3><p>${c.note}</p><div class="settings-meta">${c.items.map(k=>`<button type="button" class="settings-count" data-admin-open="${k}" title="Open ${tabLabel(k)}">${tabLabel(k)}</button>`).join('')}</div></div></article>`).join('')}</div></div></div>`;''',
        'landing-category-grid',
    )

    # 1c. Wire the Search settings button to the app's own command palette.
    # (Two-line anchor: adminToolView's data-clear-filter line is followed by
    # the data-admin-home line, so this only matches adminLanding.)
    app = swap(
        app,
        ''' $('[data-clear-filter]').onclick=()=>{current='dashboard';viewFilter=null;renderView()};
 document.querySelectorAll('[data-admin-open]').forEach(b=>b.onclick=()=>{adminTab=b.dataset.adminOpen;adminView='tool';renderView()});''',
        ''' $('[data-clear-filter]').onclick=()=>{current='dashboard';viewFilter=null;renderView()};
 const adminSearchBtn=$('#adminSearchSettings');if(adminSearchBtn)adminSearchBtn.onclick=()=>openCmdPalette();
 document.querySelectorAll('[data-admin-open]').forEach(b=>b.onclick=()=>{adminTab=b.dataset.adminOpen;adminView='tool';renderView()});''',
        'landing-search-wiring',
    )
    return app


# ---------------------------------------------------------------------------
# 2. adminToolView()
# ---------------------------------------------------------------------------
def transform_admin_tool_view(app):
    print('region: adminToolView()')

    # 2a. Helpers: real category label + note per tab for the artifact eyebrow
    # and page description (no invented copy).
    app = swap(
        app,
        '''function adminToolView(){''',
        '''// Modern UI PR-D (Worker 4): artifact settings-page language for admin
// section pages - the eyebrow names the category the tab belongs to and the
// description reuses that category's own note, so both stay real data.
function adminCategoryForTab(tab){return ADMIN_CATEGORIES.find(c=>c.items.includes(tab))||null}
function adminEyebrowSuffix(tab){const c=adminCategoryForTab(tab);return c?` / ${c.label}`:''}
function adminTabBlurb(tab){const c=adminCategoryForTab(tab);return c?c.note:'Configure this part of the workspace.'}
function adminToolView(){''',
        'toolview-helpers',
    )

    # 2b. Section shell: .page-head + breadcrumbs -> artifact .settings-page >
    # .admin-panel with back buttons (Dashboard + All settings keep both
    # data-clear-filter and data-admin-home links) and .page-heading.
    # #adminBody keeps its admin-body class; tab bodies are NOT restructured -
    # every control and its wiring stays byte-identical.
    app = swap(
        app,
        ''' $('#view').innerHTML=`<div class="page-head"><div><div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><button data-admin-home>Admin</button><span>›</span><span>${tabLabel(adminTab)}</span></div><h1>${tabLabel(adminTab)}</h1></div></div><div id="adminBody" class="admin-body"></div>`;''',
        ''' $('#view').innerHTML=`<div class="settings-page"><div class="admin-panel active"><div style="display:flex;gap:8px;margin-bottom:18px;flex-wrap:wrap"><button class="btn btn-ghost btn-sm" data-clear-filter type="button">Dashboard</button><button class="btn btn-ghost btn-sm" data-admin-home type="button"><span class="chevron-right" aria-hidden="true" style="transform:scaleX(-1);display:inline-block">›</span> All settings</button></div><div class="page-heading"><div><div class="eyebrow">Administration${adminEyebrowSuffix(adminTab)}</div><h2 style="margin-top:5px">${tabLabel(adminTab)}</h2><p>${adminTabBlurb(adminTab)}</p></div></div><div id="adminBody" class="admin-body"></div></div></div>`;''',
        'toolview-shell',
    )
    return app


# ---------------------------------------------------------------------------
# 3. themeStudioTab() + sub-views
# ---------------------------------------------------------------------------
def transform_theme_studio(app):
    print('region: themeStudioTab() + sub-views')

    # 3a. No-permission gate: .panel -> .admin-panel + .page-heading + .card.
    # Text and the manage_themes guard logic are untouched.
    app = swap(
        app,
        '''  body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Theme Studio</h3><p class="muted" style="font-size:13px">Theme management needs the <code>manage_themes</code> capability, and the <b>Administrator</b> role does not have it - so Theme Studio is hidden. (There is no signed-in user in this demo; the desktop edition resolves this against the real signed-in user's role.)</p></div>`;''',
        '''  body.innerHTML=`<div class="admin-panel active"><div class="page-heading"><div><div class="eyebrow">Administration / Experience</div><h2 style="margin-top:5px">Theme Studio</h2><p>Brand colors, typography, shape and density for this workspace.</p></div></div><div class="card"><div class="card-body"><p class="muted" style="font-size:13px;margin:0">Theme management needs the <code>manage_themes</code> capability, and the <b>Administrator</b> role does not have it - so Theme Studio is hidden. (There is no signed-in user in this demo; the desktop edition resolves this against the real signed-in user's role.)</p></div></div></div>`;''',
        'theme-gate-panel',
    )

    # 3b. Preset picker: .panel -> .admin-panel + .page-heading; presets become
    # artifact button.card.settings-card cards with .swatches/.swatch previews
    # and a chevron. data-preset wiring below is untouched.
    app = swap(
        app,
        ''' body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Theme Studio</h3><p class="muted">Brand colors, typography, shape and density for this workspace. Start from a curated preset or from scratch, then Publish once it passes a WCAG contrast check. Publishing never overwrites history - rolling back creates a new Draft from an older version's tokens and publishes that.</p>
 <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(220px,1fr));gap:12px">${THEME_PRESETS.map(([key,name,blurb,tokens])=>`<button type="button" class="template-card" data-preset="${key}" style="text-align:left">
  <div style="display:flex;gap:4px;margin-bottom:8px">${[tokens.color.brand_primary,tokens.color.brand_secondary,tokens.color.surface_sidebar,tokens.color.status_success].map(c=>`<span style="width:20px;height:20px;border-radius:4px;background:${c};border:1px solid var(--line)"></span>`).join('')}</div>
  <strong>${name}</strong><p class="muted" style="font-size:12px;margin:4px 0 0">${blurb}</p></button>`).join('')}</div>
 </div>''',
        ''' body.innerHTML=`<div class="admin-panel active"><div class="page-heading"><div><div class="eyebrow">Administration / Experience</div><h2 style="margin-top:5px">Theme Studio</h2><p>Brand colors, typography, shape and density for this workspace. Start from a curated preset or from scratch, then Publish once it passes a WCAG contrast check. Publishing never overwrites history - rolling back creates a new Draft from an older version's tokens and publishes that.</p></div></div>
 <section class="card"><div class="card-head"><h3>Start from a preset</h3><p class="card-desc">A curated preset becomes a draft you can keep customizing.</p></div><div class="card-body"><div class="preset-grid">${THEME_PRESETS.map(([key,name,blurb,tokens])=>`<button type="button" class="card settings-card" data-preset="${key}">
  <span class="swatches" aria-hidden="true">${[tokens.color.brand_primary,tokens.color.brand_secondary,tokens.color.surface_sidebar,tokens.color.status_success].map(c=>`<i class="swatch" style="background:${c}"></i>`).join('')}</span>
  <span class="settings-copy"><h3>${name}</h3><p>${blurb}</p></span><span class="chevron-right" aria-hidden="true">›</span></button>`).join('')}</div></div></section>''',
        'theme-presets',
    )

    # 3c. Versions panel: .panel/.panel-head + table.table -> section.card +
    # .card-head.panel-title + table.token-table. Row HTML (themeVersionRowHtml)
    # is intentionally untouched - all data-* hooks and the schedule inputs
    # stay exactly as bindThemeDraftPanel/themeStudioTab expect.
    app = swap(
        app,
        ''' <div class="panel"><div class="panel-head"><h3>Versions</h3></div>
 <div class="table-wrap"><table class="table"><thead><tr><th>Version</th><th>Name</th><th>Status</th><th>Published</th><th></th></tr></thead><tbody>''',
        ''' <section class="card"><div class="card-head panel-title"><div><h3>Versions</h3><p class="card-desc">Draft → Published → Archived. Publishing never overwrites history.</p></div></div><div class="card-body" style="padding-top:14px">
 <div class="table-wrap"><table class="token-table"><thead><tr><th>Version</th><th>Name</th><th>Status</th><th>Published</th><th></th></tr></thead><tbody>''',
        'theme-versions-panel',
    )
    app = swap(
        app,
        ''' </tbody></table></div></div>
 ${themeAuditLogHtml()}''',
        ''' </tbody></table></div></div></section>
 ${themeAuditLogHtml()}''',
        'theme-versions-close',
    )

    # 3d. Audit log panel: .panel -> section.card, table.table -> .token-table.
    app = swap(
        app,
        ''' return `<div class="panel"><div class="panel-head"><h3>Audit log</h3></div>
 <div class="table-wrap"><table class="table"><thead><tr><th>When</th><th>Who</th><th>Action</th><th>Version</th><th>Name</th></tr></thead><tbody>''',
        ''' return `<section class="card"><div class="card-head"><h3>Audit log</h3><p class="card-desc">Every publish, rollback, draft delete and scheduled publish.</p></div><div class="card-body" style="padding-top:14px">
 <div class="table-wrap"><table class="token-table"><thead><tr><th>When</th><th>Who</th><th>Action</th><th>Version</th><th>Name</th></tr></thead><tbody>''',
        'theme-audit-panel',
    )
    app = swap(
        app,
        ''' </tbody></table></div></div>`;''',
        ''' </tbody></table></div></div></section>`;''',
        'theme-audit-close',
    )

    # 3e. Diff table: table.table -> table.token-table (themeDiffModal untouched).
    app = swap(
        app,
        ''' return `<div class="table-wrap"><table class="table"><thead><tr><th>Token</th><th>v${a.version} — ${a.name}</th><th>v${b.version} — ${b.name}</th></tr></thead><tbody>''',
        ''' return `<div class="table-wrap"><table class="token-table"><thead><tr><th>Token</th><th>v${a.version} — ${a.name}</th><th>v${b.version} — ${b.name}</th></tr></thead><tbody>''',
        'theme-diff-table',
    )

    # 3f. Draft editor panel: full artifact restructure. Every id and data-*
    # hook is preserved verbatim so bindThemeDraftPanel() keeps working:
    # themeDraftClose/Name/Font/Size/Radius/RadiusPx/Shadow/ShadowPreview/
    # Density/DensityPreview/Motion/BtnVariant/CardStyle/TableDensity/
    # InputRadius/BadgeStyle/CustomCss/CustomCssCount/Logo/Favicon/HeroArt (+
    # Name/Clear), CheckContrast/SaveDraft/IssuesBox/Preview, plus
    # data-theme-color[-text], data-theme-color-dark[-text], data-theme-chart[-text].

    # 3f-i. Panel shell: .panel/.panel-head + inline grid -> section.card +
    # .card-head.panel-title + .draft-grid.
    app = swap(
        app,
        ''' return `<div class="panel"><div class="panel-head"><h3>${d.id?'Edit draft':'New draft'}</h3><button type="button" class="icon-btn" id="themeDraftClose">Close</button></div>
 <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;align-items:start">''',
        ''' return `<section class="card"><div class="card-head panel-title"><div><h3>${d.id?'Edit draft':'New draft'}</h3><p class="card-desc">Token editor - every change previews live on the right.</p></div><button type="button" class="btn btn-ghost btn-sm" id="themeDraftClose">Close</button></div><div class="card-body">
 <div class="draft-grid">''',
        'draft-shell',
    )

    # 3f-ii. Colors: two-column .field grid -> .token-table (Token | Light | Dark).
    app = swap(
        app,
        ''' <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px">${THEME_COLOR_FIELDS.map(([key,label])=>`<div class="field"><label>${label}</label><div style="display:flex;gap:6px;align-items:center"><input type="color" data-theme-color="${key}" value="${d.tokens.color[key]}" style="width:36px;height:30px;padding:0" aria-label="${label} (light)"><input data-theme-color-text="${key}" value="${d.tokens.color[key]}" style="flex:1" aria-label="${label} hex (light)"></div><div style="display:flex;gap:6px;align-items:center;margin-top:4px"><span class="muted" style="font-size:11px;width:30px">Dark</span><input type="color" data-theme-color-dark="${key}" value="${(d.tokens.dark||{})[key]||d.tokens.color[key]}" style="width:36px;height:30px;padding:0" aria-label="${label} (dark)"><input data-theme-color-dark-text="${key}" value="${(d.tokens.dark||{})[key]||''}" placeholder="Inherit light" style="flex:1" aria-label="${label} hex (dark)"></div></div>`).join('')}</div>''',
        ''' <div class="table-wrap"><table class="token-table"><thead><tr><th>Token</th><th>Light</th><th>Dark</th></tr></thead><tbody>${THEME_COLOR_FIELDS.map(([key,label])=>`<tr><td><strong>${label}</strong><div class="muted" style="font-size:11px"><code>${key}</code></div></td><td><div class="control-row"><input type="color" data-theme-color="${key}" value="${d.tokens.color[key]}" style="width:36px;height:30px;padding:0" aria-label="${label} (light)"><input data-theme-color-text="${key}" value="${d.tokens.color[key]}" style="flex:1;min-width:0" aria-label="${label} hex (light)"></div></td><td><div class="control-row"><input type="color" data-theme-color-dark="${key}" value="${(d.tokens.dark||{})[key]||d.tokens.color[key]}" style="width:36px;height:30px;padding:0" aria-label="${label} (dark)"><input data-theme-color-dark-text="${key}" value="${(d.tokens.dark||{})[key]||''}" placeholder="Inherit light" style="flex:1;min-width:0" aria-label="${label} hex (dark)"></div></td></tr>`).join('')}</tbody></table></div>''',
        'draft-colors-table',
    )

    # 3f-iii. Chart palette: flex row of fixed-width fields -> .form-grid +
    # .control-row.
    app = swap(
        app,
        ''' <div style="display:flex;flex-wrap:wrap;gap:8px">${d.tokens.chart.palette.map((c,i)=>`<div class="field" style="width:90px"><label>Series ${i+1}</label><div style="display:flex;gap:6px;align-items:center"><input type="color" data-theme-chart="${i}" value="${c}" style="width:36px;height:30px;padding:0"><input data-theme-chart-text="${i}" value="${c}" style="width:70px"></div></div>`).join('')}</div>''',
        ''' <div class="form-grid">${d.tokens.chart.palette.map((c,i)=>`<div class="field"><label>Series ${i+1}</label><div class="control-row"><input type="color" data-theme-chart="${i}" value="${c}" style="width:36px;height:30px;padding:0" aria-label="Series ${i+1} color"><input data-theme-chart-text="${i}" value="${c}" style="flex:1;min-width:0" aria-label="Series ${i+1} hex"></div></div>`).join('')}</div>''',
        'draft-chart-palette',
    )

    # 3f-iv. Typography: stacked fields -> .form-grid.
    app = swap(
        app,
        ''' <div class="field"><label>Font family</label><input id="themeDraftFont" value="${d.tokens.typography.font_family}"></div>
 <div class="field"><label>Base size (px)</label><input id="themeDraftSize" type="number" value="${d.tokens.typography.base_size_px}"></div>''',
        ''' <div class="form-grid"><div class="field"><label>Font family</label><input id="themeDraftFont" value="${d.tokens.typography.font_family}"></div>
 <div class="field"><label>Base size (px)</label><input id="themeDraftSize" type="number" value="${d.tokens.typography.base_size_px}"></div></div>''',
        'draft-typography',
    )

    # 3f-v. Shape & density: stacked fields -> .form-grid (open + close).
    app = swap(
        app,
        ''' <h4>Shape &amp; density</h4>
 <div class="field"><label>Corner radius</label>''',
        ''' <h4>Shape &amp; density</h4>
 <div class="form-grid"><div class="field"><label>Corner radius</label>''',
        'draft-shape-open',
    )
    app = swap(
        app,
        ''' <div class="field"><label>Density preview</label><div id="themeDensityPreview">${themeDensityPreviewHtml(d.tokens)}</div></div>
 <h4>Motion</h4>''',
        ''' <div class="field"><label>Density preview</label><div id="themeDensityPreview">${themeDensityPreviewHtml(d.tokens)}</div></div></div>
 <h4>Motion</h4>''',
        'draft-shape-close',
    )

    # 3f-vi. Component overrides: inline grid -> .form-grid.
    app = swap(
        app,
        ''' <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px">
 <div class="field"><label>Primary button</label>''',
        ''' <div class="form-grid">
 <div class="field"><label>Primary button</label>''',
        'draft-overrides-grid',
    )

    # 3f-vii. Draft action buttons: inline flex -> .control-row.
    app = swap(
        app,
        ''' <div style="display:flex;gap:8px;margin-top:12px"><button type="button" class="btn btn-secondary" id="themeCheckContrast">Check contrast</button><button type="button" class="btn btn-primary" id="themeSaveDraft">Save draft</button></div>''',
        ''' <div class="control-row" style="margin-top:12px"><button type="button" class="btn btn-secondary" id="themeCheckContrast">Check contrast</button><button type="button" class="btn btn-primary" id="themeSaveDraft">Save draft</button></div>''',
        'draft-actions',
    )

    # 3f-viii. Draft panel close: panel/grid divs -> card-body/section closes.
    app = swap(
        app,
        ''' <div><h4>Live preview</h4><div id="themePreview"></div></div>
 </div></div>`;''',
        ''' <div><h4>Live preview</h4><div id="themePreview"></div></div>
 </div></div></section>`;''',
        'draft-close',
    )

    # 3g. Close the .admin-panel wrapper opened in 3b (the pristine markup had
    # two self-closed .panel divs; the artifact structure wraps everything).
    app = swap(
        app,
        ''' ${published&&!themeDraft?`<p class="muted" style="font-size:13px">Currently published: <strong>${published.name}</strong> (v${published.version}), applied live across this demo.</p>`:''}`;''',
        ''' ${published&&!themeDraft?`<p class="muted" style="font-size:13px">Currently published: <strong>${published.name}</strong> (v${published.version}), applied live across this demo.</p>`:''}</div>`;''',
        'theme-panel-close',
    )
    return app


# ---------------------------------------------------------------------------
# styles.css: APPEND ONLY
# ---------------------------------------------------------------------------
PRD4_CSS = '''
/* ===== Modern UI PR-D/4 admin+theme-studio — artifact fidelity ===== */
/* Token-mapped from the approved concept artifact: its HSL-channel color form
   maps to plain var(--X) (app tokens are hex, not HSL channels); its alpha
   variants map to color-mix(in srgb, var(--X) calc(A*100%), transparent);
   radii use the --radius-* scale; shadows reuse --shadow-sm. The existing
   [data-theme="dark"] token blocks carry dark mode with no extra work.
   Zero hardcoded hex/radius/shadow. */
.settings-page{max-width:1180px;margin:0 auto;padding:34px 28px 70px}
.page-heading{display:flex;align-items:flex-start;justify-content:space-between;gap:20px;margin-bottom:24px}
.page-heading h2{font-size:var(--text-2xl);font-weight:720;letter-spacing:-.025em;margin:0}
.page-heading p{color:var(--muted-foreground);margin-top:7px;margin-bottom:0}
.settings-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px;margin-top:24px}
.settings-card{width:100%;display:flex;align-items:flex-start;gap:14px;padding:20px;color:inherit;text-align:left;cursor:default}
button.settings-card{cursor:pointer}
button.settings-card:hover{border-color:color-mix(in srgb,var(--primary) 38%,transparent);background:color-mix(in srgb,var(--accent) 28%,transparent)}
.settings-icon{width:42px;height:42px;flex:0 0 auto;display:grid;place-items:center;border-radius:var(--radius);color:var(--accent-foreground);background:var(--accent);font-size:20px}
.settings-copy{min-width:0;flex:1}
.settings-copy h3{margin:0;font-size:var(--text-base);font-weight:680}
.settings-copy p{margin-top:5px;margin-bottom:0;color:var(--muted-foreground);font-size:var(--text-sm)}
.settings-meta{display:flex;gap:8px;flex-wrap:wrap;margin-top:14px}
.settings-count{padding:4px 7px;border-radius:var(--radius-sm);background:var(--secondary);color:var(--muted-foreground);font-size:11px;font-weight:650;border:1px solid transparent}
button.settings-count{cursor:pointer}
button.settings-count:hover{border-color:color-mix(in srgb,var(--primary) 38%,transparent);color:var(--primary)}
.chevron-right{color:var(--muted-foreground);font-size:20px;line-height:1;flex:0 0 auto}
.admin-panel{display:none}
.admin-panel.active{display:block;animation:prd4-view-in .2s ease}
@keyframes prd4-view-in{from{opacity:.4;transform:translateY(3px)}}
.admin-list{margin-top:20px;overflow:hidden}
.admin-row{display:flex;align-items:center;gap:14px;width:100%;padding:16px 18px;border:0;border-top:1px solid var(--border);background:transparent;color:inherit;text-align:left;cursor:pointer}
.admin-row:first-child{border-top:0}
.admin-row:hover{background:color-mix(in srgb,var(--secondary) 55%,transparent)}
.admin-row .settings-icon{width:36px;height:36px;border-radius:var(--radius-md)}
.admin-row-copy{flex:1;min-width:0}
.admin-row-copy p{color:var(--muted-foreground);font-size:12px;margin-top:3px;margin-bottom:0}
.token-table{width:100%;border-collapse:collapse}
.token-table th{text-align:left;color:var(--muted-foreground);padding:0 8px 10px;font-size:12px;font-weight:650}
.token-table td{border-top:1px solid var(--border);padding:11px 8px;vertical-align:middle}
.swatches{display:flex;gap:6px}
.swatch{width:26px;height:26px;border-radius:var(--radius-sm);border:1px solid var(--border)}
.control-row{display:flex;flex-wrap:wrap;gap:20px;align-items:center}
.card-head{padding:20px 20px 0}
.card-head h3{margin:0;font-size:var(--text-base);font-weight:700}
.card-desc{color:var(--muted-foreground);margin-top:5px;margin-bottom:0;font-size:var(--text-sm)}
.card-body{padding:20px}
.panel-title{display:flex;align-items:center;justify-content:space-between;gap:12px}
.preset-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(220px,1fr));gap:12px}
.draft-grid{display:grid;grid-template-columns:1fr 1fr;gap:16px;align-items:start}
.demo-label{font-size:12px;color:var(--muted-foreground);font-weight:650;margin-bottom:7px}
@media (max-width:760px){
 .settings-page{padding:24px 16px 48px}
 .settings-grid{grid-template-columns:1fr}
 .page-heading{flex-direction:column}
 .draft-grid{grid-template-columns:1fr}
}
'''


def main():
    app = open(APP_PATH, encoding='utf-8').read()
    css = open(CSS_PATH, encoding='utf-8').read()

    app = transform_admin_landing(app)
    app = transform_admin_tool_view(app)
    app = transform_theme_studio(app)

    print('styles.css: appending PR-D/4 artifact-fidelity block')
    if 'Modern UI PR-D/4 admin+theme-studio' in css:
        raise SystemExit('CSS FAIL: PR-D/4 block already present - refusing to double-append')
    if not css.endswith('\n'):
        css += '\n'
    css += PRD4_CSS.lstrip('\n')

    open(APP_PATH, 'w', encoding='utf-8').write(app)
    open(CSS_PATH, 'w', encoding='utf-8').write(css)
    print('done: %s, %s' % (APP_PATH, CSS_PATH))


if __name__ == '__main__':
    main()
