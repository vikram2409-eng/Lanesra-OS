#!/usr/bin/env python3
"""Modern UI Revamp PR-C - Worker B (TS-3 component style overrides,
TS-5 guardrailed custom CSS, TS-6 brand assets) - combined app.js +
styles.css transform.

app.js (9 replacements):
  b1: sidebar brand gains <img class="theme-brand-logo"> (hidden until a
      logo asset is published).
  b2: topbar gains <img class="theme-brand-logo topbar-brand-logo">.
  b3: public nav brand gains <img class="theme-brand-logo">.
  b4: TS-3/TS-5/TS-6 helper functions inserted before themeDraftPanel():
      themeOverrideOptions, themeCustomCssTextareaVal, THEME_CUSTOM_CSS_MAX,
      sanitizeThemeCss, themeCustomCssApply, themeAssetsApply,
      applyWorkspaceThemeExtras.
  b5: "Component styles" + "Custom CSS" + "Brand assets" sections in
      themeDraftPanel(), inserted BEFORE the save-buttons row.
  b6: bindings in bindThemeDraftPanel(): override selects, custom-CSS live
      char count, brand-asset file inputs (500 KB cap, data URLs).
  b7: sanitize gate in the #themeSaveDraft handler - over the 64 KB cap the
      save is rejected with a message; disallowed rules are stripped.
  b8: no-published-theme branch of applyWorkspaceTheme() also clears the
      extras (data attributes, custom CSS element, brand assets).
  b9: applyWorkspaceTheme() calls applyWorkspaceThemeExtras(theme) right
      after applyWorkspaceThemeDark(theme.tokens).

styles.css: one appended section (TS-3 override rules mapped from the #app
data attributes + TS-6 brand-logo sizing + reduced-motion) - 100%
token-driven, no hardcoded colors/radii/shadows.

Every replacement asserts its anchor matches EXACTLY the expected count;
any mismatch fails loudly.

Usage: run from the repo root. Stage both files for the commit.
"""
import sys

APP = 'app.js'
CSS = 'styles.css'

