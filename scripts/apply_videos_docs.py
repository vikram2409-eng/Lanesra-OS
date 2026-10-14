#!/usr/bin/env python3
"""Apply videos + docs navigation to the website (idempotent, drift-tolerant).

The authored transform (scripts/site_videos_docs.patch) was generated against
an older app.js; the modernSitePage() shell and SPA route table from an
earlier transform (scripts/site_fresh.patch) have since landed, so a verbatim
`git apply` no longer matches. This script applies the remaining delta -- the
hero-video sound hook, the docs-hub search/browse helpers, and the route
table -- in a context-tolerant, idempotent way.
"""
import subprocess
import sys

APP = "app.js"
PATCH = "scripts/site_videos_docs.patch"

HOOK = (
    "  // Hero video sound toggle (homepage)\n"
    "  bindHeroIntroSound();\n"
    "  // Docs hub: search + browse categories\n"
    "  if(pageId==='docs'){ bindDocsHubSearch(root); renderDocsBrowse(root); }\n"
)

FUNCS = """// Docs hub search: live-filter HELP_CATEGORIES topics
function bindDocsHubSearch(root){
 const input=root.querySelector('#docsHubSearch');
 const results=root.querySelector('#docsHubResults');
 if(!input||!results||typeof HELP_CATEGORIES==='undefined')return;
 const flat=HELP_CATEGORIES.flatMap(c=>c.topics.map(t=>({slug:t.slug,title:t.title,summary:t.summary||'',cat:c.label})));
 function render(q){
  q=(q||'').trim().toLowerCase();
  if(!q){results.hidden=true;results.innerHTML='';return}
  const hits=flat.filter(t=>(t.title+' '+t.summary+' '+t.cat).toLowerCase().includes(q)).slice(0,12);
  if(!hits.length){results.hidden=false;results.innerHTML='<div class="no-results">No documentation matches "'+q.replace(/</g,'&lt;')+'".</div>';return}
  results.hidden=false;
  results.innerHTML=hits.map(h=>`<a href="/help/${h.slug}"><div class="result-title">${h.title.replace(/</g,'&lt;')}</div><div class="result-cat">${h.cat.replace(/</g,'&lt;')}</div></a>`).join('');
 }
 input.addEventListener('input',()=>render(input.value));
 input.addEventListener('focus',()=>render(input.value));
 document.addEventListener('click',e=>{ if(!e.target.closest('.docs-search-wrap')) results.hidden=true; });
}
// Docs hub: render all categories with topic links
function renderDocsBrowse(root){
 const wrap=root.querySelector('#docsBrowseCategories');
 if(!wrap||typeof HELP_CATEGORIES==='undefined')return;
 wrap.innerHTML=HELP_CATEGORIES.map(c=>`<div class="docs-browse-cat"><h3>${c.icon} ${c.label}</h3><p class="cat-blurb">${(c.blurb||'').slice(0,140)}${(c.blurb||'').length>140?'...':''}</p><ul>${c.topics.map(t=>`<li><a href="/help/${t.slug}">${t.title}</a></li>`).join('')}</ul></div>`).join('');
}
"""

NEW_ROUTE = "else if(path==='/demo')appShell();else if(path==='/roadmap')roadmapPage();else if(path==='/releases')releasesPage();else if(path==='/platform'||path==='/build')modernSitePage('product-apps');else if(path==='/agent-foundry')modernSitePage('product-agents');else if(path==='/product/orchestration')modernSitePage('product-orchestration');else if(path==='/low-code')lowCodePage();else if(path==='/apps')appCatalogPage();else if(path.startsWith('/apps/'))appDetailPage(path.slice(6));else if(path==='/integrations')modernSitePage('dev-integrations');else if(path==='/voice')voicePage();else if(path==='/developers')modernSitePage('dev-architecture');else if(path==='/developers/apis')modernSitePage('dev-apis');else if(path==='/developers/mcp')modernSitePage('dev-mcp');else if(path==='/developers/solutions')modernSitePage('dev-solutions');else if(path==='/compare')comparePage();else if(path==='/download')downloadPage();else if(path==='/help')modernSitePage('docs');else if(path==='/pricing')modernSitePage('pricing');else if(path==='/open-source')modernSitePage('open-source');else if(path.startsWith('/help/'))helpTopicPage(path.slice(6));else modernSitePage('home');"


def main():
    src = open(APP, encoding="utf-8").read()

    if "function modernSitePage(" not in src:
        # Predecessor transform absent: use the authored patch verbatim.
        subprocess.run(["git", "apply", PATCH], check=True)
        print("Applied (verbatim patch)")
        return

    changed = []
    fstart = src.index("function modernSitePage(")
    close = src.index("\n}\n", fstart) + 1  # index of the function's closing brace
    head, tail = src[:close + 1], src[close + 1:]
    fspan = head[fstart:]
    if "window.scrollTo(0, 0);" not in fspan:
        sys.exit("FATAL: unexpected modernSitePage() shape; refusing to edit")

    if "bindHeroIntroSound();" in fspan:
        print("hook already present; skipping")
    else:
        at = fstart + fspan.rindex("window.scrollTo(0, 0);")
        head = head[:at] + HOOK + head[at:]
        changed.append("inserted hero-video/docs hook")

    if "function bindDocsHubSearch(" in src:
        print("docs helpers already present; skipping")
    else:
        head = head + "\n" + FUNCS
        changed.append("added docs-hub helpers")

    src = head + tail

    lines = src.split("\n")
    for i, ln in enumerate(lines):
        if ln.startswith("else if(path==='/demo')"):
            if ln != NEW_ROUTE:
                lines[i] = NEW_ROUTE
                changed.append("updated route table")
            else:
                print("route table already current")
            break
    else:
        sys.exit("FATAL: route dispatch line not found in app.js")
    src = "\n".join(lines)

    if changed:
        with open(APP, "w", encoding="utf-8") as fh:
            fh.write(src)
    print("CHANGED: " + (", ".join(changed) if changed else "nothing (already applied)"))
    print("Applied")


main()
