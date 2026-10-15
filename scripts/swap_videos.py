#!/usr/bin/env python3
"""Swap the videos on platform and agent-foundry pages."""
import os
REPO = "."

def main():
    p = os.path.join(REPO, "site-modern-pages.js")
    src = open(p, encoding="utf-8").read()
    
    # Use a placeholder to avoid double-replacement
    src = src.replace('/media/agent-foundry-overview.mp4', '/media/__TEMP_SWAP__.mp4')
    src = src.replace('/media/build-apps-overview.mp4', '/media/agent-foundry-overview.mp4')
    src = src.replace('/media/__TEMP_SWAP__.mp4', '/media/build-apps-overview.mp4')
    
    open(p, "w", encoding="utf-8").write(src)
    print("Swapped videos: agent-foundry page now has build-apps video, platform page now has agent-foundry video")
    print("Done")

if __name__ == "__main__":
    main()
