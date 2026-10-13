#!/usr/bin/env python3
"""Modern UI Revamp PR-E - Worker 3: cutover + legacy CSS removal (issue #392).

Explicitly authorized as part of the approved PR-E build. Non-overlapping
with Workers 1 (public pages) and 2.

WHAT THIS DOES
1. styles.css: deletes 204 dead legacy rule blocks + 6 orphaned @keyframes
   blocks + 1 @media wrapper left empty by the deletions (211 deletions).
   A rule was classified DEAD iff it declares >=1 class in its selector AND
   every one of those classes has ZERO token matches in app.js and
   index.html (boundary-aware grep: (?<![\\w-])name(?![\\w-])). Compound
   selectors are dead only when NONE of their classes appear anywhere.
   Dynamic class construction was audited first (classList.add(var),
   querySelector('.'+x), template-literal class interpolation): the only
   variable interpolations resolve to string literals present in the file
   (fieldHtml/selectHtml cls in {'field','field full'}; pgBtn cls in
   {'btn-secondary','btn-outline'}), so literal matching is sound.
   Proof per deletion is recorded in the DELETIONS table below
   (selector + classes + app.js:0 / index.html:0).
2. index.html: <meta name="theme-color"> #0A0A0A -> #f7f8fb to match the
   locked Light default (was a dark browser-chrome default).
3. manifest.webmanifest: theme_color #111827 -> #f7f8fb (same reason;
   background_color was already #f7f8fb).
4. app.js: INTENTIONALLY UNTOUCHED. The 2 known app.js occurrences
   (.record-head x2) turned out to be a LIVE template - detail360Header()
   renders <div class="record-head"> and is called from 3 live render paths
   (companies, contacts, generic detail) - so its CSS is kept. .prb2-crumbs
   has zero app.js references, so no app.js edit was needed either.

DELIBERATELY KEPT (verified live or unsure - safety margin):
- .record-head and its live rules (used by detail360Header, 3 call sites)
- .panel / .panel-head (246 standalone token matches in app.js; voice convo
  plan + prb2-detail still use .panel) - the "superseded by .card" hypothesis
  was rejected by evidence
- .fact-tile .n (class `n` matches 203 standalone tokens in app.js; kept per
  the conservative compound-selector rule)
- @keyframes prb-beam-spin (live ref: #app[data-card-style="beam"] .card::after),
  tcursor, rmpulse, gradientText (.gradient-name), cardIn (.kpi-card),
  beamSpin (.beam), btn-shimmer, ontology-*
- Every rule whose selector mixes a dead class with any live class
  (e.g. .record-head .muted, .record-head-meta .badge, .timeline:before,
  .trust-cards,.roadmap-columns,.story-grid,.footer-grid{...})

ANCHOR PROTOCOL: every deletion anchor is the exact byte sequence of the
rule block in the pristine tree; the script asserts match-count == 1 and
fails loudly otherwise. Run from the repo root:
    python3 scripts/apply_modern_pr_e_c.py   # (saved as pre-chain script)
The workflow must stage styles.css, index.html and manifest.webmanifest.
"""
import sys

def fail(msg):
    print('FATAL: ' + msg, file=sys.stderr)
    sys.exit(1)

