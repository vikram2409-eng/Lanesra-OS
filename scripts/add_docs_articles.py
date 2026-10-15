#!/usr/bin/env python3
"""Add 55 help articles to the modern docs page."""
import json, os, re

REPO = "."

# The articles HTML is embedded here (generated from HELP_CATEGORIES)
# Categories: AI Agent Foundry (12), Voice-First Mode (7), Low-Code Platform (15),
# App Catalog (4), Deployment (4), Integration Hub (6), Architecture (2), Org & Access (5)

def build_articles_html():
    # This will be populated by reading from the repo's app.js HELP_CATEGORIES
    # For the workflow, we extract it dynamically
    src = open(os.path.join(REPO, "app.js"), encoding="utf-8").read()
    cat_headers = [(m.start(), m.group(1), m.group(2)) for m in re.finditer(r"label:'([^']+)',icon:'([^']+)'", src)]
    topics = [(m.start(), m.group(1), m.group(2).replace("\\'", "'")) for m in re.finditer(r"\{slug:'([^']+)',title:'((?:[^'\\]|\\.)*)'", src)]
    cats = []
    for i, (cpos, label, icon) in enumerate(cat_headers):
        next_cpos = cat_headers[i+1][0] if i+1 < len(cat_headers) else len(src)
        cat_topics = [(slug, title) for (tpos, slug, title) in topics if cpos < tpos < next_cpos]
        # Only include the 8 main help categories (filter out non-help categories)
        if cat_topics and label in ['AI Agent Foundry', 'Voice-First Mode', 'Low-Code Application Platform', 'App Catalog & Industry Data Models', 'Deployment Management', 'Integration Hub', 'Architecture & Extensibility', 'Organization & Access Control']:
            cats.append((label, icon, cat_topics))
    
    parts = ['<section class="shell section" id="docs-articles"><div class="section-head"><div class="section-index">02 / BROWSE</div><div class="section-title"><h2>Browse all guides.</h2><p>Step-by-step articles organized by category.</p></div></div>']
    for label, icon, topics in cats:
        parts.append(f'<details class="docs-cat"><summary><span class="docs-cat-icon">{icon}</span><span class="docs-cat-label">{label}</span><span class="docs-cat-count">{len(topics)} guides</span></summary><div class="docs-cat-topics">')
        for slug, title in topics:
            te = title.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
            parts.append(f'<a class="doc-article-link" href="/help/{slug}"><strong>{te}</strong><span class="arrow">Read guide →</span></a>')
        parts.append('</div></details>')
    parts.append('</section>')
    return ''.join(parts)

def main():
    p = os.path.join(REPO, "site-modern-pages.js")
    src = open(p, encoding="utf-8").read()
    
    # Check if already added
    if 'id="docs-articles"' in src:
        print("Articles already present, skipping")
        return
    
    articles_html = build_articles_html()
    print(f"Built articles HTML: {len(articles_html)} chars")
    
    # Find the docs section and insert after the "01 / START HERE" section
    # The docs page has: <section class="shell section"><div class="section-head"><div class="section-index">01 / START HERE</div>
    # We want to insert after the closing </section> of that section
    
    # Find the docs article
    docs_start = src.find('"docs": `<article')
    if docs_start == -1:
        print("ERROR: docs section not found")
        return
    
    # Find the first </section> after the "01 / START HERE" marker
    marker = '01 / START HERE'
    marker_pos = src.find(marker, docs_start)
    if marker_pos == -1:
        print("ERROR: marker not found")
        return
    
    # Find the closing </section> after the marker (need to balance)
    # Simpler: find the pattern '</div></section>' after the marker
    # The section ends with </div></section> - let's find the second occurrence
    search_from = marker_pos
    # Find the end of the docs-grid section
    # Look for '</section>' that closes the 01 section
    # The structure is: <section class="shell section">...<div class="docs-grid">...</div></section>
    # We'll find '</div></section>' after the docs-grid
    grid_end = src.find('</div></section>', search_from)
    if grid_end == -1:
        print("ERROR: section end not found")
        return
    
    insert_pos = grid_end + len('</div></section>')
    src = src[:insert_pos] + articles_html + src[insert_pos:]
    
    # Also update the "Getting started" card link from external to internal
    old_link = 'href="https://lanesraos.com/help" target="_blank" rel="noopener"><div class="icon-box">GS</div><h3>Getting started</h3>'
    new_link = 'href="/help/workspace-essentials" class="route"><div class="icon-box">GS</div><h3>Getting started</h3>'
    # Only replace within docs section
    docs_end = src.find('",\n  "pricing":', docs_start)
    docs_section = src[docs_start:docs_end]
    if old_link in docs_section:
        docs_section = docs_section.replace(old_link, new_link, 1)
        src = src[:docs_start] + docs_section + src[docs_end:]
        print("Updated Getting started link")
    
    open(p, "w", encoding="utf-8").write(src)
    print("Done: articles added to docs page")

if __name__ == "__main__":
    main()
