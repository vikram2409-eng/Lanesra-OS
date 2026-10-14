#!/usr/bin/env python3
import re, os
REPO = "."
p = os.path.join(REPO, "app.js")
src = open(p, encoding="utf-8").read()
idx = src.find('function releasesPage()')
m = re.search(r"\$\('#app'\)\.innerHTML=`<div class=\"site-dark\">", src[idx:])
if m:
    pos = idx + m.start()
    old = "$('#app').innerHTML=`<div class=\"site-dark\">"
    new = "$('#app').innerHTML=`<div class=\"modern-site\" id=\"modernSite\">${window.MODERN_HEADER||''}<main><div class=\"site-dark\">"
    src = src[:pos] + new + src[pos+len(old):]
    open(p, 'w', encoding='utf-8').write(src)
    print("Fixed releasesPage wrapper")
else:
    print("NOT FOUND")