CSS_DELETIONS = [    # .ontology-graph-wrap | ontology-graph-wrap=[app.js:0,index.html:0]
    """.ontology-graph-wrap{position:relative;overflow:auto;max-height:640px;border:1px solid var(--line);border-radius:10px;background-color:var(--card);background-image:radial-gradient(color-mix(in srgb,var(--foreground) 13%,transparent) 1.5px,transparent 1.5px);background-size:24px 24px}""",
    # .ontology-inspector-groups | ontology-inspector-groups=[app.js:0,index.html:0]
    """.ontology-inspector-groups{max-height:260px;overflow-y:auto}""",
    # .mock-top | mock-top=[app.js:0,index.html:0]
    """.mock-top{height:44px;background:#182235;display:flex;align-items:center;padding:0 14px;gap:7px}""",
    # .mock-body | mock-body=[app.js:0,index.html:0]
    """.mock-body{padding:18px;background:#f4f6fa}""",
    # .mock-grid | mock-grid=[app.js:0,index.html:0]
    """.mock-grid{display:grid;grid-template-columns:repeat(2,1fr);gap:12px}""",
    # .mock-card | mock-card=[app.js:0,index.html:0]
    """.mock-card{background:#fff;border:1px solid #e5e7eb;border-radius:14px;padding:15px}""",
    # .mock-card strong | mock-card=[app.js:0,index.html:0]
    """.mock-card strong{font-size:22px}""",
    # .mock-chart | mock-chart=[app.js:0,index.html:0]
    """.mock-chart{grid-column:1/-1;height:170px;display:flex;align-items:end;gap:10px;padding:18px}""",
    # .feature-grid | feature-grid=[app.js:0,index.html:0]
    """.feature-grid{display:grid;grid-template-columns:repeat(3,1fr);gap:18px}""",
    # .feature-card | feature-card=[app.js:0,index.html:0]
    """.feature-card{background:#fff;border:1px solid var(--line);border-radius:var(--radius);padding:24px;box-shadow:0 8px 30px rgba(17,24,39,.04);transition:.2s ease}""",
    # .feature-card:hover | feature-card=[app.js:0,index.html:0]
    """.feature-card:hover{transform:translateY(-3px);box-shadow:var(--shadow-lift);border-color:#d9dbf5}""",
    # .feature-card h3 | feature-card=[app.js:0,index.html:0]
    """.feature-card h3{margin:16px 0 8px}""",
    # .split-3 | split-3=[app.js:0,index.html:0]
    """.split-3{grid-template-columns:repeat(3,1fr)}""",
    # .footer-inner | footer-inner=[app.js:0,index.html:0]
    """.footer-inner{display:flex;justify-content:space-between;gap:20px;flex-wrap:wrap}""",
    # .side-bottom | side-bottom=[app.js:0,index.html:0]
    """.side-bottom{position:absolute;bottom:15px;left:14px;right:14px}""",
    # .top-actions | top-actions=[app.js:0,index.html:0]
    """.top-actions{display:flex;gap:10px;align-items:center}""",
    # .grid-2 | grid-2=[app.js:0,index.html:0]
    """.grid-2{display:grid;grid-template-columns:1.2fr .8fr;gap:18px;margin-top:18px}""",
    # .simple-chart | simple-chart=[app.js:0,index.html:0]
    """.simple-chart{height:220px;display:flex;align-items:end;gap:12px;border-bottom:1px solid var(--line);padding:20px 8px 0}""",
    # .activity-item | activity-item=[app.js:0,index.html:0]
    """.activity-item{display:flex;gap:11px}""",
    # .activity-dot | activity-dot=[app.js:0,index.html:0]
    """.activity-dot{width:9px;height:9px;border-radius:50%;background:#6366f1;margin-top:7px}""",
    # .kanban-col | kanban-col=[app.js:0,index.html:0]
    """.kanban-col{background:#eef1f5;border-radius:15px;padding:12px;min-height:540px}""",
    # .kanban-head | kanban-head=[app.js:0,index.html:0]
    """.kanban-head{display:flex;justify-content:space-between;font-weight:800;margin-bottom:10px}""",
    # .deal-title | deal-title=[app.js:0,index.html:0]
    """.deal-title{font-weight:750}""",
    # .deal-value | deal-value=[app.js:0,index.html:0]
    """.deal-value{font-weight:850;margin-top:10px}""",
    # .mobile-nav-toggle | mobile-nav-toggle=[app.js:0,index.html:0]
    """.mobile-nav-toggle{display:none;border:1px solid var(--line);background:var(--card);border-radius:8px;padding:8px 11px;cursor:pointer;margin-right:12px;font-size:16px}""",
    # .sidebar-backdrop | sidebar-backdrop=[app.js:0,index.html:0]
    """.sidebar-backdrop{display:none}""",
    # .feature-grid | feature-grid=[app.js:0,index.html:0]
    """.feature-grid{grid-template-columns:repeat(2,1fr)}""",
    # .mobile-nav-toggle | mobile-nav-toggle=[app.js:0,index.html:0]
    """.mobile-nav-toggle{display:inline-flex;align-items:center}""",
    # .sidebar-backdrop:not([hidden]) | sidebar-backdrop=[app.js:0,index.html:0]
    """.sidebar-backdrop:not([hidden]){display:block;position:fixed;inset:0;background:rgba(15,23,42,.5);z-index:55}""",
    # .top-actions | top-actions=[app.js:0,index.html:0]
    """.top-actions{gap:6px}""",
    # .mobile-nav-toggle | mobile-nav-toggle=[app.js:0,index.html:0]
    """.mobile-nav-toggle{margin-right:6px;padding:7px 9px}""",
    # .trust-cards | trust-cards=[app.js:0,index.html:0]
    """.trust-cards{display:grid;grid-template-columns:repeat(3,1fr);gap:18px}""",
    # .trust-card | trust-card=[app.js:0,index.html:0]
    """.trust-card{background:#fff;border:1px solid var(--line);border-radius:18px;padding:26px;transition:.2s ease}""",
    # .trust-card:hover | trust-card=[app.js:0,index.html:0]
    """.trust-card:hover{transform:translateY(-3px);box-shadow:var(--shadow)}""",
    # .trust-card span | trust-card=[app.js:0,index.html:0]
    """.trust-card span{font-size:12px;color:#6366f1;font-weight:900}""",
    # .trust-card h3 | trust-card=[app.js:0,index.html:0]
    """.trust-card h3{margin:12px 0 8px}""",
    # .trust-card p | trust-card=[app.js:0,index.html:0]
    """.trust-card p{margin:0;color:#6b7280;line-height:1.6}""",
    # .timeline-item | timeline-item=[app.js:0,index.html:0]
    """.timeline-item{position:relative;display:grid;grid-template-columns:105px 1fr;gap:50px;padding:0 0 48px}""",
    # .timeline-dot | timeline-dot=[app.js:0,index.html:0]
    """.timeline-dot{position:absolute;left:118px;top:5px;width:16px;height:16px;border:3px solid #fff;border-radius:50%;background:#94a3b8;box-shadow:0 0 0 2px #cbd5e1}""",
    # .timeline-date | timeline-date=[app.js:0,index.html:0]
    """.timeline-date{font-weight:850;color:#475569;padding-top:2px}""",
    # .timeline-content | timeline-content=[app.js:0,index.html:0]
    """.timeline-content{background:#fff;border:1px solid var(--line);border-radius:18px;padding:24px;transition:.2s ease}""",
    # .timeline-content:hover | timeline-content=[app.js:0,index.html:0]
    """.timeline-content:hover{box-shadow:var(--shadow-lift);border-color:#d9dbf5}""",
    # .timeline-content h2 | timeline-content=[app.js:0,index.html:0]
    """.timeline-content h2{font-size:24px;margin:0 0 8px}""",
    # .timeline-content p | timeline-content=[app.js:0,index.html:0]
    """.timeline-content p{margin:0 0 12px;color:#64748b;line-height:1.65}""",
    # .roadmap-board | roadmap-board=[app.js:0,index.html:0]
    """.roadmap-board{background:#f8fafc}""",
    # .roadmap-columns | roadmap-columns=[app.js:0,index.html:0]
    """.roadmap-columns{display:grid;grid-template-columns:repeat(3,1fr);gap:18px}""",
    # .roadmap-columns>div | roadmap-columns=[app.js:0,index.html:0]
    """.roadmap-columns>div{background:#fff;border:1px solid var(--line);border-radius:18px;padding:24px}""",
    # .roadmap-columns h2 | roadmap-columns=[app.js:0,index.html:0]
    """.roadmap-columns h2{font-size:22px;margin-top:0}""",
    # .roadmap-row | roadmap-row=[app.js:0,index.html:0]
    """.roadmap-row{padding:9px 0;margin:0;border-bottom:1px solid #f1f5f9;color:#64748b}""",
    # .story-grid | story-grid=[app.js:0,index.html:0]
    """.story-grid{display:grid;grid-template-columns:1.5fr .7fr;gap:42px;align-items:start}""",
    # .story-copy>h2 | story-copy=[app.js:0,index.html:0]
    """.story-copy>h2{font-size:32px;margin:0 0 14px}""",
    # .story-copy>p | story-copy=[app.js:0,index.html:0]
    """.story-copy>p{font-size:17px;line-height:1.8;color:#475569}""",
    # .principle-grid | principle-grid=[app.js:0,index.html:0]
    """.principle-grid{display:grid;grid-template-columns:1fr 1fr;gap:14px;margin-top:22px}""",
    # .maker-card | maker-card=[app.js:0,index.html:0]
    """.maker-card{position:sticky;top:100px;background:var(--ink);color:#fff;border-radius:22px;padding:28px}""",
    # .maker-mark | maker-mark=[app.js:0,index.html:0]
    """.maker-mark{width:54px;height:54px;border-radius:16px;background:linear-gradient(135deg,#6366f1,#8b5cf6);display:grid;place-items:center;font-weight:900;margin-bottom:22px}""",
    # .side-meta | side-meta=[app.js:0,index.html:0]
    """.side-meta{font-size:10px;line-height:1.5;color:color-mix(in srgb,var(--sidebar-foreground) 60%,transparent);text-align:center;margin-bottom:10px}""",
    # .side-meta a | side-meta=[app.js:0,index.html:0]
    """.side-meta a{color:var(--sidebar-foreground)}""",
    # .maker-card | maker-card=[app.js:0,index.html:0]
    """.maker-card{position:static}""",
    # .timeline-item | timeline-item=[app.js:0,index.html:0]
    """.timeline-item{grid-template-columns:1fr;gap:8px;padding-left:46px}""",
    # .timeline-dot | timeline-dot=[app.js:0,index.html:0]
    """.timeline-dot{left:8px}""",
    # .kpi-link | kpi-link=[app.js:0,index.html:0]
    """.kpi-link{appearance:none;text-align:left;color:inherit;font:inherit;cursor:pointer;transition:.18s ease;position:relative}""",
    # .kpi-link:hover | kpi-link=[app.js:0,index.html:0]
    """.kpi-link:hover{transform:translateY(-2px);border-color:#a5b4fc;box-shadow:0 12px 30px rgba(79,70,229,.10)}""",
    # .kpi-link span | kpi-link=[app.js:0,index.html:0]
    """.kpi-link span{display:block;margin-top:10px;color:#4f46e5;font-size:12px;font-weight:800}""",
    # .side-meta | side-meta=[app.js:0,index.html:0]
    """.side-meta{font-size:11px}""",
    # .side-meta a | side-meta=[app.js:0,index.html:0]
    """.side-meta a{display:inline-block;margin-top:3px}""",
    # .kpi-link | kpi-link=[app.js:0,index.html:0]
    """.kpi-link{width:100%}""",
    # .side-product-links | side-product-links=[app.js:0,index.html:0]
    """.side-product-links{display:flex;flex-direction:column;gap:4px;margin:8px 0}""",
    # .side-product-links a | side-product-links=[app.js:0,index.html:0]
    """.side-product-links a{display:block;color:var(--sidebar-foreground);text-decoration:none}""",
    # .side-product-links a:hover | side-product-links=[app.js:0,index.html:0]
    """.side-product-links a:hover{text-decoration:underline;color:var(--sidebar-foreground)}""",
    # .side-meta>span | side-meta=[app.js:0,index.html:0]
    """.side-meta>span{display:block;margin-top:6px}""",
    # .mock-body | mock-body=[app.js:0,index.html:0]
    """.mock-body{padding:14px}""",
    # .mock-grid | mock-grid=[app.js:0,index.html:0]
    """.mock-grid{gap:10px}""",
    # .mock-card | mock-card=[app.js:0,index.html:0]
    """.mock-card{padding:13px}""",
    # .mock-chart | mock-chart=[app.js:0,index.html:0]
    """.mock-chart{height:145px;padding:14px}""",
    # .side-bottom | side-bottom=[app.js:0,index.html:0]
    """.side-bottom{
  position:static;
  flex:0 0 auto;
  margin-top:12px;
  padding-top:12px;
  border-top:1px solid color-mix(in srgb,var(--sidebar-foreground) 14%,transparent);
  background:var(--sidebar);
}""",
    # .side-meta | side-meta=[app.js:0,index.html:0]
    """.side-meta{
  margin-bottom:10px;
}""",
    # .side-product-links | side-product-links=[app.js:0,index.html:0]
    """.side-product-links{
  flex-direction:row;
  justify-content:center;
  flex-wrap:wrap;
  gap:4px 10px;
}""",
    # .side-product-links a | side-product-links=[app.js:0,index.html:0]
    """.side-product-links a{
  white-space:nowrap;
}""",
    # .side-bottom | side-bottom=[app.js:0,index.html:0]
    """.side-bottom{margin-top:8px;padding-top:8px}""",
    # .side-meta | side-meta=[app.js:0,index.html:0]
    """.side-meta{font-size:10px;line-height:1.35;margin-bottom:7px}""",
    # .side-product-links | side-product-links=[app.js:0,index.html:0]
    """.side-product-links{margin:5px 0}""",
    # .side-bottom | side-bottom=[app.js:0,index.html:0]
    """.side-bottom{display:none}""",
    # .voice-orb-idle | voice-orb-idle=[app.js:0,index.html:0]
    """.voice-orb-idle{opacity:.85}""",
    # .voice-orb-listening | voice-orb-listening=[app.js:0,index.html:0]
    """.voice-orb-listening{animation:voiceOrbPulse 1.4s ease-in-out infinite}""",
    # .voice-orb-listening span | voice-orb-listening=[app.js:0,index.html:0]
    """.voice-orb-listening span{animation:voiceOrbRing 1.6s ease-out infinite}""",
    # .voice-orb-listening span:nth-child(2) | voice-orb-listening=[app.js:0,index.html:0]
    """.voice-orb-listening span:nth-child(2){animation-delay:.25s}""",
    # .voice-orb-listening span:nth-child(3) | voice-orb-listening=[app.js:0,index.html:0]
    """.voice-orb-listening span:nth-child(3){animation-delay:.5s}""",
    # .voice-orb-thinking | voice-orb-thinking=[app.js:0,index.html:0]
    """.voice-orb-thinking{animation:voiceOrbSpin 1.1s linear infinite;background:conic-gradient(from 0deg,#4f46e5,#818cf8,#c7d2fe,#4f46e5)}""",
    # .voice-orb-speaking | voice-orb-speaking=[app.js:0,index.html:0]
    """.voice-orb-speaking{animation:voiceOrbBounce .5s ease-in-out infinite alternate;box-shadow:0 0 80px rgba(129,140,248,.75)}""",
    # .voice-convo-row-user | voice-convo-row-user=[app.js:0,index.html:0]
    """.voice-convo-row-user{justify-content:flex-end}""",
    # .voice-convo-row-assistant | voice-convo-row-assistant=[app.js:0,index.html:0]
    """.voice-convo-row-assistant{justify-content:flex-start}""",
    # .builder-layout | builder-layout=[app.js:0,index.html:0]
    """.builder-layout{display:grid;grid-template-columns:1fr 300px;gap:18px;align-items:start}""",
    # .rule360-header | rule360-header=[app.js:0,index.html:0]
    """.rule360-header{display:flex;justify-content:space-between;align-items:flex-start;gap:16px;background:#fff;border:1px solid var(--line);border-radius:18px;padding:22px;margin-bottom:18px;flex-wrap:wrap}""",
    # .rule360-header>div:first-child | rule360-header=[app.js:0,index.html:0]
    """.rule360-header>div:first-child{min-width:0}""",
    # .rule360-header h1 | rule360-header=[app.js:0,index.html:0]
    """.rule360-header h1{margin:6px 0 4px}""",
    # .rule360-meta | rule360-meta=[app.js:0,index.html:0]
    """.rule360-meta{display:flex;gap:16px;flex-wrap:wrap;color:#6b7280;font-size:13px;margin-top:6px}""",
    # .rule360-tabs | rule360-tabs=[app.js:0,index.html:0]
    """.rule360-tabs{display:flex;gap:6px;flex-wrap:wrap;margin-bottom:16px}""",
    # .rule360-grid | rule360-grid=[app.js:0,index.html:0]
    """.rule360-grid{display:grid;grid-template-columns:1.3fr .7fr;gap:18px;align-items:start}""",
    # .rule360-grid>div | rule360-grid=[app.js:0,index.html:0]
    """.rule360-grid>div{min-width:0}""",
    # .rm-stats | rm-stats=[app.js:0,index.html:0]
    """.rm-stats{display:flex;flex-wrap:wrap;gap:10px;margin:30px 0 50px}""",
    # .rm-stat | rm-stat=[app.js:0,index.html:0]
    """.rm-stat{display:flex;align-items:baseline;gap:9px;background:#fff;border:1px solid var(--line);border-radius:999px;padding:11px 20px;box-shadow:0 1px 2px rgba(17,24,39,.03);transition:.2s ease}""",
    # .rm-stat:hover | rm-stat=[app.js:0,index.html:0]
    """.rm-stat:hover{border-color:var(--brand);transform:translateY(-1px);box-shadow:0 10px 24px rgba(79,70,229,.13)}""",
    # .section-dark | section-dark=[app.js:0,index.html:0]
    """.section-dark{background:var(--ink);color:#fff}""",
    # .fact-strip | fact-strip=[app.js:0,index.html:0]
    """.fact-strip{display:grid;grid-template-columns:repeat(4,1fr);gap:1px;background:var(--line);border:1px solid var(--line);border-radius:var(--radius-lg);overflow:hidden;margin-top:-40px;position:relative;z-index:2;box-shadow:var(--shadow-lift)}""",
    # .fact-tile | fact-tile=[app.js:0,index.html:0]
    """.fact-tile{background:#fff;padding:26px 22px;text-align:center}""",
    # .persona-grid | persona-grid=[app.js:0,index.html:0]
    """.persona-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:16px}""",
    # .persona-card | persona-card=[app.js:0,index.html:0]
    """.persona-card{background:#fff;border:1px solid var(--line);border-radius:var(--radius);padding:26px 22px;transition:.2s ease}""",
    # .persona-card:hover | persona-card=[app.js:0,index.html:0]
    """.persona-card:hover{transform:translateY(-3px);box-shadow:var(--shadow-lift);border-color:#d9dbf5}""",
    # .persona-card .persona-icon | persona-card=[app.js:0,index.html:0] persona-icon=[app.js:0,index.html:0]
    """.persona-card .persona-icon{width:42px;height:42px;border-radius:12px;background:var(--accent-soft);color:#4338ca;display:grid;place-items:center;font-size:19px;margin-bottom:16px}""",
    # .persona-card h3 | persona-card=[app.js:0,index.html:0]
    """.persona-card h3{margin:0 0 8px;font-size:17px}""",
    # .persona-card p | persona-card=[app.js:0,index.html:0]
    """.persona-card p{margin:0;color:var(--muted);font-size:13.5px;line-height:1.6}""",
    # .bento-list | bento-list=[app.js:0,index.html:0]
    """.bento-list{margin:0;padding-left:0;list-style:none;display:flex;flex-direction:column;gap:8px}""",
    # .bento-list li | bento-list=[app.js:0,index.html:0]
    """.bento-list li{padding-left:20px;position:relative;font-size:13.5px;color:#374151;line-height:1.5}""",
    # .bento-list li:before | bento-list=[app.js:0,index.html:0]
    """.bento-list li:before{content:"→";position:absolute;left:0;color:var(--brand);font-weight:800}""",
    # .fact-strip | fact-strip=[app.js:0,index.html:0]
    """.fact-strip{grid-template-columns:repeat(2,1fr);margin-top:-24px}""",
    # .persona-grid | persona-grid=[app.js:0,index.html:0]
    """.persona-grid{grid-template-columns:repeat(2,1fr)}""",
    # .fact-strip | fact-strip=[app.js:0,index.html:0]
    """.fact-strip{grid-template-columns:1fr 1fr;margin-top:-18px;border-radius:20px}""",
    # .fact-tile | fact-tile=[app.js:0,index.html:0]
    """.fact-tile{padding:20px 14px}""",
    # .persona-grid | persona-grid=[app.js:0,index.html:0]
    """.persona-grid{grid-template-columns:1fr}""",
    # .terminal-window | terminal-window=[app.js:0,index.html:0]
    """.terminal-window{background:#0d0f13;border:1px solid var(--line);border-radius:20px;box-shadow:0 30px 80px rgba(0,0,0,.5);overflow:hidden;font-family:var(--font-mono)}""",
    # .terminal-top | terminal-top=[app.js:0,index.html:0]
    """.terminal-top{height:40px;background:#14171c;display:flex;align-items:center;padding:0 14px;gap:7px;border-bottom:1px solid var(--line)}""",
    # .terminal-body | terminal-body=[app.js:0,index.html:0]
    """.terminal-body{padding:22px;font-size:13px;line-height:2;color:#d7f7ff}""",
    # .terminal-body .muted-line | muted-line=[app.js:0,index.html:0] terminal-body=[app.js:0,index.html:0]
    """.terminal-body .muted-line{color:var(--muted)}""",
    # .ontology-graph-layout | ontology-graph-layout=[app.js:0,index.html:0]
    """.ontology-graph-layout{display:flex;gap:12px;align-items:stretch}""",
    # .ontology-toolbar | ontology-toolbar=[app.js:0,index.html:0]
    """.ontology-toolbar{display:flex;gap:8px;align-items:center;flex-wrap:wrap;margin:12px 0}""",
    # .ontology-toolbar-group | ontology-toolbar-group=[app.js:0,index.html:0]
    """.ontology-toolbar-group{display:inline-flex;gap:4px}""",
    # .ontology-node-sim-added | ontology-node-sim-added=[app.js:0,index.html:0]
    """.ontology-node-sim-added{box-shadow:0 0 0 2px var(--card),0 0 0 4px var(--success)}""",
    # .ontology-node-sim-removed | ontology-node-sim-removed=[app.js:0,index.html:0]
    """.ontology-node-sim-removed{box-shadow:0 0 0 2px var(--card),0 0 0 4px var(--muted-foreground)}""",
    # .o360-toolbar | o360-toolbar=[app.js:0,index.html:0]
    """.o360-toolbar{display:flex;align-items:center;gap:14px;flex-wrap:wrap;margin:10px 0}""",
    # .o360-breadcrumb | o360-breadcrumb=[app.js:0,index.html:0]
    """.o360-breadcrumb{display:flex;align-items:center;gap:6px;flex-wrap:wrap;font-size:12.5px;margin:8px 0 0}""",
    # .o360-chip | o360-chip=[app.js:0,index.html:0]
    """.o360-chip{display:inline-flex;align-items:center;border-radius:999px;padding:3px 9px;font-size:11px;font-weight:700;white-space:nowrap;margin:1px 2px}""",
    # .o360-chip-pass | o360-chip-pass=[app.js:0,index.html:0]
    """.o360-chip-pass{background:color-mix(in srgb,var(--success) 13%,transparent);color:color-mix(in srgb,var(--success) 78%,black)}""",
    # .o360-chip-fail | o360-chip-fail=[app.js:0,index.html:0]
    """.o360-chip-fail{background:color-mix(in srgb,var(--destructive) 13%,transparent);color:color-mix(in srgb,var(--destructive) 78%,black)}""",
    # .o360-chip-neutral | o360-chip-neutral=[app.js:0,index.html:0]
    """.o360-chip-neutral{background:var(--accent);color:var(--accent-foreground)}""",
    # .btn-destructive | btn-destructive=[app.js:0,index.html:0]
    """.btn-destructive{background:var(--destructive);color:var(--destructive-foreground)}""",
    # .btn-destructive:hover | btn-destructive=[app.js:0,index.html:0]
    """.btn-destructive:hover{filter:brightness(.96)}""",
    # .btn-lg | btn-lg=[app.js:0,index.html:0]
    """.btn-lg{height:48px;padding:0 24px;font-size:var(--text-base);border-radius:var(--radius-lg)}""",
    # .input-error | input-error=[app.js:0,index.html:0]
    """.input-error{border-color:var(--destructive)}""",
    # .input-error:focus | input-error=[app.js:0,index.html:0]
    """.input-error:focus{border-color:var(--destructive);box-shadow:0 0 0 3px color-mix(in srgb,var(--destructive) 25%,transparent)}""",
    # .card-header | card-header=[app.js:0,index.html:0]
    """.card-header{padding:20px 20px 0}""",
    # .card-content | card-content=[app.js:0,index.html:0]
    """.card-content{padding:20px}""",
    # .card-footer | card-footer=[app.js:0,index.html:0]
    """.card-footer{padding:0 20px 20px;display:flex;align-items:center;gap:8px}""",
    # .dialog-overlay | dialog-overlay=[app.js:0,index.html:0]
    """.dialog-overlay{position:fixed;inset:0;z-index:150;background:color-mix(in srgb,var(--foreground) 45%,transparent);display:flex;align-items:center;justify-content:center;padding:20px}""",
    # .dialog-overlay[hidden] | dialog-overlay=[app.js:0,index.html:0]
    """.dialog-overlay[hidden]{display:none}""",
    # .dialog-content | dialog-content=[app.js:0,index.html:0]
    """.dialog-content{background:var(--popover);color:var(--popover-foreground);border:1px solid var(--border);border-radius:var(--radius-lg);box-shadow:var(--shadow-lg);width:min(560px,100%);max-height:88vh;overflow:auto;padding:24px}""",
    # .skeleton | skeleton=[app.js:0,index.html:0]
    """.skeleton{position:relative;overflow:hidden;background:var(--secondary);border-radius:var(--radius-sm);min-height:1em}""",
    # .skeleton::after | skeleton=[app.js:0,index.html:0]
    """.skeleton::after{content:"";position:absolute;inset:0;background:linear-gradient(100deg,transparent 30%,color-mix(in srgb,var(--foreground) 8%,transparent) 50%,transparent 70%);animation:skeleton-shimmer 1.6s ease-in-out infinite}""",
    # .prb-greeting | prb-greeting=[app.js:0,index.html:0]
    """.prb-greeting{
  background:linear-gradient(100deg,var(--foreground) 25%,var(--primary) 50%,var(--foreground) 75%);
  background-size:220% auto;
  -webkit-background-clip:text;
  background-clip:text;
  color:transparent;
  animation:prb-greeting-shift 9s ease-in-out infinite;
}""",
    # .prb-beam | prb-beam=[app.js:0,index.html:0]
    """.prb-beam{position:relative}""",
    # .prb-beam::after | prb-beam=[app.js:0,index.html:0]
    """.prb-beam::after{
  content:"";
  position:absolute;
  inset:-1px;
  border-radius:inherit;
  padding:2px;
  pointer-events:none;
  background:conic-gradient(from var(--prb-beam-angle),transparent 0deg,color-mix(in srgb,var(--primary) 85%,transparent) 45deg,transparent 90deg);
  -webkit-mask:linear-gradient(var(--foreground) 0 0) content-box,linear-gradient(var(--foreground) 0 0);
  -webkit-mask-composite:xor;
  mask-composite:exclude;
  animation:prb-beam-spin 5s linear infinite;
}""",
    # .prb-kpi | prb-kpi=[app.js:0,index.html:0]
    """.prb-kpi{
  appearance:none;
  display:flex;
  flex-direction:column;
  align-items:flex-start;
  gap:var(--space-2);
  padding:var(--space-5);
  margin:0;
  text-align:left;
  color:var(--foreground);
  cursor:pointer;
  transition:transform var(--motion-base) ease,box-shadow var(--motion-base) ease,border-color var(--motion-fast) ease;
}""",
    # .prb-kpi:hover | prb-kpi=[app.js:0,index.html:0]
    """.prb-kpi:hover{
  transform:translateY(-2px);
  box-shadow:var(--shadow-md);
  border-color:color-mix(in srgb,var(--primary) 40%,var(--border));
}""",
    # .prb-kpi:focus-visible | prb-kpi=[app.js:0,index.html:0]
    """.prb-kpi:focus-visible{outline:2px solid var(--ring);outline-offset:2px}""",
    # .prb-kpi-label | prb-kpi-label=[app.js:0,index.html:0]
    """.prb-kpi-label{font-size:var(--text-sm);font-weight:600;color:var(--muted-foreground)}""",
    # .prb-kpi-value | prb-kpi-value=[app.js:0,index.html:0]
    """.prb-kpi-value{font-size:var(--text-2xl);font-weight:800;letter-spacing:-0.02em;color:var(--foreground)}""",
    # .prb-kpi-link | prb-kpi-link=[app.js:0,index.html:0]
    """.prb-kpi-link{font-size:var(--text-xs);font-weight:600;color:var(--primary)}""",
    # .prb-card-head | prb-card-head=[app.js:0,index.html:0]
    """.prb-card-head{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3)}""",
    # .prb-card-head h3 | prb-card-head=[app.js:0,index.html:0]
    """.prb-card-head h3{margin:0;font-size:var(--text-base);font-weight:700;letter-spacing:-0.01em}""",
    # .prb-row-spread | prb-row-spread=[app.js:0,index.html:0]
    """.prb-row-spread{display:flex;justify-content:space-between;align-items:baseline;gap:var(--space-3)}""",
    # .record-head-id | record-head-id=[app.js:0,index.html:0]
    """.record-head-id{display:flex;gap:16px;align-items:flex-start;min-width:0}""",
    # .record-head-avatar | record-head-avatar=[app.js:0,index.html:0]
    """.record-head-avatar{width:56px;height:56px;font-size:var(--text-xl);border-radius:var(--radius-lg);flex-shrink:0}""",
    # .record-head-titles | record-head-titles=[app.js:0,index.html:0]
    """.record-head-titles{min-width:0}""",
    # .record-head-titles h1 | record-head-titles=[app.js:0,index.html:0]
    """.record-head-titles h1{margin:2px 0 6px;font-size:var(--text-2xl);letter-spacing:-.02em;color:var(--foreground)}""",
    # .record-head-meta | record-head-meta=[app.js:0,index.html:0]
    """.record-head-meta{display:flex;align-items:center;gap:14px;flex-wrap:wrap;font-size:var(--text-sm);color:var(--muted-foreground);margin-top:6px}""",
    # .record-head-actions | record-head-actions=[app.js:0,index.html:0]
    """.record-head-actions{display:flex;gap:8px;flex-wrap:wrap;align-items:center}""",
    # .prb2-crumbs | prb2-crumbs=[app.js:0,index.html:0]
    """.prb2-crumbs{color:var(--muted-foreground);font-size:var(--text-xs);margin-bottom:12px}""",
    # .prb2-crumbs button | prb2-crumbs=[app.js:0,index.html:0]
    """.prb2-crumbs button{color:var(--primary);font-weight:600}""",
    # .prb2-crumbs button:hover | prb2-crumbs=[app.js:0,index.html:0]
    """.prb2-crumbs button:hover{text-decoration:underline}""",
    # .card-title | card-title=[app.js:0,index.html:0]
    """.card-title{font-size:var(--text-base);font-weight:700;margin:0;color:var(--card-foreground);display:flex;align-items:center;gap:8px}""",
    # .prb2-card-head-split | prb2-card-head-split=[app.js:0,index.html:0]
    """.prb2-card-head-split{display:flex;align-items:center;justify-content:space-between;gap:12px;flex-wrap:wrap}""",
    # .prb2-field-group | prb2-field-group=[app.js:0,index.html:0]
    """.prb2-field-group{margin-bottom:18px}""",
    # .prb2-field-group:last-child | prb2-field-group=[app.js:0,index.html:0]
    """.prb2-field-group:last-child{margin-bottom:0}""",
    # .prb2-field-group-title | prb2-field-group-title=[app.js:0,index.html:0]
    """.prb2-field-group-title{font-size:var(--text-xs);font-weight:700;text-transform:uppercase;letter-spacing:.06em;color:var(--muted-foreground);margin:0 0 4px}""",
    # .prb2-field-grid | prb2-field-grid=[app.js:0,index.html:0]
    """.prb2-field-grid{display:grid;grid-template-columns:repeat(var(--prb2-cols,2),minmax(0,1fr));gap:0 24px;margin:0}""",
    # .prb2-field-row | prb2-field-row=[app.js:0,index.html:0]
    """.prb2-field-row{padding:9px 0;border-bottom:1px solid var(--border)}""",
    # .prb2-field-row:last-child | prb2-field-row=[app.js:0,index.html:0]
    """.prb2-field-row:last-child{border-bottom:0}""",
    # .prb2-field-row-full | prb2-field-row-full=[app.js:0,index.html:0]
    """.prb2-field-row-full{grid-column:1/-1}""",
    # .prb2-field-row dt | prb2-field-row=[app.js:0,index.html:0]
    """.prb2-field-row dt{font-size:var(--text-xs);font-weight:600;color:var(--muted-foreground);margin-bottom:2px}""",
    # .prb2-field-row dd | prb2-field-row=[app.js:0,index.html:0]
    """.prb2-field-row dd{margin:0;font-size:var(--text-sm);color:var(--foreground);overflow-wrap:anywhere}""",
    # .prb2-activity-fields | prb2-activity-fields=[app.js:0,index.html:0]
    """.prb2-activity-fields{margin:0 0 16px}""",
    # .prb2-timeline | prb2-timeline=[app.js:0,index.html:0]
    """.prb2-timeline{margin:0;padding:4px 0 0}""",
    # .prb2-tl-item | prb2-tl-item=[app.js:0,index.html:0]
    """.prb2-tl-item{position:relative;display:flex;gap:14px;padding:0 0 20px 26px}""",
    # .prb2-tl-item:last-child | prb2-tl-item=[app.js:0,index.html:0]
    """.prb2-tl-item:last-child{padding-bottom:2px}""",
    # .prb2-tl-item::before | prb2-tl-item=[app.js:0,index.html:0]
    """.prb2-tl-item::before{content:"";position:absolute;left:8px;top:20px;bottom:0;width:2px;background:var(--border)}""",
    # .prb2-tl-item:last-child::before | prb2-tl-item=[app.js:0,index.html:0]
    """.prb2-tl-item:last-child::before{display:none}""",
    # .prb2-tl-dot | prb2-tl-dot=[app.js:0,index.html:0]
    """.prb2-tl-dot{position:absolute;left:3px;top:5px;width:12px;height:12px;border-radius:var(--radius-full);box-shadow:0 0 0 3px var(--card),0 0 0 4.5px var(--border)}""",
    # .prb2-tl-email | prb2-tl-email=[app.js:0,index.html:0]
    """.prb2-tl-email{background:var(--info)}""",
    # .prb2-tl-call | prb2-tl-call=[app.js:0,index.html:0]
    """.prb2-tl-call{background:var(--success)}""",
    # .prb2-tl-message | prb2-tl-message=[app.js:0,index.html:0]
    """.prb2-tl-message{background:var(--secondary-foreground)}""",
    # .prb2-tl-body | prb2-tl-body=[app.js:0,index.html:0]
    """.prb2-tl-body{min-width:0;flex:1}""",
    # .prb2-tl-head | prb2-tl-head=[app.js:0,index.html:0]
    """.prb2-tl-head{display:flex;align-items:baseline;gap:8px;flex-wrap:wrap;font-size:var(--text-sm)}""",
    # .prb2-tl-dir | prb2-tl-dir=[app.js:0,index.html:0]
    """.prb2-tl-dir{font-size:var(--text-xs);color:var(--muted-foreground);text-transform:capitalize}""",
    # .prb2-tl-notes | prb2-tl-notes=[app.js:0,index.html:0]
    """.prb2-tl-notes{margin-top:4px;font-size:var(--text-sm);color:var(--foreground)}""",
    # .prb2-tl-meta | prb2-tl-meta=[app.js:0,index.html:0]
    """.prb2-tl-meta{font-size:var(--text-xs);color:var(--muted-foreground);margin-top:4px}""",
    # .prb2-field-grid | prb2-field-grid=[app.js:0,index.html:0]
    """.prb2-field-grid{grid-template-columns:1fr}""",
    # .form-row | form-row=[app.js:0,index.html:0]
    """.form-row{display:flex;gap:12px;flex-wrap:wrap;align-items:flex-end;margin-bottom:4px}""",
    # .form-row label | form-row=[app.js:0,index.html:0]
    """.form-row label{display:flex;flex-direction:column;gap:6px;flex:1 1 180px;font-size:var(--text-sm);font-weight:600;color:var(--foreground)}""",
    # .admin-list | admin-list=[app.js:0,index.html:0]
    """.admin-list{margin-top:20px;overflow:hidden}""",
    # .admin-row | admin-row=[app.js:0,index.html:0]
    """.admin-row{display:flex;align-items:center;gap:14px;width:100%;padding:16px 18px;border:0;border-top:1px solid var(--border);background:transparent;color:inherit;text-align:left;cursor:pointer}""",
    # .admin-row:first-child | admin-row=[app.js:0,index.html:0]
    """.admin-row:first-child{border-top:0}""",
    # .admin-row:hover | admin-row=[app.js:0,index.html:0]
    """.admin-row:hover{background:color-mix(in srgb,var(--secondary) 55%,transparent)}""",
    # .admin-row-copy | admin-row-copy=[app.js:0,index.html:0]
    """.admin-row-copy{flex:1;min-width:0}""",
    # .admin-row-copy p | admin-row-copy=[app.js:0,index.html:0]
    """.admin-row-copy p{color:var(--muted-foreground);font-size:12px;margin-top:3px;margin-bottom:0}""",
    # .demo-label | demo-label=[app.js:0,index.html:0]
    """.demo-label{font-size:12px;color:var(--muted-foreground);font-weight:650;margin-bottom:7px}""",
    # @keyframes voiceOrbPulse - orphaned: sole animation: refs were in deleted rules (app.js:0 index.html:0)
    """@keyframes voiceOrbPulse{0%,100%{transform:scale(1)}50%{transform:scale(1.06)}}""",
    # @keyframes voiceOrbRing - orphaned: sole animation: refs were in deleted rules (app.js:0 index.html:0)
    """@keyframes voiceOrbRing{0%{transform:scale(1);opacity:.6}100%{transform:scale(1.5);opacity:0}}""",
    # @keyframes voiceOrbSpin - orphaned: sole animation: refs were in deleted rules (app.js:0 index.html:0)
    """@keyframes voiceOrbSpin{to{transform:rotate(360deg)}}""",
    # @keyframes voiceOrbBounce - orphaned: sole animation: refs were in deleted rules (app.js:0 index.html:0)
    """@keyframes voiceOrbBounce{from{transform:scale(1)}to{transform:scale(1.1)}}""",
    # @keyframes skeleton-shimmer - orphaned: sole animation: refs were in deleted rules (app.js:0 index.html:0)
    """@keyframes skeleton-shimmer{from{transform:translateX(-100%)}to{transform:translateX(100%)}}""",
    # @keyframes prb-greeting-shift - orphaned: sole animation: refs were in deleted rules (app.js:0 index.html:0)
    """@keyframes prb-greeting-shift{
  0%,100%{background-position:0% center}
  50%{background-position:100% center}
}""",
    # @media (prefers-reduced-motion:reduce) - both inner rules dead (.prb-greeting, .prb-beam::after); wrapper left empty
    """@media (prefers-reduced-motion:reduce){
  .prb-greeting{animation:none;background:none;color:var(--foreground);-webkit-background-clip:border-box;background-clip:border-box}
  .prb-beam::after{animation:none}
}""",
]
def main():
    css = open('styles.css', encoding='utf-8').read()
    for i, anchor in enumerate(CSS_DELETIONS):
        n = css.count(anchor)
        if n != 1:
            fail(f'styles.css deletion #{i}: expected exactly 1 match, found {n}')
        css = css.replace(anchor, '', 1)
    open('styles.css', 'w', encoding='utf-8').write(css)
    print(f'styles.css: {len(CSS_DELETIONS)} dead blocks removed')

    # --- index.html: theme-color -> light default ---
    html = open('index.html', encoding='utf-8').read()
    a = '<meta name="theme-color" content="#0A0A0A" />'
    if html.count(a) != 1:
        fail(f'index.html theme-color anchor: found {html.count(a)} matches')
    html = html.replace(a, '<meta name="theme-color" content="#f7f8fb" />', 1)
    open('index.html', 'w', encoding='utf-8').write(html)
    print('index.html: theme-color -> #f7f8fb (light default)')

    # --- manifest.webmanifest: theme_color -> light default ---
    mf = open('manifest.webmanifest', encoding='utf-8').read()
    b = '"theme_color":"#111827"'
    if mf.count(b) != 1:
        fail(f'manifest.webmanifest theme_color anchor: found {mf.count(b)} matches')
    mf = mf.replace(b, '"theme_color":"#f7f8fb"', 1)
    open('manifest.webmanifest', 'w', encoding='utf-8').write(mf)
    print('manifest.webmanifest: theme_color -> #f7f8fb (light default)')

    print('app.js: untouched (record-head is live; prb2-crumbs has no app.js refs)')

if __name__ == '__main__':
    main()
