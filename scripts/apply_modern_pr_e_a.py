#!/usr/bin/env python3
"""Modern UI PR-E — Worker 1: public pages (issue #390, spec section 6).

Rebuilds the public-page templates in app.js on shared, token-driven
marketing components (.mkt-*), scrubs CRM wording from index.html, and
appends exactly one token-driven CSS block to styles.css (append-only —
no existing CSS is modified).

Scope (non-overlapping with the other PR-E workers):
  app.js    — public-page templates only: publicNav/publicFooter, landing(),
              platformPage, agentFoundryPage, lowCodePage, appCatalogPage,
              appDetailPage, integrationsPage, voicePage, developersPage,
              comparePage, downloadPage, roadmapPage, releasesPage,
              helpHubPage, helpTopicPage (+ the JS hooks that select the
              renamed classes: bindPublicNav, themeAssetsApply).
              Copy, routes, analytics hooks and behavior are unchanged —
              visual/structural only.
  index.html — CRM scrub (8 lines, 14 mentions) + head/meta chrome reskin
              (color-scheme, theme-color annotation, cache-bust bump).
  styles.css — appends one block headed by
              /* ===== PR-E public pages ===== */
              100% token-driven (var() + color-mix only), zero color literals.

Idempotent: every anchor asserts its match count is either 0 (already
applied — skipped) or exactly the expected count; anything else fails
loudly. Running the script twice on the same files is a byte-identical
no-op.

Usage:
  python3 apply_modern_pr_e_a.py [repo_dir]     # default: /tmp/lanesra-os
"""

import os
import sys

REPO = sys.argv[1] if len(sys.argv) > 1 else os.getcwd()

