#!/usr/bin/env python3
"""Add Ontology Graph and Object 360 videos to the platform page."""
import os
REPO = "."

def main():
    p = os.path.join(REPO, "site-modern-pages.js")
    src = open(p, encoding="utf-8").read()
    
    # Find the Ontology Graph dark-band section on the platform page
    # Current: <section class="shell"><div class="dark-band"><h2>See the system behind every process.</h2>...
    old_band = '<section class="shell"><div class="dark-band"><h2>See the system behind every process.</h2><p>The Ontology Graph connects objects, rules, workflows, agents, and interfaces so dependencies remain visible as the platform grows.</p><div class="actions"><a class="button route" href="/developers" data-page="dev-architecture">Explore architecture →</a></div></div></section>'
    
    new_band = '''<section class="shell"><div class="dark-band"><h2>See the system behind every process.</h2><p>The Ontology Graph connects objects, rules, workflows, agents, and interfaces so dependencies remain visible as the platform grows.</p><div class="hero-video-wrap" style="margin:24px 0"><video poster="/media/ontology-graph-overview-poster.jpg" preload="metadata" controls playsinline style="width:100%;border-radius:12px"><source src="/media/ontology-graph-overview.mp4" type="video/mp4"></video></div><div class="actions"><a class="button route" href="/developers" data-page="dev-architecture">Explore architecture →</a><a class="button" href="/help/ontology-graph-interactions">Read the guide →</a></div></div></section>
    <section class="shell section"><div class="section-head"><div class="section-index">OBJECT 360</div><div class="section-title"><h2>Every record, in context.</h2><p>Object 360 shows the record-level relationship graph — everything connected to a single record, with type filters and pivot navigation.</p></div></div><div class="hero-video-wrap"><video poster="/media/object-360-overview-poster.jpg" preload="metadata" controls playsinline style="width:100%;border-radius:12px"><source src="/media/object-360-overview.mp4" type="video/mp4"></video></div><div class="actions" style="margin-top:18px"><a class="button" href="/help/object-360-record-graph">Read the guide →</a></div></section>'''
    
    if old_band in src:
        src = src.replace(old_band, new_band, 1)
        print("Added Ontology Graph and Object 360 videos to platform page")
    else:
        print("WARNING: Ontology Graph section not found")
        # Debug: find the section
        idx = src.find("See the system behind every process")
        if idx > 0:
            print("Found text at:", idx)
            print("Context:", src[idx-100:idx+100][:200])
    
    open(p, "w", encoding="utf-8").write(src)
    print("Done")

if __name__ == "__main__":
    main()