JS_REPLACEMENTS = [
    ('b1:sidebar brand logo img',
     '<div class="side-brand"><span class="brand-mark">L</span><span>Lanesra OS</span><span class="demo-pill">DEMO</span></div>',
     '<div class="side-brand"><img class="theme-brand-logo" alt="Workspace logo" hidden><span class="brand-mark">L</span><span>Lanesra OS</span><span class="demo-pill">DEMO</span></div>',
     1),
    ('b2:topbar logo img',
     '<button class="mobile-nav-toggle" id="mobileNavToggle" aria-label="Open navigation" aria-expanded="false">\u2630</button><div class="search">',
     '<button class="mobile-nav-toggle" id="mobileNavToggle" aria-label="Open navigation" aria-expanded="false">\u2630</button><img class="theme-brand-logo topbar-brand-logo" alt="" hidden><div class="search">',
     1),
    ('b3:public nav brand logo img',
     '<a class="brand" href="/"><span class="brand-mark">L</span>Lanesra OS</a>',
     '<a class="brand" href="/"><img class="theme-brand-logo" alt="Lanesra OS logo" hidden><span class="brand-mark">L</span>Lanesra OS</a>',
     1),
    ('b4:TS-3/TS-5/TS-6 helpers before themeDraftPanel',
     'function themeDraftPanel(){',
     '// Modern UI PR-C (Worker B, TS-3): option list for a component-override\n'
     '// select. Empty value = "Catalog default", which removes the override.\n'
     'function themeOverrideOptions(current,pairs){\n'
     ' return `<option value="" ${current?\'\':\'selected\'}>Catalog default</option>`+pairs.map(([v,l])=>`<option value="${v}" ${current===v?\'selected\':\'\'}>${l}</option>`).join(\'\');\n'
     '}\n'
     '// Modern UI PR-C (Worker B, TS-5): neutralize a closing textarea tag\n'
     '// inside custom CSS so the draft panel markup can never be broken out of.\n'
     'function themeCustomCssTextareaVal(css){return String(css||\'\').replace(/<\\/textarea/gi,\'<\\\\/textarea\')}\n'
     '// Modern UI PR-C (Worker B, TS-5): guardrailed custom CSS. Strips\n'
     '// @import, @font-face blocks with remote URLs, and script-tag\n'
     '// sequences; rejects anything over the 64 KB cap with a message.\n'
     'const THEME_CUSTOM_CSS_MAX=65536;\n'
     'function sanitizeThemeCss(css){\n'
     ' if(typeof css!==\'string\')css=\'\';\n'
     ' if(css.length>THEME_CUSTOM_CSS_MAX)return{ok:false,css:\'\',error:\'Custom CSS is \'+css.length.toLocaleString()+\' characters - the limit is 65,536 (64 KB). Shorten it and try again.\'};\n'
     ' let out=css;\n'
     ' out=out.replace(/@import[^;]*;?/gi,\'\');\n'
     ' out=out.replace(/@font-face\\s*\\{[^{}]*\\}/gi,m=>/url\\s*\\(\\s*[\'"]?(https?:)?\\/\\//i.test(m)?\'\':m);\n'
     ' out=out.replace(/<\\/?script\\b[^>]*>/gi,\'\');\n'
     ' out=out.replace(/<\\/textarea/gi,\'<\\\\/textarea\');\n'
     ' return{ok:true,css:out,error:\'\'};\n'
     '}\n'
     '// Modern UI PR-C (Worker B, TS-5): writes the published theme\'s\n'
     '// sanitized custom CSS into the theme-owned <style id="themeCustomCss">\n'
     '// element. It never touches styles.css (the lint rule exempts this\n'
     '// runtime-injected element). Empty CSS removes the element, so deleting\n'
     '// the theme version removes its CSS too.\n'
     'function themeCustomCssApply(css){\n'
     ' const clean=sanitizeThemeCss(css||\'\');\n'
     ' let el=document.getElementById(\'themeCustomCss\');\n'
     ' if(!clean.ok||!clean.css){if(el)el.remove();return}\n'
     ' if(!el){el=document.createElement(\'style\');el.id=\'themeCustomCss\';document.head.appendChild(el)}\n'
     ' if(el.textContent!==clean.css)el.textContent=clean.css;\n'
     '}\n'
     '// Modern UI PR-C (Worker B, TS-6): brand assets from the published\n'
     '// theme version. The logo swaps every <img class="theme-brand-logo">\n'
     '// (sidebar, topbar, public nav); the favicon swaps <link rel="icon">;\n'
     '// hero artwork sets the homepage hero video poster. Empty assets\n'
     '// restore the shipped defaults. Rollback restores the previous set\n'
     '// automatically because assets are versioned with the theme.\n'
     'function themeAssetsApply(assets){\n'
     ' assets=assets||{};\n'
     ' document.querySelectorAll(\'img.theme-brand-logo\').forEach(img=>{\n'
     '  if(assets.logo){img.src=assets.logo;img.hidden=false}\n'
     '  else{img.removeAttribute(\'src\');img.hidden=true}\n'
     ' });\n'
     ' document.querySelectorAll(\'.brand-mark\').forEach(m=>{\n'
     '  const host=m.closest(\'.side-brand,a.brand\');\n'
     '  m.style.display=(assets.logo&&host&&host.querySelector(\'img.theme-brand-logo\'))?\'none\':\'\';\n'
     ' });\n'
     ' const icon=document.querySelector(\'link[rel="icon"]\');\n'
     ' if(icon){\n'
     '  if(icon.dataset.defaultHref===undefined)icon.dataset.defaultHref=icon.getAttribute(\'href\')||\'\';\n'
     '  icon.setAttribute(\'href\',assets.favicon||icon.dataset.defaultHref);\n'
     ' }\n'
     ' const hero=document.getElementById(\'heroIntroVideo\');\n'
     ' if(hero){\n'
     '  if(hero.dataset.defaultPoster===undefined)hero.dataset.defaultPoster=hero.getAttribute(\'poster\')||\'\';\n'
     '  if(assets.heroArt)hero.setAttribute(\'poster\',assets.heroArt);\n'
     '  else if(hero.dataset.defaultPoster)hero.setAttribute(\'poster\',hero.dataset.defaultPoster);\n'
     '  else hero.removeAttribute(\'poster\');\n'
     ' }\n'
     ' document.querySelectorAll(\'[data-theme-hero]\').forEach(el=>{\n'
     '  el.style.backgroundImage=assets.heroArt?(\'url("\'+assets.heroArt+\'")\'):\'\';\n'
     ' });\n'
     '}\n'
     '// Modern UI PR-C (Worker B, TS-3/TS-5/TS-6): applies the published\n'
     '// theme\'s component style overrides (as data attributes on #app),\n'
     '// guardrailed custom CSS, and brand assets. A "Catalog default" (empty)\n'
     '// override removes its data attribute; a null theme clears everything.\n'
     'function applyWorkspaceThemeExtras(theme){\n'
     ' const appEl=document.getElementById(\'app\');\n'
     ' if(!appEl)return;\n'
     ' const tokens=(theme&&theme.tokens)||{};\n'
     ' const o=tokens.overrides||{};\n'
     ' const setOrClear=(name,val)=>{if(val)appEl.dataset[name]=val;else delete appEl.dataset[name]};\n'
     ' setOrClear(\'btnVariant\',o.primaryButton||\'\');\n'
     ' setOrClear(\'cardStyle\',o.cardStyle||\'\');\n'
     ' setOrClear(\'tableDensity\',o.tableDensity||\'\');\n'
     ' setOrClear(\'inputRadius\',o.inputRadius||\'\');\n'
     ' setOrClear(\'badgeStyle\',o.badgeStyle||\'\');\n'
     ' themeCustomCssApply(tokens.customCss||\'\');\n'
     ' themeAssetsApply(tokens.assets||{});\n'
     '}\n'
     'function themeDraftPanel(){',
     1),
    ('b5:draft panel Component styles + Custom CSS + Brand assets sections',
     ' <div style="display:flex;gap:8px;margin-top:12px"><button type="button" class="btn btn-secondary" id="themeCheckContrast">Check contrast</button><button type="button" class="btn btn-primary" id="themeSaveDraft">Save draft</button></div>',
     ' <h4>Component styles</h4>\n'
     ' <p class="muted" style="font-size:12px;margin:0 0 8px">Per-component variant defaults \u2014 no code needed. "Catalog default" removes the override.</p>\n'
     ' <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px">\n'
     ' <div class="field"><label>Primary button</label><select id="themeDraftBtnVariant">${themeOverrideOptions((d.tokens.overrides||{}).primaryButton||\'\',[[\'solid\',\'Solid\'],[\'shimmer\',\'Shimmer\'],[\'outline\',\'Outline\']])}</select></div>\n'
     ' <div class="field"><label>Card style</label><select id="themeDraftCardStyle">${themeOverrideOptions((d.tokens.overrides||{}).cardStyle||\'\',[[\'bordered\',\'Bordered\'],[\'elevated\',\'Elevated\'],[\'beam\',\'Border beam\']])}</select></div>\n'
     ' <div class="field"><label>Table density</label><select id="themeDraftTableDensity">${themeOverrideOptions((d.tokens.overrides||{}).tableDensity||\'\',[[\'comfortable\',\'Comfortable\'],[\'compact\',\'Compact\']])}</select></div>\n'
     ' <div class="field"><label>Input radius</label><select id="themeDraftInputRadius">${themeOverrideOptions((d.tokens.overrides||{}).inputRadius||\'\',[[\'default\',\'Default\'],[\'small\',\'Small\'],[\'large\',\'Large\']])}</select></div>\n'
     ' <div class="field"><label>Badge style</label><select id="themeDraftBadgeStyle">${themeOverrideOptions((d.tokens.overrides||{}).badgeStyle||\'\',[[\'soft\',\'Soft\'],[\'solid\',\'Solid\'],[\'outline\',\'Outline\']])}</select></div>\n'
     ' </div>\n'
     ' <h4>Custom CSS <span class="muted" style="font-size:11px">custom \u2014 not covered by presets</span></h4>\n'
     ' <div class="field"><label>Scoped custom CSS <span class="muted" style="font-size:11px">(64 KB max \u00b7 applies only when this version is published)</span></label><textarea id="themeDraftCustomCss" rows="8" style="font-family:var(--font-mono);font-size:12px" placeholder="/* e.g. .card{border-radius:18px} */">${themeCustomCssTextareaVal(d.tokens.customCss)}</textarea><div class="muted" style="font-size:11px;margin-top:4px"><span id="themeCustomCssCount">${(d.tokens.customCss||\'\').length}</span> / 65536 characters \u2014 @import, remote-URL @font-face and script tags are stripped on save.</div></div>\n'
     ' <h4>Brand assets</h4>\n'
     ' <p class="muted" style="font-size:12px;margin:0 0 8px">Logo, favicon and hero artwork ride with this theme version \u2014 publishing swaps them in, rollback restores the previous set. SVG/PNG, 500 KB per file.</p>\n'
     ' <div class="field"><label>Logo <span class="muted" style="font-size:11px">(sidebar, topbar, public nav)</span></label><input type="file" id="themeDraftLogo" accept="image/svg+xml,image/png,image/jpeg,image/webp"><div class="muted" style="font-size:11px;margin-top:4px"><span id="themeDraftLogoName"></span> <button type="button" class="link-btn" id="themeDraftLogoClear">Remove</button></div></div>\n'
     ' <div class="field"><label>Favicon</label><input type="file" id="themeDraftFavicon" accept="image/svg+xml,image/png,image/x-icon"><div class="muted" style="font-size:11px;margin-top:4px"><span id="themeDraftFaviconName"></span> <button type="button" class="link-btn" id="themeDraftFaviconClear">Remove</button></div></div>\n'
     ' <div class="field"><label>Hero artwork <span class="muted" style="font-size:11px">(homepage hero)</span></label><input type="file" id="themeDraftHeroArt" accept="image/svg+xml,image/png,image/jpeg,image/webp"><div class="muted" style="font-size:11px;margin-top:4px"><span id="themeDraftHeroArtName"></span> <button type="button" class="link-btn" id="themeDraftHeroArtClear">Remove</button></div></div>\n'
     ' <div style="display:flex;gap:8px;margin-top:12px"><button type="button" class="btn btn-secondary" id="themeCheckContrast">Check contrast</button><button type="button" class="btn btn-primary" id="themeSaveDraft">Save draft</button></div>',
     1),
    ('b6:bindThemeDraftPanel override/css/asset bindings',
     ' $(\'#themeDraftMotion\').onchange=e=>{themeDraft.tokens.motion=e.target.value};',
     ' $(\'#themeDraftMotion\').onchange=e=>{themeDraft.tokens.motion=e.target.value};\n'
     ' // Modern UI PR-C (Worker B, TS-3): component style overrides. Empty =\n'
     ' // catalog default; the key is deleted so pre-override versions keep working.\n'
     ' if(!themeDraft.tokens.overrides)themeDraft.tokens.overrides={};\n'
     ' const themeSetOverride=(key,val)=>{if(val)themeDraft.tokens.overrides[key]=val;else delete themeDraft.tokens.overrides[key]};\n'
     ' $(\'#themeDraftBtnVariant\').onchange=e=>themeSetOverride(\'primaryButton\',e.target.value);\n'
     ' $(\'#themeDraftCardStyle\').onchange=e=>themeSetOverride(\'cardStyle\',e.target.value);\n'
     ' $(\'#themeDraftTableDensity\').onchange=e=>themeSetOverride(\'tableDensity\',e.target.value);\n'
     ' $(\'#themeDraftInputRadius\').onchange=e=>themeSetOverride(\'inputRadius\',e.target.value);\n'
     ' $(\'#themeDraftBadgeStyle\').onchange=e=>themeSetOverride(\'badgeStyle\',e.target.value);\n'
     ' // Modern UI PR-C (Worker B, TS-5): guardrailed custom CSS, live char count.\n'
     ' const themeCssCount=()=>{$(\'#themeCustomCssCount\').textContent=String(($(\'#themeDraftCustomCss\').value||\'\').length)};\n'
     ' $(\'#themeDraftCustomCss\').oninput=themeCssCount;themeCssCount();\n'
     ' // Modern UI PR-C (Worker B, TS-6): brand assets, 500 KB per file, data URLs.\n'
     ' if(!themeDraft.tokens.assets)themeDraft.tokens.assets={};\n'
     ' [[\'Logo\',\'logo\'],[\'Favicon\',\'favicon\'],[\'HeroArt\',\'heroArt\']].forEach(([idPart,key])=>{\n'
     '  const nameEl=$(\'#themeDraft\'+idPart+\'Name\');\n'
     '  const paintAsset=()=>{const v=themeDraft.tokens.assets[key];nameEl.textContent=v?(\'Set \\u00b7 \'+Math.round(v.length/1024)+\' KB data URL\'):\'None\'};\n'
     '  paintAsset();\n'
     '  $(\'#themeDraft\'+idPart).onchange=e=>{\n'
     '   const f=e.target.files&&e.target.files[0];if(!f)return;\n'
     '   if(f.size>512000){alert(\'Brand asset "\'+f.name+\'" exceeds the 500 KB per-file cap.\');e.target.value=\'\';return}\n'
     '   const r=new FileReader();\n'
     '   r.onload=()=>{themeDraft.tokens.assets[key]=r.result;paintAsset()};\n'
     '   r.readAsDataURL(f);\n'
     '  };\n'
     '  $(\'#themeDraft\'+idPart+\'Clear\').onclick=()=>{delete themeDraft.tokens.assets[key];$(\'#themeDraft\'+idPart).value=\'\';paintAsset()};\n'
     ' });',
     1),
    ('b7:save-draft custom-CSS sanitize gate',
     ' $(\'#themeSaveDraft\').onclick=()=>{\n  if(!themeDraft.name.trim())return alert(\'Name is required.\');',
     ' $(\'#themeSaveDraft\').onclick=()=>{\n  if(!themeDraft.name.trim())return alert(\'Name is required.\');\n'
     '  // Modern UI PR-C (Worker B, TS-5): sanitize custom CSS before\n'
     '  // persisting; over the 64 KB cap the save is rejected with a message.\n'
     '  const cssInput=$(\'#themeDraftCustomCss\');\n'
     '  const cssCheck=sanitizeThemeCss(cssInput?cssInput.value:(themeDraft.tokens.customCss||\'\'));\n'
     '  if(!cssCheck.ok)return alert(cssCheck.error);\n'
     '  if(cssCheck.css!==(themeDraft.tokens.customCss||\'\'))toast(\'Custom CSS sanitized: disallowed rules were stripped.\');\n'
     '  themeDraft.tokens.customCss=cssCheck.css;',
     1),
    ('b8:no-theme branch clears extras',
     ' if(!theme){appEl.removeAttribute(\'style\');if(sidebar)sidebar.style.background=\'\';applyWorkspaceThemeDark(null);return}',
     ' if(!theme){appEl.removeAttribute(\'style\');if(sidebar)sidebar.style.background=\'\';applyWorkspaceThemeDark(null);applyWorkspaceThemeExtras(null);return}',
     1),
    ('b9:applyWorkspaceTheme applies extras',
     ' applyWorkspaceThemeDark(theme.tokens);\n}',
     ' applyWorkspaceThemeDark(theme.tokens);\n // Modern UI PR-C (Worker B: TS-3/TS-5/TS-6) - component style overrides,\n // guardrailed custom CSS and brand assets from the published theme version.\n applyWorkspaceThemeExtras(theme);\n}',
     1),
]