# ---------------------------------------------------------------------------
# app.js — exact byte-sequence replacements (old, new, expected match count)
# ---------------------------------------------------------------------------
APP_REPLACEMENTS = [
    # themeAssetsApply() selects the nav/footer brand marks for the
    # Theme Studio logo swap — keep it covering the renamed classes.
    ("document.querySelectorAll('.brand-mark')",
     "document.querySelectorAll('.brand-mark,.mkt-brand-mark')", 1),
    ("m.closest('.side-brand,a.brand')",
     "m.closest('.side-brand,a.brand,a.mkt-brand')", 1),

    # --- shared nav ---
    ('<nav class="landing-nav">', '<nav class="mkt-nav">', 1),
    ('class="container nav-inner"', 'class="container mkt-nav-inner"', 1),
    ('<a class="brand" href="/">', '<a class="mkt-brand" href="/">', 1),
    ('<span class="brand-mark">L</span>Lanesra OS',
     '<span class="mkt-brand-mark">L</span>Lanesra OS', 2),
    ('class="nav-links"', 'class="mkt-nav-links"', 1),
    ('class="nav-actions"', 'class="mkt-nav-actions"', 1),
    ('class="menu-toggle"', 'class="mkt-menu-toggle"', 1),
    ('class="mobile-drawer"', 'class="mkt-mobile-drawer"', 1),
    # --- shared footer ---
    ('class="brand footer-brand"', 'class="mkt-brand mkt-footer-brand"', 1),
    ('<footer class="footer">', '<footer class="mkt-footer">', 1),
    ('class="container footer-grid"', 'class="container mkt-footer-grid"', 1),
    ('class="container footer-bottom"', 'class="container mkt-footer-bottom"', 1),
    # --- bindPublicNav selectors follow the renames (behavior identical) ---
    ("btn.closest('.landing-nav')", "btn.closest('.mkt-nav')", 1),
    ("nav.querySelector('.mobile-drawer')",
     "nav.querySelector('.mkt-mobile-drawer')", 1),
    ("document.querySelectorAll('.menu-toggle')",
     "document.querySelectorAll('.mkt-menu-toggle')", 1),
    ("document.querySelectorAll('.mobile-drawer a')",
     "document.querySelectorAll('.mkt-mobile-drawer a')", 1),
    ("a.closest('.mobile-drawer')", "a.closest('.mkt-mobile-drawer')", 1),
    ("drawer.closest('.landing-nav').querySelector('.menu-toggle')",
     "drawer.closest('.mkt-nav').querySelector('.mkt-menu-toggle')", 1),

    # --- landing hero / trust bar ---
    ('<section class="hero">', '<section class="mkt-hero">', 1),
    ('class="container hero-grid"', 'class="container mkt-hero-grid"', 1),
    ('class="hero-actions"', 'class="mkt-hero-actions"', 19),
    ('class="hero-screenshot"', 'class="mkt-hero-media"', 6),
    ('class="hero-video-wrap"', 'class="mkt-hero-media-wrap"', 1),
    ('class="hero-video-sound"', 'class="mkt-hero-media-sound"', 1),
    ('class="trust-row"', 'class="mkt-trust-row"', 2),

    # --- page shell ---
    ('class="page-site"', 'class="mkt-page"', 14),
    ('class="page-hero"', 'class="mkt-page-hero"', 14),

    # --- shared cards (capability/industry/bento/platform-layer/download/
    #     choice/app-example cards all become .mkt-card) ---
    ('class="capability-grid"', 'class="mkt-card-grid mkt-card-grid-4"', 1),
    ('<article class="capability-card">', '<article class="mkt-card">', 1),
    ('class="cap-icon"', 'class="mkt-card-icon"', 1),
    ('<a href="${c[3]}" data-analytics-event',
     '<a class="mkt-card-link" href="${c[3]}" data-analytics-event', 1),
    ('class="industry-grid"', 'class="mkt-card-grid"', 3),
    ('<article class="industry-card">', '<article class="mkt-card">', 3),
    ('class="industry-badge"', 'class="badge"', 3),
    ('class="bento"', 'class="mkt-bento"', 3),
    ('<article class="bento-tile">', '<article class="mkt-card">', 22),
    ('<article class="bento-tile lead on-dark">',
     '<article class="mkt-card mkt-lead mkt-on-dark">', 1),
    ('class="bento-icon"', 'class="mkt-card-icon"', 1),
    ('class="platform-layers"', 'class="mkt-card-grid"', 3),
    ('<article class="platform-layer-card">', '<article class="mkt-card">', 3),
    ('class="platform-layer-icon"', 'class="mkt-card-icon"', 3),
    ('<a href="${c.href}"${c.href.startsWith',
     '<a class="mkt-card-link" href="${c.href}"${c.href.startsWith', 1),
    ('class="container download-grid"', 'class="container mkt-card-grid"', 1),
    ('<article class="download-card featured">',
     '<article class="mkt-card mkt-featured">', 1),
    ('<article class="download-card">', '<article class="mkt-card">', 2),
    ('class="sovereignty-cols"', 'class="mkt-card-grid"', 1),
    ('class="choice-card"', 'class="mkt-card"', 3),
    ('class="app-examples"', 'class="mkt-card-grid mkt-card-grid-2"', 1),
    ('<article class="app-example-card">', '<article class="mkt-card">', 1),

    # --- CTA bands ---
    ('class="container cta"', 'class="container mkt-cta"', 9),

    # --- hardcoded color literals in inline styles -> tokens ---
    ('color:#cbd5e1;max-width:640px', 'color:var(--muted);max-width:640px', 8),
    ('color:#cbd5e1;max-width:700px', 'color:var(--muted);max-width:700px', 1),
]

