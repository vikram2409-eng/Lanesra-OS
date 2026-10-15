#!/usr/bin/env python3
"""Force dark theme on roadmap/releases for consistent backgrounds."""
import os
REPO = "."
p = os.path.join(REPO, "app.js")
src = open(p, encoding='utf-8').read()
old = '<div class="modern-site" id="modernSite">${window.MODERN_HEADER'
new = '<div class="modern-site" id="modernSite" data-theme="dark">${window.MODERN_HEADER'
count = src.count(old)
if count >= 2:
    src = src.replace(old, new)
    open(p, 'w', encoding='utf-8').write(src)
    print(f"Fixed {count} wrappers")
else:
    print(f"Found {count}, expected 2")
