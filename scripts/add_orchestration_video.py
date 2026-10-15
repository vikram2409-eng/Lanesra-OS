#!/usr/bin/env python3
"""Add orchestration video to the /product/orchestration page."""
import os
REPO = "."

def main():
    p = os.path.join(REPO, "site-modern-pages.js")
    src = open(p, encoding="utf-8").read()
    
    # Find the product-orchestration page hero and add video after it
    orch_start = src.find('"product-orchestration":')
    if orch_start > 0:
        hero_end = src.find('</section>', orch_start)
        if hero_end > 0:
            video_html = '<section class="shell"><div class="hero-video-wrap"><video preload="metadata" controls playsinline style="width:100%;border-radius:12px;aspect-ratio:16/9;object-fit:cover"><source src="/media/orchestration-overview.mp4" type="video/mp4"></video></div></section>'
            src = src[:hero_end + len('</section>')] + video_html + src[hero_end + len('</section>'):]
            print("Added orchestration video to /product/orchestration page")
        else:
            print("WARNING: hero section not found")
    else:
        print("WARNING: product-orchestration page not found")
    
    open(p, "w", encoding="utf-8").write(src)
    print("Done")

if __name__ == "__main__":
    main()