# ---------------------------------------------------------------------------
# index.html — CRM scrub + head/meta chrome reskin
# ---------------------------------------------------------------------------
INDEX_REPLACEMENTS = [
    ("build CRM, operational and industry apps",
     "build sales, operational and industry apps", 1),
    ("open source CRM,", "open source sales app,", 1),
    ("self-hosted CRM,", "self-hosted sales app,", 1),
    ("custom objects CRM,", "custom objects sales app,", 1),
    ("CRM with REST API,", "sales app with REST API,", 1),
    ("offline CRM Windows,", "offline sales app Windows,", 1),
    ("role-based access control CRM,", "sales app with role-based access control,", 1),
    ("voice CRM assistant", "voice sales assistant", 1),
    ("no-code application platform, CRM, industry app catalog",
     "no-code application platform, sales app, industry app catalog", 1),
    ("the same platform underneath a complete CRM out of the box",
     "the same platform underneath a complete sales app out of the box", 1),
    ("Complete CRM sales lifecycle;", "Complete sales lifecycle;", 1),
    ("instead of using a cloud CRM?", "instead of using a cloud sales app?", 1),
    ("Is Lanesra OS just a CRM, or can I build my own app on it?",
     "Is Lanesra OS just a sales app, or can I build my own app on it?", 1),
    ("It ships as a complete CRM out of the box,",
     "It ships as a complete sales app out of the box,", 1),
    # head/meta chrome: declare color-scheme like the concept artifact;
    # bump stale cache-busters so the rebuilt CSS/JS actually reach
    # returning visitors. (theme-color value is set by the cutover script.)
    ('<meta name="twitter:image" content="https://lanesraos.com/social-preview.png" />\n'
     '  <meta name="theme-color" content="#0A0A0A" />',
     '<meta name="twitter:image" content="https://lanesraos.com/social-preview.png" />\n'
     '  <meta name="color-scheme" content="light dark" />\n'
     '  <meta name="theme-color" content="#0A0A0A" />', 1),
    ("/styles.css?v=0.79.0", "/styles.css?v=0.96.0", 1),
    ("/app.js?v=0.79.0", "/app.js?v=0.96.0", 1),
]

