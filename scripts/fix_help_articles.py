#!/usr/bin/env python3
"""Convert help article pages to modern template."""
import os
REPO = "."
p = os.path.join(REPO, "app.js")
with open(p, 'rb') as f:
    data = f.read()

# 1. Opening wrapper
old1 = b'<div class="site-dark">${publicNav()}<main class="mkt-page">'
new1 = b'<div class="modern-site" id="modernSite">${window.MODERN_HEADER||\'\'}<main><div class="mkt-page">'
# Only replace in helpTopicPage (find it first)
idx = data.find(b'function helpTopicPage(')
if idx > 0:
    # Find the pattern after idx
    pos = data.find(old1, idx)
    if pos > 0 and pos < idx + 50000:
        data = data[:pos] + new1 + data[pos+len(old1):]
        print("1. Fixed helpTopicPage opening")

# 2. Closing - need to find the specific pattern
# Old: </div></section></main>${publicFooter()}${docsSearchModalHtml()}</div>
# New: </div></section></div></main>${window.MODERN_FOOTER||''}${docsSearchModalHtml()}</div>
old2 = b'</div></section></main>${publicFooter()}${docsSearchModalHtml()}</div>'
new2 = b'</div></section></div></main>${window.MODERN_FOOTER||\'\'}${docsSearchModalHtml()}</div>'
pos2 = data.find(old2, idx)
if pos2 > 0:
    data = data[:pos2] + new2 + data[pos2+len(old2):]
    print("2. Fixed helpTopicPage closing")

# 3. Binding
old3 = b'bindPublicNav();'
# Find the one in helpTopicPage (after the closing)
# There might be multiple, find the one near helpTopicPage
# Let's find all and replace the one after idx
import re
# Find bindPublicNav after helpTopicPage but before next function
next_func = data.find(b'\nfunction ', idx + 100)
section = data[idx:next_func]
if b'bindPublicNav();' in section:
    section = section.replace(b'bindPublicNav();', b"bindModernChrome(document.getElementById('modernSite'));", 1)
    data = data[:idx] + section + data[next_func:]
    print("3. Fixed helpTopicPage binding")

with open(p, 'wb') as f:
    f.write(data)
print("Done")
