#!/usr/bin/env python3
"""Add Integration Hub video to the /integrations page."""
import os
REPO = "."

def main():
    p = os.path.join(REPO, "site-modern-pages.js")
    src = open(p, encoding="utf-8").read()
    
    # Find the dev-integrations page hero and add video after it
    int_start = src.find('"dev-integrations":')
    if int_start > 0:
        hero_end = src.find('</section>', int_start)
        if hero_end > 0:
            video_html = '<section class="shell"><div class="hero-video-wrap"><video preload="metadata" controls playsinline style="width:100%;border-radius:12px;aspect-ratio:16/9;object-fit:cover"><source src="/media/integration-hub-overview.mp4" type="video/mp4"></video></div></section>'
            src = src[:hero_end + len('</section>')] + video_html + src[hero_end + len('</section>'):]
            print("Added Integration Hub video to /integrations page")
        else:
            print("WARNING: hero section not found")
    else:
        print("WARNING: dev-integrations page not found")
    
    open(p, "w", encoding="utf-8").write(src)
    print("Done")

if __name__ == "__main__":
    main()