# ---------------------------------------------------------------------------
# styles.css — the one appended block (append-only; existing CSS untouched).
# 100% token-driven: var() + color-mix() only, zero color literals.
# Scoped under .site-dark: every public page renders inside that wrapper,
# so each token resolves to the dark token pairs.
# ---------------------------------------------------------------------------
MKT_CSS = """/* ===== PR-E public pages ===== */
/* Modern UI PR-E (issue 390, spec section 6): shared marketing components
   for the public site, rebuilt on the catalog token system. Every rule is
   scoped under .site-dark - all public pages render inside that wrapper,
   so each var() below resolves to the dark token pairs ("dark sections use
   dark token pairs"). This block is 100% token-driven: no color literals. */
.site-dark .mkt-page{min-height:70vh}
/* --- shared nav --- */
.site-dark .mkt-nav{position:sticky;top:0;z-index:20;background:color-mix(in srgb,var(--bg) 78%,transparent);backdrop-filter:blur(16px);-webkit-backdrop-filter:blur(16px);border-bottom:1px solid var(--line)}
.site-dark .mkt-nav-inner{height:72px;display:flex;align-items:center;justify-content:space-between;gap:16px}
.site-dark .mkt-brand{display:flex;align-items:center;gap:10px;font-weight:900;font-size:20px;color:var(--text)}
.site-dark .mkt-brand-mark{width:34px;height:34px;border-radius:11px;background:linear-gradient(135deg,var(--brand),var(--brand2));display:grid;place-items:center;color:var(--bg);font-weight:900;flex:0 0 auto}
.site-dark .mkt-nav-links{display:flex;gap:28px;align-items:center;color:var(--muted);font-weight:600}
.site-dark .mkt-nav-links a{transition:color var(--motion-fast) ease}
.site-dark .mkt-nav-links a:hover{color:var(--brand)}
.site-dark .mkt-nav-actions{display:flex;gap:10px;align-items:center}
.site-dark .mkt-nav-actions .mobile-try{display:inline-flex;padding:12px 17px;white-space:nowrap}
.site-dark .mkt-menu-toggle{display:none;width:46px;height:46px;border:1px solid var(--line);border-radius:13px;background:var(--panel);color:var(--text);font-size:22px;font-weight:800;cursor:pointer;align-items:center;justify-content:center}
.site-dark .mkt-mobile-drawer{display:none}
/* --- shared footer --- */
.site-dark .mkt-footer{padding:34px 0;border-top:1px solid var(--line);color:var(--muted)}
.site-dark .mkt-footer-grid{display:grid;grid-template-columns:2fr 1fr 1fr 1fr 1fr;gap:34px;padding:42px 0 28px}
.site-dark .mkt-footer-grid>div{display:flex;flex-direction:column;gap:10px;align-items:flex-start}
.site-dark .mkt-footer-brand{margin-bottom:5px}
.site-dark .mkt-footer-grid strong{color:var(--text);font-size:13px;text-transform:uppercase;letter-spacing:.06em}
.site-dark .mkt-footer-grid a{color:var(--muted);transition:color var(--motion-fast) ease}
.site-dark .mkt-footer-grid a:hover{color:var(--brand)}
.site-dark .mkt-footer-bottom{border-top:1px solid var(--line);padding:20px 0 28px;display:flex;justify-content:space-between;gap:20px;flex-wrap:wrap;font-size:13px;color:var(--muted)}
/* --- landing hero --- */
.site-dark .mkt-hero{padding:96px 0 74px;background:radial-gradient(circle at 82% 8%,color-mix(in srgb,var(--brand) 10%,transparent),transparent 34%),radial-gradient(circle at 8% 30%,color-mix(in srgb,var(--brand2) 6%,transparent),transparent 32%),radial-gradient(circle at 50% 95%,color-mix(in srgb,var(--brand) 5%,transparent),transparent 40%)}
.site-dark .mkt-hero-grid{display:grid;grid-template-columns:1.05fr .95fr;gap:60px;align-items:center}
.site-dark .mkt-hero h1{font-size:clamp(42px,6vw,72px);line-height:1.02;letter-spacing:-.045em;margin:15px 0 22px;color:var(--text)}
.site-dark .mkt-hero p{color:var(--muted)}
.site-dark .mkt-hero-actions{display:flex;gap:12px;margin-top:28px;flex-wrap:wrap}
.site-dark .mkt-hero-media-wrap{position:relative}
.site-dark .mkt-hero-media{width:100%;height:auto;display:block;border-radius:16px;border:1px solid var(--line);box-shadow:var(--shadow)}
.site-dark .mkt-hero-media-sound{margin-top:10px;border:1px solid var(--line);background:var(--panel);color:var(--text);border-radius:12px;padding:8px 12px;font-size:16px;cursor:pointer}
.site-dark .mkt-trust-row{display:flex;gap:22px;flex-wrap:wrap;margin-top:28px;color:var(--muted);font-weight:600;font-size:14px}
/* --- inner page hero --- */
.site-dark .mkt-page-hero{padding:90px 0 58px;background:radial-gradient(circle at 80% 15%,color-mix(in srgb,var(--brand) 10%,transparent),transparent 30%),var(--bg);border-bottom:1px solid var(--line)}
.site-dark .mkt-page-hero h1{font-size:clamp(42px,6vw,68px);line-height:1.05;letter-spacing:-.045em;margin:14px 0 20px;max-width:900px;color:var(--text)}
.site-dark .mkt-page-hero p{font-size:19px;line-height:1.7;color:var(--muted);max-width:820px}
/* --- shared cards --- */
.site-dark .mkt-card-grid{display:grid;grid-template-columns:repeat(3,1fr);gap:18px}
.site-dark .mkt-card-grid-4{grid-template-columns:repeat(4,1fr)}
.site-dark .mkt-card-grid-2{grid-template-columns:repeat(2,1fr)}
.site-dark .mkt-card{background:color-mix(in srgb,var(--text) 3%,transparent);backdrop-filter:blur(12px);-webkit-backdrop-filter:blur(12px);border:1px solid var(--line);border-radius:var(--radius);padding:26px;color:var(--text);transition:transform var(--motion-base) ease,box-shadow var(--motion-base) ease,border-color var(--motion-base) ease}
.site-dark .mkt-card:hover{transform:translateY(-3px);border-color:color-mix(in srgb,var(--text) 24%,transparent);box-shadow:var(--shadow-lift)}
.site-dark .mkt-card h2{font-size:30px;margin:0 0 12px;color:var(--text)}
.site-dark .mkt-card h3{font-size:20px;margin:0 0 8px;color:var(--text)}
.site-dark .mkt-card h4{font-size:17px;margin:0 0 8px;color:var(--text)}
.site-dark .mkt-card p{margin:0;color:var(--muted);font-size:14px;line-height:1.65}
.site-dark .mkt-card .eyebrow{margin-bottom:8px}
.site-dark .mkt-card .tagline{color:var(--muted);margin:0 0 16px;line-height:1.6}
.site-dark .mkt-card .example-detail{margin:0 0 16px;padding-left:18px;color:var(--muted);line-height:1.75;font-size:14px}
.site-dark .mkt-card .example-detail li{margin-bottom:5px}
.site-dark .mkt-card .built-from{display:flex;flex-wrap:wrap;gap:7px;margin-top:14px}
.site-dark .mkt-card .built-from span{background:var(--accent-soft);color:var(--brand);border-radius:999px;padding:5px 12px;font-size:12px;font-weight:700}
.site-dark .mkt-card .demo-tile-shot{width:100%;height:auto;display:block;border-radius:12px;border:1px solid var(--line);margin-bottom:14px}
.site-dark .mkt-card-icon{width:44px;height:44px;border-radius:13px;background:var(--accent-soft);color:var(--brand);display:grid;place-items:center;font-size:20px;margin-bottom:14px}
.site-dark .mkt-card-link{font-size:13px;font-weight:700;color:var(--brand)}
.site-dark .mkt-card.mkt-featured{border-color:color-mix(in srgb,var(--brand) 45%,transparent);box-shadow:var(--shadow-lift)}
.site-dark .mkt-card.mkt-on-dark{background:color-mix(in srgb,var(--brand) 7%,transparent);border-color:color-mix(in srgb,var(--brand) 20%,transparent)}
/* --- bento layout --- */
.site-dark .mkt-bento{display:grid;grid-template-columns:repeat(6,1fr);gap:16px}
.site-dark .mkt-bento>.mkt-card{grid-column:span 2}
.site-dark .mkt-bento>.mkt-card.mkt-lead{grid-column:span 6;display:grid;grid-template-columns:1.1fr 1fr;gap:24px;align-items:center}
/* --- CTA band --- */
.site-dark .mkt-cta{background:linear-gradient(135deg,var(--panel),color-mix(in srgb,var(--brand) 14%,var(--panel)));border:1px solid var(--line);color:var(--text);border-radius:30px;padding:54px;text-align:center}
.site-dark .mkt-cta h2{font-size:42px;margin:0 0 16px;color:var(--text)}
.site-dark .mkt-cta p{color:var(--muted);max-width:640px;margin:0 auto 24px}
/* --- responsive (mirrors the pre-PR-E marketing breakpoints) --- */
@media(max-width:900px){.site-dark .mkt-nav-links{display:none}.site-dark .mkt-menu-toggle{display:inline-flex}.site-dark .mkt-mobile-drawer{position:absolute;top:74px;right:18px;width:min(330px,calc(100% - 36px));background:var(--panel);border:1px solid var(--line);border-radius:18px;padding:10px;box-shadow:var(--shadow);z-index:50}.site-dark .mkt-mobile-drawer:not([hidden]){display:flex;flex-direction:column}.site-dark .mkt-mobile-drawer a{padding:14px 15px;border-radius:11px;font-weight:750;color:var(--text);min-height:48px;display:flex;align-items:center}.site-dark .mkt-mobile-drawer a:hover{background:color-mix(in srgb,var(--text) 6%,transparent);color:var(--brand)}.site-dark .mkt-mobile-drawer hr{width:100%;border:0;border-top:1px solid var(--line);margin:7px 0}.site-dark .mkt-hero-grid{grid-template-columns:1fr}.site-dark .mkt-card-grid,.site-dark .mkt-card-grid-2,.site-dark .mkt-card-grid-4{grid-template-columns:repeat(2,1fr)}}
@media(max-width:620px){.site-dark .mkt-nav-actions .btn-secondary{display:none}.site-dark .mkt-nav-actions .mobile-try{font-size:14px;padding:12px 14px;border-radius:12px}.site-dark .mkt-menu-toggle{width:44px;height:44px;font-size:21px}.site-dark .mkt-mobile-drawer{top:78px;right:12px;width:calc(100% - 24px)}.site-dark .mkt-hero{padding-top:60px}.site-dark .mkt-hero h1{font-size:43px}.site-dark .mkt-page-hero{padding-top:62px}.site-dark .mkt-page-hero h1{font-size:42px}.site-dark .mkt-card-grid,.site-dark .mkt-card-grid-2,.site-dark .mkt-card-grid-4{grid-template-columns:1fr}.site-dark .mkt-footer-grid{grid-template-columns:1fr}.site-dark .mkt-footer-bottom{flex-direction:column}.site-dark .mkt-bento>.mkt-card,.site-dark .mkt-bento>.mkt-card.mkt-lead{grid-column:span 6}.site-dark .mkt-bento>.mkt-card.mkt-lead{grid-template-columns:1fr}.site-dark .mkt-cta{padding:40px 22px;border-radius:24px}}
"""