CSS_REPLACEMENTS = []

PRC_CSS = [
    ('PR-C/WorkerB TS-3 component overrides + TS-6 brand assets',
     '/* TS-3 \u00b7 Component style overrides \u2014 the published theme\'s overrides land\n'
     '   as data attributes on #app (applyWorkspaceThemeExtras); "Catalog\n'
     '   default" removes the attribute and restores the catalog look.\n'
     '   100% token-driven: no hardcoded colors, radii, or shadows. */\n'
     '\n'
     '/* Primary button variants */\n'
     '#app[data-btn-variant="solid"] .btn-primary{background:var(--primary);color:var(--primary-foreground);border-color:transparent;box-shadow:var(--shadow-sm)}\n'
     '#app[data-btn-variant="shimmer"] .btn-primary{position:relative;overflow:hidden;isolation:isolate}\n'
     '#app[data-btn-variant="shimmer"] .btn-primary::after{content:"";position:absolute;inset:0;background:linear-gradient(105deg,transparent 40%,color-mix(in srgb,var(--primary-foreground) 35%,transparent) 50%,transparent 60%);transform:translateX(-120%);animation:btn-shimmer 2.8s ease-in-out infinite;pointer-events:none}\n'
     '#app[data-btn-variant="outline"] .btn-primary{background:transparent;border-color:var(--primary);color:var(--primary);box-shadow:none}\n'
     '#app[data-btn-variant="outline"] .btn-primary:hover{background:color-mix(in srgb,var(--primary) 10%,transparent)}\n'
     '\n'
     '/* Card styles */\n'
     '#app[data-card-style="bordered"] .card{box-shadow:var(--shadow-sm)}\n'
     '#app[data-card-style="elevated"] .card{box-shadow:var(--shadow-lg)}\n'
     '#app[data-card-style="beam"] .card{position:relative}\n'
     '#app[data-card-style="beam"] .card::after{content:"";position:absolute;inset:-1px;border-radius:inherit;padding:2px;pointer-events:none;background:conic-gradient(from var(--prb-beam-angle),transparent 0deg,color-mix(in srgb,var(--primary) 85%,transparent) 45deg,transparent 90deg);-webkit-mask:linear-gradient(var(--foreground) 0 0) content-box,linear-gradient(var(--foreground) 0 0);-webkit-mask-composite:xor;mask-composite:exclude;animation:prb-beam-spin 5s linear infinite}\n'
     '\n'
     '/* Table density */\n'
     '#app[data-table-density="comfortable"] .table th,#app[data-table-density="comfortable"] .table td{padding:var(--space-4) var(--space-4)}\n'
     '#app[data-table-density="compact"] .table th,#app[data-table-density="compact"] .table td{padding:var(--space-2) var(--space-3)}\n'
     '\n'
     '/* Input radius */\n'
     '#app[data-input-radius="default"] .input,#app[data-input-radius="default"] .textarea,#app[data-input-radius="default"] .select{border-radius:var(--radius-md)}\n'
     '#app[data-input-radius="small"] .input,#app[data-input-radius="small"] .textarea,#app[data-input-radius="small"] .select{border-radius:var(--radius-sm)}\n'
     '#app[data-input-radius="large"] .input,#app[data-input-radius="large"] .textarea,#app[data-input-radius="large"] .select{border-radius:var(--radius-lg)}\n'
     '\n'
     '/* Badge styles */\n'
     '#app[data-badge-style="soft"] .badge{background:color-mix(in srgb,var(--primary) 13%,transparent);color:var(--primary);border-color:color-mix(in srgb,var(--primary) 30%,transparent)}\n'
     '#app[data-badge-style="solid"] .badge{background:var(--primary);color:var(--primary-foreground);border-color:transparent}\n'
     '#app[data-badge-style="outline"] .badge{background:transparent;border-color:var(--primary);color:var(--primary)}\n'
     '\n'
     '/* TS-6 \u00b7 Theme brand-logo <img> (hidden until a logo asset is published) */\n'
     '.theme-brand-logo{height:28px;width:auto;max-width:160px;object-fit:contain;vertical-align:middle}\n'
     '.side-brand .theme-brand-logo{height:30px}\n'
     '.topbar-brand-logo{margin-right:4px}\n'
     'a.brand .theme-brand-logo{height:30px}\n'
     '\n'
     '/* Reduced motion: static shimmer + border beam */\n'
     '@media (prefers-reduced-motion:reduce){\n'
     ' #app[data-btn-variant="shimmer"] .btn-primary::after{animation:none}\n'
     ' #app[data-card-style="beam"] .card::after{animation:none}\n'
     '}\n'),
]


def apply_reps(path, reps):
    with open(path, encoding='utf-8') as f:
        src = f.read()
    for name, old, new, expected in reps:
        n = src.count(old)
        if n != expected:
            sys.exit('ANCHOR FAIL [%s] (%s): expected %d, found %d' % (name, path, expected, n))
        src = src.replace(old, new, n if expected != 1 else 1)
        print('ok [%s] (%s)' % (name, path))
    with open(path, 'w', encoding='utf-8') as f:
        f.write(src)


def main():
    apply_reps(APP, JS_REPLACEMENTS)
    apply_reps(CSS, CSS_REPLACEMENTS)
    with open(CSS, 'a', encoding='utf-8') as f:
        for tag, css in PRC_CSS:
            f.write('\n\n/* ===== Modern UI %s ===== */\n' % tag)
            f.write(css)
            print('appended [%s] to %s' % (tag, CSS))
    print('PR-C WORKER B TRANSFORM COMPLETE')


if __name__ == '__main__':
    main()
