#!/usr/bin/env python3
"""Remove site-dark wrapper so roadmap/releases follow homepage theme."""
import os
REPO = "."
p = os.path.join(REPO, "app.js")
with open(p, 'rb') as f:
    data = f.read()

# 1. Remove data-theme="dark" (if present)
old1 = b'<div class="modern-site" id="modernSite" data-theme="dark">'
new1 = b'<div class="modern-site" id="modernSite">'
data = data.replace(old1, new1)

# 2. Remove site-dark wrapper opening
old2 = b'<div class="site-dark"><div class="mkt-page">'
new2 = b'<div class="mkt-page">'
data = data.replace(old2, new2)

# 3. Fix closing tags
old3 = b'</div></div></main>${window.MODERN_FOOTER'
new3 = b'</div></main>${window.MODERN_FOOTER'
data = data.replace(old3, new3)

with open(p, 'wb') as f:
    f.write(data)
print("Done: removed site-dark, pages now follow homepage theme")