def apply_replacements(path, replacements, label):
    """Apply (old, new, expected) triples idempotently. Fails loudly on
    unexpected counts; a 0 count means already applied (no-op)."""
    with open(path, encoding="utf-8") as f:
        src = f.read()
    changed = 0
    for old, new, expected in replacements:
        count = src.count(old)
        if count == 0:
            print("  [skip] already applied: %r" % old[:64], file=sys.stderr)
            continue
        if count != expected:
            sys.exit(
                "CHAIN FAIL [%s]: anchor %r found %dx, expected %dx"
                % (label, old[:80], count, expected)
            )
        src = src.replace(old, new)
        changed += 1
    if changed:
        with open(path, "w", encoding="utf-8") as f:
            f.write(src)
    return changed


def append_css_block(path):
    marker = "/* ===== PR-E public pages ===== */"
    with open(path, encoding="utf-8") as f:
        css = f.read()
    n = css.count(marker)
    if n == 1:
        if not css.rstrip().endswith(MKT_CSS.rstrip()):
            sys.exit(
                "CHAIN FAIL [styles.css]: marker present but appended block "
                "differs from this script's block"
            )
        print("  [skip] CSS block already appended", file=sys.stderr)
        return 0
    if n != 0:
        sys.exit("CHAIN FAIL [styles.css]: marker found %dx" % n)
    # zero color literals allowed in the appended block
    import re

    literals = re.findall(r"#[0-9a-fA-F]{3,8}\b|rgba?\s*\(", MKT_CSS)
    if literals:
        sys.exit("CHAIN FAIL [styles.css]: color literals in new block: %r" % literals[:5])
    css = css.rstrip("\n") + "\n\n" + MKT_CSS
    if not css.endswith("\n"):
        css += "\n"
    with open(path, "w", encoding="utf-8") as f:
        f.write(css)
    return 1


def main():
    app_path = os.path.join(REPO, "app.js")
    index_path = os.path.join(REPO, "index.html")
    css_path = os.path.join(REPO, "styles.css")
    for p in (app_path, index_path, css_path):
        if not os.path.isfile(p):
            sys.exit("CHAIN FAIL: %s not found" % p)

    print("== PR-E worker 1: app.js ==")
    n = apply_replacements(app_path, APP_REPLACEMENTS, "app.js")
    print("   applied %d replacement groups" % n)

    print("== PR-E worker 1: index.html ==")
    n = apply_replacements(index_path, INDEX_REPLACEMENTS, "index.html")
    with open(index_path, encoding="utf-8") as f:
        idx = f.read()
    if "crm" in idx.lower():
        sys.exit("CHAIN FAIL [index.html]: 'crm' still present after scrub")
    print("   applied %d replacement groups; CRM scrub verified clean" % n)

    print("== PR-E worker 1: styles.css ==")
    n = append_css_block(css_path)
    print("   appended block: %s" % ("yes" if n else "already present"))

    print("PR-E WORKER 1 COMPLETE")


if __name__ == "__main__":
    main()
