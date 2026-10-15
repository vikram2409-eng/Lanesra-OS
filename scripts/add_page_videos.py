#!/usr/bin/env python3
"""Add agent-foundry and build-apps videos to their pages."""
import os
REPO = "."

def main():
    p = os.path.join(REPO, "site-modern-pages.js")
    src = open(p, encoding="utf-8").read()
    
    # 1. Add agent-foundry video to the Create Agents page (product-agents)
    # Find the product-agents page hero section and add video after it
    # The page starts with '"product-agents":'
    agents_start = src.find('"product-agents":')
    if agents_start > 0:
        # Find the first </section> after the hero (the hero is the first section)
        # Look for the hero section pattern
        hero_end = src.find('</section>', agents_start)
        if hero_end > 0:
            # Insert video section after the hero
            video_html = '<section class="shell"><div class="hero-video-wrap"><video preload="metadata" controls playsinline style="width:100%;border-radius:12px;aspect-ratio:16/9;object-fit:cover"><source src="/media/agent-foundry-overview.mp4" type="video/mp4"></video></div></section>'
            src = src[:hero_end + len('</section>')] + video_html + src[hero_end + len('</section>'):]
            print("Added agent-foundry video to /agent-foundry page")
    
    # 2. Add build-apps video to the Build Apps page (product-apps)
    # Find the product-apps page hero and add video after it
    apps_start = src.find('"product-apps":')
    if apps_start > 0:
        hero_end = src.find('</section>', apps_start)
        if hero_end > 0:
            video_html = '<section class="shell"><div class="hero-video-wrap"><video preload="metadata" controls playsinline style="width:100%;border-radius:12px;aspect-ratio:16/9;object-fit:cover"><source src="/media/build-apps-overview.mp4" type="video/mp4"></video></div></section>'
            src = src[:hero_end + len('</section>')] + video_html + src[hero_end + len('</section>'):]
            print("Added build-apps video to /platform page")
    
    open(p, "w", encoding="utf-8").write(src)
    print("Done")

if __name__ == "__main__":
    main()
